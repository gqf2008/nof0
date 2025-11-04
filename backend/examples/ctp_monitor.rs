/// CTP 数据接收监控工具
///
/// 用于检查程序运行时是否正在接收 CTP 行情数据
use nof0_backend::brokers::ctp::{CtpBroker, CtpConfig};
use nof0_backend::brokers::{Broker, MarketData};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

static DATA_COUNT: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志 - 显示所有级别
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // 显示 DEBUG 级别的行情数据
        .with_target(true)
        .init();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           CTP 数据接收监控                                ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // 检测 Mock 或 Real 模式
    let use_mock = std::env::args().any(|arg| arg == "--mock");

    let config = if use_mock {
        println!("🎭 使用 Mock 模式");
        CtpConfig {
            broker_id: "9999".to_string(),
            investor_id: "test".to_string(),
            password: "test".to_string(),
            md_address: "tcp://trading.openctp.cn:30011".to_string(),
            td_address: "tcp://trading.openctp.cn:30011".to_string(),
            app_id: "simnow_client_test".to_string(),
            auth_code: "0000000000000000".to_string(),
            user_product_info: "nof0".to_string(),
            mock_mode: true,
        }
    } else {
        println!("🔌 使用真实 CTP 连接");
        CtpConfig {
            broker_id: "9999".to_string(),
            investor_id: "test".to_string(),
            password: "test".to_string(),
            md_address: "tcp://trading.openctp.cn:30011".to_string(),
            td_address: "tcp://trading.openctp.cn:30011".to_string(),
            app_id: "simnow_client_test".to_string(),
            auth_code: "0000000000000000".to_string(),
            user_product_info: "nof0".to_string(),
            mock_mode: false,
        }
    };

    let broker = Arc::new(CtpBroker::new(
        "ctp_monitor".to_string(),
        "CTP Monitor".to_string(),
        config.clone(),
    ));

    // 如果是真实模式，连接并订阅
    if !use_mock {
        println!("\n📡 连接到 CTP 服务器...");
        match broker.connect().await {
            Ok(_) => {
                println!("✅ 连接成功");

                // 订阅测试合约
                println!("\n📋 订阅合约...");
                let symbols = vec![
                    "IF2501".to_string(),
                    "IC2501".to_string(),
                    "rb2505".to_string(),
                ];

                match broker.subscribe_market_data(symbols.clone()).await {
                    Ok(_) => println!("✅ 订阅请求已发送: {:?}", symbols),
                    Err(e) => println!("⚠️ 订阅失败: {}", e),
                }

                println!("\n⏳ 等待数据接收...");
                sleep(Duration::from_secs(3)).await;
            }
            Err(e) => {
                println!("❌ 连接失败: {}", e);
                println!("💡 提示: 使用 --mock 参数运行 Mock 模式");
                return Ok(());
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 开始监控数据接收...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 启动统计任务
    let stats_broker = broker.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        let mut last_count = 0u64;
        let mut check_count = 0u32;

        loop {
            interval.tick().await;
            check_count += 1;

            let current = DATA_COUNT.load(Ordering::Relaxed);
            let delta = current.saturating_sub(last_count);

            println!("\n📈 统计 #{} (过去 5 秒)", check_count);
            println!("   数据接收: {} 条", delta);
            println!("   累计接收: {} 条", current);

            if delta == 0 {
                println!("   ⚠️ 未接收到新数据");
            } else {
                println!("   ✅ 正在接收数据 ({} 条/5秒)", delta);
            }

            last_count = current;
        }
    });

    // 主循环：定期获取行情测试
    let mut test_interval = interval(Duration::from_secs(10));
    let test_symbols = vec!["IF2501", "IC2501", "IH2501"];

    loop {
        test_interval.tick().await;

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔍 测试行情获取...\n");

        for symbol in &test_symbols {
            match broker.get_ticker_24h(symbol).await {
                Ok(ticker) => {
                    println!(
                        "✅ {} - 价格: {:.2}, 成交量: {:.0}",
                        symbol, ticker.last_price, ticker.volume_24h
                    );
                    DATA_COUNT.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    println!("❌ {} - 错误: {}", symbol, e);
                }
            }
        }
    }
}
