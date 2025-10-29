// Release 模式下隐藏控制台窗口
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod engine;
mod error;
mod llm;
mod markets;
mod mcp;
mod server;
mod tray;

use std::net::SocketAddr;

use tracing::{info, Level};

use crate::server::run_http_server;
use crate::tray::run_system_tray;

fn main() -> anyhow::Result<()> {
    init_tracing();

    let addr: SocketAddr = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .map(|port| SocketAddr::from(([0, 0, 0, 0], port)))
        .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8788)));

    let url = format!("http://localhost:{}", addr.port());

    // 在独立线程中启动 HTTP 服务器
    let url_for_server = url.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async move { run_http_server(addr, url_for_server).await })
    });

    // 主线程运行系统托盘（Windows GUI 需要主线程）
    info!("Starting system tray on main thread");
    run_system_tray(url)?;
    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_max_level(Level::INFO)
        .try_init();
}
