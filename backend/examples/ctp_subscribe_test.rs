/// 简化的 CTP 订阅测试
/// 专注于测试订阅和接收行情数据
use nof0_backend::brokers::ctp::{CtpBroker, CtpConfig};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .init();

    println!("🚀 CTP 订阅测试");
    println!("════════════════════════════════════════");

    // OpenCTP 配置
    let config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "test".to_string(),
        password: "test".to_string(),
        md_address: "tcp://trading.openctp.cn:30011".to_string(),
        td_address: "tcp://trading.openctp.cn:30011".to_string(),
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
        user_product_info: "nof0_test".to_string(),
        mock_mode: false,
    };

    println!("📝 配置: {}", config.md_address);

    // 创建 broker
    let broker = CtpBroker::new("ctp_test".to_string(), "CTP测试".to_string(), config);

    // 连接
    println!("\n🔌 连接中...");
    broker.connect().await?;
    println!("✅ 连接成功");

    // 订阅 - 尝试 OpenCTP 支持的测试合约
    // OpenCTP 通常支持: rb2505 (螺纹钢), au2506 (黄金), cu2505 (铜) 等商品期货
    println!("\n📡 订阅合约: rb2505 (螺纹钢), au2506 (黄金), cu2505 (铜)");
    let symbols = vec![
        "rb2505".to_string(),
        "au2506".to_string(),
        "cu2505".to_string(),
    ];
    broker.subscribe_market_data(symbols).await?;
    println!("✅ 订阅请求已发送");

    // 等待数据
    println!("\n⏳ 等待 10 秒接收数据...");
    sleep(Duration::from_secs(10)).await;

    println!("\n✅ 测试完成 (查看日志中的行情数据回调)");
    Ok(())
}
