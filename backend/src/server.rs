use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{OriginalUri, Path, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::TryStreamExt;
use http::Uri;
use reqwest::Client;
use rust_embed::RustEmbed;
use tokio::signal;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

use crate::engine::TradingEngine;
use crate::mcp::{GetPriceTool, McpServer, PlaceOrderTool};

#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    upstream: Arc<String>,
    client: Client,
}

/// 运行 HTTP 服务器（在独立的 Tokio 运行时中）
pub async fn run_http_server(addr: SocketAddr, url: String) -> anyhow::Result<()> {
    // 初始化 MCP Server
    let mut mcp_server = McpServer::new();
    mcp_server.register_tool(GetPriceTool::schema(), Box::new(GetPriceTool));
    mcp_server.register_tool(PlaceOrderTool::schema(), Box::new(PlaceOrderTool));
    let mcp_server = Arc::new(mcp_server);

    // 初始化 Trading Engine
    let _trading_engine = TradingEngine::new(mcp_server.clone());

    // TODO: 注册 LLM Providers 和 Market Adapters
    // trading_engine.register_llm_provider("openai".to_string(), Box::new(OpenAiProvider::new()));
    // trading_engine.register_market("binance".to_string(), Box::new(BinanceAdapter::new()));

    info!("Initialized MCP Server and Trading Engine");

    let upstream =
        std::env::var("NOF1_API_BASE_URL").unwrap_or_else(|_| "https://nof1.ai/api".to_string());
    let upstream = upstream.trim_end_matches('/').to_string();

    let client = Client::builder()
        .tcp_keepalive(Duration::from_secs(60))
        .user_agent("nof0-backend/0.1")
        .build()
        .context("failed to build reqwest client")?;

    let state = AppState {
        upstream: Arc::new(upstream),
        client,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers([header::ETAG, header::LAST_MODIFIED]);

    let app = Router::new()
        .route("/api/nof1/{*path}", get(proxy))
        .route("/health", get(health))
        .fallback(static_handler)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    info!("starting server at {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

async fn proxy(
    State(state): State<AppState>,
    Path(path): Path<String>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let target = build_target(&state.upstream, &path, &uri);

    let mut req = state.client.get(target);
    req = req.header(header::ACCEPT, "application/json");
    if let Some(val) = headers.get(header::IF_NONE_MATCH) {
        req = req.header(header::IF_NONE_MATCH, val.clone());
    }
    if let Some(val) = headers.get(header::IF_MODIFIED_SINCE) {
        req = req.header(header::IF_MODIFIED_SINCE, val.clone());
    }

    let upstream = req.send().await.map_err(|err| {
        error!(?err, "upstream request failed");
        StatusCode::BAD_GATEWAY
    })?;

    let status = upstream.status();
    let mut builder = Response::builder().status(status);

    let first_segment = path.split('/').find(|s| !s.is_empty()).unwrap_or("");
    let cache_header = cache_control(first_segment);

    {
        let headers_mut = builder
            .headers_mut()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        headers_mut.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_str(&cache_header).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        );
        headers_mut.insert(
            HeaderName::from_static("cdn-cache-control"),
            HeaderValue::from_str(&cache_header).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        );
        headers_mut.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        headers_mut.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));

        if let Some(ct) = upstream.headers().get(header::CONTENT_TYPE) {
            headers_mut.insert(header::CONTENT_TYPE, ct.clone());
        } else {
            headers_mut.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json; charset=utf-8"),
            );
        }
        if let Some(etag) = upstream.headers().get(header::ETAG) {
            headers_mut.insert(header::ETAG, etag.clone());
        }
        if let Some(last) = upstream.headers().get(header::LAST_MODIFIED) {
            headers_mut.insert(header::LAST_MODIFIED, last.clone());
        }
    }

    let stream = upstream.bytes_stream();
    let body = Body::from_stream(stream.map_err(|err| {
        error!(?err, "streaming upstream body failed");
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }));

    builder.body(body).map_err(|err| {
        error!(?err, "failed to build response body");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn static_handler(OriginalUri(uri): OriginalUri) -> Response {
    let requested_path = uri.path();

    if let Some(normalized) = sanitize_path(requested_path) {
        if let Some(response) = asset_response(&normalized) {
            return response;
        }

        if looks_like_static_asset(&normalized) {
            return StatusCode::NOT_FOUND.into_response();
        }
    } else {
        return StatusCode::BAD_REQUEST.into_response();
    }

    match asset_response("index.html") {
        Some(response) => response,
        None => {
            error!(path = requested_path, "missing embedded index.html");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn asset_response(path: &str) -> Option<Response> {
    Assets::get(path).map(|asset| {
        let body = Body::from(asset.data.into_owned());
        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::OK;

        if let Ok(content_type) =
            HeaderValue::from_str(mime_guess::from_path(path).first_or_octet_stream().as_ref())
        {
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, content_type);
        }

        let cache_header = if path == "index.html" {
            HeaderValue::from_static("no-cache")
        } else {
            HeaderValue::from_static("public, max-age=31536000, immutable")
        };
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, cache_header);
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));

        response
    })
}

fn sanitize_path(path: &str) -> Option<String> {
    if path.contains("..") {
        return None;
    }

    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some("index.html".to_string());
    }

    let mut owned = trimmed.to_string();
    if owned.ends_with('/') {
        owned.push_str("index.html");
    }

    Some(owned)
}

fn looks_like_static_asset(path: &str) -> bool {
    path.split('/')
        .last()
        .and_then(|name| name.split('.').nth(1))
        .is_some()
}

async fn health() -> impl IntoResponse {
    Json(HashMap::from([("status", "ok")]))
}

fn cache_control(segment: &str) -> String {
    let ttl = match segment {
        "crypto-prices" => 5,
        "account-totals" => 15,
        "conversations" => 30,
        "leaderboard" => 60,
        "trades" => 300,
        "since-inception-values" => 600,
        "analytics" => 300,
        _ => 30,
    };
    let s_max = (ttl * 2).max(30);
    let swr = (ttl * 4).max(60);
    format!(
        "public, max-age={}, s-maxage={}, stale-while-revalidate={}",
        ttl, s_max, swr
    )
}

fn build_target(base: &str, path: &str, uri: &Uri) -> String {
    let mut target = base.to_string();
    if !target.ends_with('/') {
        target.push('/');
    }
    target.push_str(path.trim_start_matches('/'));
    if let Some(query) = uri.query() {
        target.push('?');
        target.push_str(query);
    }
    target
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut sig) = signal(SignalKind::terminate()) {
            sig.recv().await;
        }
    };

    #[cfg(windows)]
    let terminate = async {
        // On Windows, we don't have SIGTERM, so just wait for ctrl_c
        std::future::pending::<()>().await;
    };

    #[cfg(not(any(unix, windows)))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("received ctrl_c signal");
        },
        _ = terminate => {
            info!("received terminate signal");
        },
    }

    info!("shutdown signal completed");
}
