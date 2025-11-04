/// CTP 状态检查工具
///
/// 快速检查 CTP 连接和数据接收状态
use nof0_backend::brokers::ctp::{CtpBroker, CtpConfig};
use nof0_backend::brokers::{Broker, MarketData};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║           CTP 状态检查                                    ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Mock 模式配置
    let config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "test".to_string(),
        password: "test".to_string(),
        md_address: "tcp://trading.openctp.cn:30011".to_string(),
        td_address: "tcp://trading.openctp.cn:30011".to_string(),
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
        user_product_info: "nof0".to_string(),
        mock_mode: true, // 使用 Mock 模式测试
    };

    println!("📋 配置信息:");
    println!(
        "   模式: {}",
        if config.mock_mode {
            "Mock 模式 🎭"
        } else {
            "Real 模式 🔌"
        }
    );
    println!("   Broker ID: {}", config.broker_id);
    println!("   行情地址: {}", config.md_address);
    println!();

    // 创建 Broker
    println!("🔨 创建 Broker...");
    let broker = CtpBroker::new("ctp_check".to_string(), "CTP Check".to_string(), config);
    println!("✅ Broker 创建成功\n");

    // 测试 1: 快速响应测试
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 1: 响应速度测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let start = Instant::now();
    match broker.get_ticker_24h("IF2501").await {
        Ok(ticker) => {
            let elapsed = start.elapsed();
            println!("✅ 数据获取成功");
            println!("   合约: IF2501");
            println!("   最新价: {:.2}", ticker.last_price);
            println!("   响应时间: {:?}", elapsed);

            if elapsed.as_millis() < 100 {
                println!("   ✅ 响应速度: 优秀");
            } else if elapsed.as_millis() < 500 {
                println!("   ✅ 响应速度: 良好");
            } else {
                println!("   ⚠️ 响应速度: 较慢");
            }
        }
        Err(e) => {
            println!("❌ 数据获取失败: {}", e);
        }
    }

    sleep(Duration::from_secs(1)).await;

    // 测试 2: 多合约测试
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 2: 多合约数据完整性");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let test_symbols = vec!["IF2501", "IC2501", "IH2501", "rb2505", "au2504"];
    let mut success_count = 0;
    let mut total_count = test_symbols.len();

    for symbol in &test_symbols {
        match broker.get_ticker_24h(symbol).await {
            Ok(ticker) => {
                success_count += 1;
                println!(
                    "✅ {} - {:.2} (涨跌: {:.2}%)",
                    symbol, ticker.last_price, ticker.change_24h
                );
            }
            Err(e) => {
                println!("❌ {} - 失败: {}", symbol, e);
            }
        }
        sleep(Duration::from_millis(200)).await;
    }

    println!(
        "\n结果: {}/{} 成功 ({:.1}%)",
        success_count,
        total_count,
        (success_count as f64 / total_count as f64) * 100.0
    );

    sleep(Duration::from_secs(1)).await;

    // 测试 3: 深度行情测试
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 3: 深度行情数据");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match broker.get_orderbook("IF2501").await {
        Ok(orderbook) => {
            println!("✅ 深度数据获取成功");
            println!("   买盘档位: {}", orderbook.bids.len());
            println!("   卖盘档位: {}", orderbook.asks.len());

            if !orderbook.bids.is_empty() {
                println!(
                    "   最优买价: {:.2} x {:.0}",
                    orderbook.bids[0].price, orderbook.bids[0].quantity
                );
            }
            if !orderbook.asks.is_empty() {
                println!(
                    "   最优卖价: {:.2} x {:.0}",
                    orderbook.asks[0].price, orderbook.asks[0].quantity
                );
            }
        }
        Err(e) => {
            println!("❌ 深度数据获取失败: {}", e);
        }
    }

    sleep(Duration::from_secs(1)).await;

    // 测试 4: K线数据测试
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 4: K线数据");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match broker.get_klines("IF2501", "1m", Some(5)).await {
        Ok(klines) => {
            println!("✅ K线数据获取成功");
            println!("   K线数量: {} 根", klines.klines.len());

            if !klines.klines.is_empty() {
                let latest = &klines.klines[klines.klines.len() - 1];
                println!(
                    "   最新K线: O:{:.2} H:{:.2} L:{:.2} C:{:.2}",
                    latest.open, latest.high, latest.low, latest.close
                );
            }
        }
        Err(e) => {
            println!("❌ K线数据获取失败: {}", e);
        }
    }

    // 总结
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 检查总结");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if success_count == total_count {
        println!("✅ 状态: 优秀");
        println!("   所有测试通过，数据接收正常");
    } else if success_count > 0 {
        println!("⚠️ 状态: 部分成功");
        println!("   部分数据接收正常，请检查失败的合约");
    } else {
        println!("❌ 状态: 失败");
        println!("   未能获取任何数据");
    }

    println!("\n💡 提示:");
    println!("   - Mock 模式: 使用模拟数据，无需外部服务");
    println!("   - Real 模式: 需要连接 CTP 服务器并订阅合约");
    println!("   - 查看日志: 使用 RUST_LOG=debug 环境变量查看详细日志\n");

    Ok(())
}
