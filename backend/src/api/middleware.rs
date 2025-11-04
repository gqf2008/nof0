/// API 中间件
///
/// 提供请求日志、鉴权、限流等功能
use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

/// 请求日志中间件
pub async fn logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    info!("📥 {} {}", method, uri);

    let response = next.run(req).await;

    let status = response.status();
    let log_msg = if status.is_success() {
        format!("✅ {} {} -> {}", method, uri, status)
    } else {
        format!("❌ {} {} -> {}", method, uri, status)
    };

    info!("{}", log_msg);

    response
}

/// CORS 中间件(已在 server.rs 中处理,这里保留接口)
pub async fn cors_middleware(req: Request, next: Next) -> Response {
    next.run(req).await
}

/// 简单的请求追踪ID中间件
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    use uuid::Uuid;

    let request_id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(request_id.clone());

    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("x-request-id", request_id.parse().unwrap());

    response
}
