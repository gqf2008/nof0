/// CTP 集成示例 - 使用已验证的 OpenCTP 配置
///
/// 此示例展示如何在项目中集成 CTP，支持 Mock 和真实模式无缝切换
use nof0_backend::brokers::ctp::{CtpBroker, CtpConfig};
use nof0_backend::brokers::{Broker, MarketData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 CTP 集成示例");
    println!("════════════════════════════════════════\n");

    // ============================================================================
    // 配置 1: Mock 模式（推荐用于开发/演示）
    // ============================================================================
    println!("📋 配置 Mock 模式...");
    let mock_config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "test".to_string(),
        password: "test".to_string(),
        md_address: "tcp://trading.openctp.cn:30011".to_string(),
        td_address: "tcp://trading.openctp.cn:30011".to_string(),
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
        user_product_info: "nof0".to_string(),
        mock_mode: true, // ✅ Mock 模式：零依赖，稳定可靠
    };

    let mock_broker = CtpBroker::new("ctp_mock".to_string(), "CTP Mock".to_string(), mock_config);

    println!("✅ Mock Broker 创建成功");
    println!("   ID: {}", mock_broker.broker_id());
    println!("   Name: {}", mock_broker.broker_name());

    // 测试 Mock 数据
    println!("\n📊 获取 Mock 行情数据...");
    match mock_broker.get_ticker_24h("IF2501").await {
        Ok(ticker) => {
            println!("✅ IF2501 最新价: {:.2}", ticker.last_price);
            println!("   涨跌幅: {:.2}%", ticker.change_24h);
            println!("   成交量: {:.0}", ticker.volume_24h);
        }
        Err(e) => println!("❌ 错误: {}", e),
    }

    // ============================================================================
    // 配置 2: 真实模式（需要 SimNow 账号或实盘账号）
    // ============================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 配置真实模式...");

    let real_config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "test".to_string(), // ⚠️ 真实使用时替换为 SimNow 账号
        password: "test".to_string(),    // ⚠️ 真实使用时替换为 SimNow 密码
        md_address: "tcp://trading.openctp.cn:30011".to_string(),
        td_address: "tcp://trading.openctp.cn:30011".to_string(),
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
        user_product_info: "nof0".to_string(),
        mock_mode: false, // ⚠️ 真实模式：需要有效账号
    };

    let real_broker = CtpBroker::new(
        "ctp_real".to_string(),
        "CTP Real".to_string(),
        real_config.clone(),
    );

    println!("✅ Real Broker 创建成功");
    println!("   ID: {}", real_broker.broker_id());
    println!("   配置: {}", real_config.md_address);

    // 连接测试（仅测试连接，不订阅行情）
    println!("\n🔌 测试真实连接...");
    match real_broker.connect().await {
        Ok(_) => {
            println!("✅ CTP 服务器连接成功");
            println!("   MD 服务器: ✅");
            println!("   TD 服务器: ✅");
            println!("   认证状态: ✅");
            println!("\n⚠️  注意: 行情订阅需要 SimNow 有效账号");
        }
        Err(e) => {
            println!("⚠️  连接失败: {}", e);
            println!("   这是正常的，因为使用的是测试账号");
        }
    }

    // ============================================================================
    // 推荐使用方式
    // ============================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 推荐使用方式:\n");
    println!("   📌 开发/演示阶段:");
    println!("      mock_mode: true");
    println!("      - 零依赖");
    println!("      - 稳定可靠");
    println!("      - 适合功能开发和演示\n");

    println!("   📌 测试阶段:");
    println!("      mock_mode: false");
    println!("      - 注册 SimNow 账号");
    println!("      - 测试真实行情");
    println!("      - 验证订单流程\n");

    println!("   📌 生产阶段:");
    println!("      mock_mode: false");
    println!("      - 使用实盘账号");
    println!("      - 期货公司 CTP 地址");

    println!("\n✅ 集成完成!");
    Ok(())
}
