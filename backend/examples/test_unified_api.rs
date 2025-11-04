use axum::{routing::get, Router};
/// 统一 API 测试示例
///
/// 运行: cargo run --example test_unified_api
use nof0_backend::api::{adapters::CtpMockAdapter, create_unified_routes, ExchangeManager};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 统一 API 测试示例启动中...");

    // 1. 创建交易所管理器
    let mut exchange_manager = ExchangeManager::new();

    // 2. 注册 CTP Mock 适配器
    let ctp_adapter = Arc::new(CtpMockAdapter::new("ctp", "CTP期货"));
    exchange_manager.register(ctp_adapter);

    println!("✅ 注册交易所: CTP期货");

    let exchange_manager = Arc::new(exchange_manager);

    // 3. 创建统一 API 路由
    let unified_routes = create_unified_routes(exchange_manager.clone());

    // 4. 创建完整应用
    let app = Router::new()
        .merge(unified_routes)
        .route("/health", get(|| async { "OK" }));

    let addr: std::net::SocketAddr = "127.0.0.1:8788".parse()?;
    println!("🌐 服务器启动: http://{}", addr);
    println!("\n📋 API 端点测试:");
    println!("  GET  http://localhost:8788/api/v1/exchanges");
    println!("  GET  http://localhost:8788/api/v1/exchanges/ctp/status");
    println!("  POST http://localhost:8788/api/v1/exchanges/ctp/connect");
    println!("  GET  http://localhost:8788/api/v1/exchanges/ctp/market/ticker/rb2501");
    println!("  GET  http://localhost:8788/api/v1/exchanges/ctp/account");
    println!("\n按 Ctrl+C 停止服务器\n");

    // 5. 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
