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

use crate::api::{adapters::CtpMockAdapter, create_unified_routes, ExchangeManager};
use crate::config::BrokersConfig;
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

    // 加载经纪商配置
    let brokers_config = match BrokersConfig::load_default() {
        Ok(config) => {
            info!("Loaded {} brokers from config", config.brokers.len());
            Arc::new(config)
        }
        Err(e) => {
            error!("Failed to load brokers config: {}, using default", e);
            Arc::new(BrokersConfig::default())
        }
    };

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

    // 创建主应用路由(需要 AppState)
    let app_routes = Router::new()
        .route("/api/nof1/{*path}", get(proxy))
        .route(
            "/api/config/brokers",
            get(|| async move {
                // 每次请求时实时读取配置文件,支持动态更新
                let config = BrokersConfig::load_default().unwrap_or_default();
                Json(config)
            }),
        )
        .route("/api/config/exchanges", get(get_exchanges_config))
        .route("/health", get(health))
        .fallback(static_handler)
        .with_state(state);

    // 创建统一 API 交易所管理器
    info!("🚀 Initializing Unified API Exchange Manager");
    let mut exchange_manager = ExchangeManager::new();

    // 注册 CTP Mock 适配器
    let ctp_adapter = Arc::new(CtpMockAdapter::new("ctp", "CTP期货"));
    exchange_manager.register(ctp_adapter);
    info!("✅ Registered exchange adapter: CTP");

    // TODO: 注册更多交易所适配器
    // let binance_adapter = Arc::new(BinanceAdapter::new("binance", "币安"));
    // exchange_manager.register(binance_adapter);

    let exchange_manager = Arc::new(exchange_manager);

    // 创建统一 API 路由(无状态)
    let unified_routes = create_unified_routes(exchange_manager.clone());

    // 创建 CTP 路由 (快速版本：直接使用 Broker)
    #[cfg(feature = "ctp-real")]
    let ctp_routes = {
        info!("Initializing CTP Quick Web API routes (with Broker)");
        crate::brokers::ctp::create_quick_ctp_routes()
    };

    // 合并所有路由
    let mut app = Router::new()
        .merge(unified_routes) // 挂载统一 API (无状态)
        .merge(app_routes) // 挂载主应用路由 (有状态)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // 挂载 CTP 路由
    #[cfg(feature = "ctp-real")]
    {
        app = app.merge(ctp_routes);
    }

    info!("starting server at {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
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
    // 从 query string 中提取 exchange_id
    let query_params: std::collections::HashMap<String, String> = uri
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();

    let exchange_id = query_params.get("exchange_id");

    // 如果有 exchange_id,返回模拟数据
    if let Some(exchange_id) = exchange_id {
        use crate::mock_data::generate_mock_data;

        if let Some(mock_data) = generate_mock_data(&path, exchange_id) {
            let json_body =
                serde_json::to_string(&mock_data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let first_segment = path.split('/').find(|s| !s.is_empty()).unwrap_or("");
            let cache_header = cache_control(first_segment);

            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
                .header(header::CACHE_CONTROL, cache_header.clone())
                .header("cdn-cache-control", cache_header)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(json_body))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // 否则走原来的代理逻辑
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

#[derive(serde::Serialize)]
struct BrokerInfo {
    id: String,
    name: String,
    broker_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    td_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    md_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_product_info: Option<String>,
    description: String,
    recommended: bool,
    category: String,
}

#[derive(serde::Serialize)]
struct ExchangeConfig {
    id: String,
    name: String,
    name_en: String,
    description: String,
    icon: String,
    color: String,
    enabled: bool,
    status: String,
    route: String,
    api_endpoint: String,
    features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brokers: Option<Vec<BrokerInfo>>,
}

#[derive(serde::Serialize)]
struct ExchangesConfigResponse {
    exchanges: Vec<ExchangeConfig>,
    settings: serde_json::Value,
}

async fn get_exchanges_config() -> impl IntoResponse {
    // 读取原始 YAML 配置文件
    let config_path = std::path::Path::new("config/exchanges.yaml");
    let config_str = match tokio::fs::read_to_string(config_path).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read config file: {:?}", e);
            // fallback 到旧的实现
            let config = BrokersConfig::load_default().unwrap_or_default();
            let exchanges: Vec<ExchangeConfig> = config
                .brokers
                .iter()
                .map(|broker| ExchangeConfig {
                    id: broker.id.clone(),
                    name: broker.name.clone(),
                    name_en: broker.name_en.clone(),
                    description: broker.description.clone(),
                    icon: broker.icon.clone(),
                    color: broker.color.clone(),
                    enabled: broker.enabled,
                    status: broker.status.clone(),
                    route: broker.route.clone(),
                    api_endpoint: broker.api_endpoint.clone(),
                    features: broker.features.clone(),
                    brokers: None,
                })
                .collect();

            return Json(ExchangesConfigResponse {
                exchanges,
                settings: serde_json::json!({
                    "defaultExchange": "crypto",
                    "refreshInterval": config.settings.refresh_interval * 1000,
                    "autoConnect": config.settings.auto_connect,
                }),
            });
        }
    };

    // 解析 YAML
    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&config_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse YAML: {:?}", e);
            return Json(ExchangesConfigResponse {
                exchanges: vec![],
                settings: serde_json::json!({}),
            });
        }
    }; // 提取 exchanges 数组
    let empty_vec = vec![];
    let exchanges_array = yaml_value
        .get("exchanges")
        .and_then(|v| v.as_sequence())
        .unwrap_or(&empty_vec);

    let mut exchanges = Vec::new();
    for exchange in exchanges_array {
        let id = exchange.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = exchange.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_en = exchange
            .get("name_en")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description = exchange
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let icon = exchange.get("icon").and_then(|v| v.as_str()).unwrap_or("");
        let color = exchange.get("color").and_then(|v| v.as_str()).unwrap_or("");
        let enabled = exchange
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let status = exchange
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("offline");
        let route = exchange.get("route").and_then(|v| v.as_str()).unwrap_or("");
        let api_endpoint = exchange
            .get("api_endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let features = exchange
            .get("features")
            .and_then(|v| v.as_sequence())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // 检查是否有 brokers 字段
        let brokers = exchange
            .get("brokers")
            .and_then(|v| v.as_sequence())
            .map(|brokers_array| {
                eprintln!("Found {} brokers for exchange {}", brokers_array.len(), id);
                brokers_array
                    .iter()
                    .filter_map(|broker| {
                        let broker_id = broker.get("name").and_then(|v| v.as_str())?;
                        let broker_name = broker.get("name").and_then(|v| v.as_str())?;
                        let broker_broker_id = broker.get("broker_id").and_then(|v| v.as_str())?;

                        Some(BrokerInfo {
                            id: broker_id.to_string(),
                            name: broker_name.to_string(),
                            broker_id: broker_broker_id.to_string(),
                            td_address: broker
                                .get("td_address")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            md_address: broker
                                .get("md_address")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            auth_code: broker
                                .get("auth_code")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            app_id: broker
                                .get("app_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            user_product_info: broker
                                .get("user_product_info")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            description: broker
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            recommended: broker
                                .get("recommended")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            category: broker
                                .get("category")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        })
                    })
                    .collect::<Vec<_>>()
            });

        exchanges.push(ExchangeConfig {
            id: id.to_string(),
            name: name.to_string(),
            name_en: name_en.to_string(),
            description: description.to_string(),
            icon: icon.to_string(),
            color: color.to_string(),
            enabled,
            status: status.to_string(),
            route: route.to_string(),
            api_endpoint: api_endpoint.to_string(),
            features,
            brokers,
        });
    }

    let response = ExchangesConfigResponse {
        exchanges,
        settings: serde_json::json!({
            "defaultExchange": "crypto",
            "refreshInterval": 30000,
            "autoConnect": false,
        }),
    };

    Json(response)
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
