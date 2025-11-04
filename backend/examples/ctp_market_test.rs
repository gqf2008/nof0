/// CTP 行情接口测试程序
///
/// 用于测试是否能够连接到CTP服务器并接收行情数据
///
/// 运行方式:
/// ```bash
/// cargo run --example ctp_market_test
/// ```
///
/// 注意:
/// 1. 默认使用 SimNow 模拟环境
/// 2. 需要在 SimNow 注册账号: http://www.simnow.com.cn/
/// 3. 测试前请确保填写正确的账号信息
use nof0_backend::brokers::ctp::{CtpBroker, CtpConfig};
use nof0_backend::brokers::{Broker, MarketData};
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

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           CTP 行情接口测试程序                            ║");
    println!("║                                                           ║");
    println!("║  测试内容:                                                ║");
    println!("║  1. 连接到 CTP 行情服务器                                 ║");
    println!("║  2. 订阅期货合约行情                                      ║");
    println!("║  3. 接收实时行情数据                                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // ============================================================================
    // 1. 配置 CTP 连接参数
    // ============================================================================

    println!("📝 配置 CTP 连接参数...");

    // CTP 测试环境配置 (OpenCTP 公开测试环境)
    let config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "test".to_string(), // AppID认证模式下此字段仅用于验证通过
        password: "test".to_string(),    // AppID认证模式下此字段仅用于验证通过

        // OpenCTP 公开测试行情地址
        md_address: "tcp://trading.openctp.cn:30011".to_string(),

        // OpenCTP 公开测试交易地址
        td_address: "tcp://trading.openctp.cn:30011".to_string(),

        // 使用 AppID/AuthCode 认证 (真实认证凭证)
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
        user_product_info: "nof0_test".to_string(),
        mock_mode: false, // ✅ 启用真实CTP连接
    };

    match config.validate() {
        Ok(_) => println!("✅ 配置验证通过"),
        Err(e) => {
            eprintln!("❌ 配置验证失败: {}", e);
            return Ok(());
        }
    }

    println!("   经纪商ID: {}", config.broker_id);
    println!("   投资者ID: {}", config.investor_id);
    println!("   行情地址: {}", config.md_address);
    println!("   交易地址: {}", config.td_address);
    println!();

    // ============================================================================
    // 2. 创建 CTP Broker 实例
    // ============================================================================

    println!("🔌 创建 CTP Broker 实例...");
    let broker = CtpBroker::new("9999".to_string(), "CTP测试".to_string(), config.clone());

    println!("✅ Broker ID: {}", broker.broker_id());
    println!("✅ Broker Name: {}", broker.broker_name());
    println!();

    // ============================================================================
    // 2.5 连接到 CTP 服务器 (真实模式)
    // ============================================================================

    if !config.mock_mode {
        println!("🔗 正在连接 CTP 服务器...");
        match broker.connect().await {
            Ok(_) => println!("✅ CTP 服务器连接成功!"),
            Err(e) => {
                eprintln!("❌ CTP 服务器连接失败: {}", e);
                eprintln!("   将使用 mock 模式继续测试...");
            }
        }
        println!();

        // 订阅测试合约
        println!("📡 订阅行情数据...");
        let test_symbols_str: Vec<String> = vec![
            "IF2501".to_string(),
            "IC2501".to_string(),
            "IH2501".to_string(),
            "rb2505".to_string(),
            "au2504".to_string(),
        ];
        match broker.subscribe_market_data(test_symbols_str.clone()).await {
            Ok(_) => println!("✅ 已订阅 {} 个合约", test_symbols_str.len()),
            Err(e) => eprintln!("⚠️ 订阅失败: {}", e),
        }
        println!();

        // 等待行情数据到达
        println!("⏳ 等待行情数据...");
        sleep(Duration::from_secs(5)).await;
        println!();
    }

    // ============================================================================
    // 3. 获取行情数据
    // ============================================================================

    println!("📊 开始获取行情数据...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // 测试获取多个合约的价格
    let test_symbols = vec!["IF2501", "IC2501", "IH2501", "rb2505", "au2504"];

    for symbol in &test_symbols {
        print!("正在获取 {} 行情... ", symbol);

        match broker.get_ticker_24h(symbol).await {
            Ok(ticker) => {
                println!("✅");
                println!("  └─ 最新价: {:.2}", ticker.last_price);
                println!("  └─ 涨跌幅: {:.2}%", ticker.change_24h);
                println!("  └─ 最高价: {:.2}", ticker.high_24h);
                println!("  └─ 最低价: {:.2}", ticker.low_24h);
                println!("  └─ 成交量: {:.0}", ticker.volume_24h);
                if let Some(oi) = ticker.open_interest {
                    println!("  └─ 持仓量: {}", oi);
                }
                println!();
            }
            Err(e) => {
                println!("❌");
                println!("  └─ 错误: {}", e);
                println!();
            }
        }

        // 短暂延时，避免请求过快
        sleep(Duration::from_millis(500)).await;
    }

    // ============================================================================
    // 4. 获取深度行情
    // ============================================================================

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 获取深度行情 (Orderbook)...");
    println!();

    let symbol = "IF2501";
    print!("正在获取 {} 深度行情... ", symbol);

    match broker.get_orderbook(symbol).await {
        Ok(orderbook) => {
            println!("✅");
            println!();
            println!("  📗 买盘 (Bids):");
            for (i, level) in orderbook.bids.iter().enumerate() {
                println!(
                    "    {} - 价格: {:.2}  数量: {:.0}",
                    i + 1,
                    level.price,
                    level.quantity
                );
            }
            println!();
            println!("  📕 卖盘 (Asks):");
            for (i, level) in orderbook.asks.iter().enumerate() {
                println!(
                    "    {} - 价格: {:.2}  数量: {:.0}",
                    i + 1,
                    level.price,
                    level.quantity
                );
            }
            println!();
        }
        Err(e) => {
            println!("❌");
            println!("  └─ 错误: {}", e);
            println!();
        }
    }

    // ============================================================================
    // 5. 获取K线数据
    // ============================================================================

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 获取K线数据...");
    println!();

    let symbol = "IF2501";
    let intervals = vec!["1m", "5m", "1h"];

    for interval in &intervals {
        print!("正在获取 {} {} K线... ", symbol, interval);

        match broker.get_klines(symbol, interval, Some(5)).await {
            Ok(klines) => {
                println!("✅ (获取 {} 根K线)", klines.klines.len());

                if !klines.klines.is_empty() {
                    let latest = &klines.klines[klines.klines.len() - 1];
                    println!("  └─ 最新K线:");
                    println!("     开盘: {:.2}", latest.open);
                    println!("     最高: {:.2}", latest.high);
                    println!("     最低: {:.2}", latest.low);
                    println!("     收盘: {:.2}", latest.close);
                    println!("     成交量: {:.0}", latest.volume);
                    if let Some(oi) = latest.open_interest {
                        println!("     持仓量: {}", oi);
                    }
                }
                println!();
            }
            Err(e) => {
                println!("❌");
                println!("  └─ 错误: {}", e);
                println!();
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    // ============================================================================
    // 6. 批量获取所有价格
    // ============================================================================

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💰 批量获取所有合约价格...");
    println!();

    print!("正在获取... ");
    match broker.get_prices().await {
        Ok(prices) => {
            println!("✅ (获取 {} 个合约)", prices.prices.len());
            println!();

            for (symbol, price) in prices.prices.iter().take(10) {
                println!("  {} = {:.2}", symbol, price);
            }

            if prices.prices.len() > 10 {
                println!("  ... 还有 {} 个合约", prices.prices.len() - 10);
            }
            println!();
        }
        Err(e) => {
            println!("❌");
            println!("  └─ 错误: {}", e);
            println!();
        }
    }

    // ============================================================================
    // 7. 测试总结
    // ============================================================================

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ 测试完成!");
    println!();
    println!("📝 说明:");
    println!("   1. 以上数据为模拟数据（当前为mock模式）");
    println!("   2. 如需测试真实CTP连接，请:");
    println!("      - 设置 mock_mode = false");
    println!("      - 填写正确的 SimNow 账号信息");
    println!("      - 确保网络可以访问 SimNow 服务器");
    println!("   3. 真实环境需要依赖 CTP 的 C++ API");
    println!("   4. 可以考虑使用 openctp-client 或其他Rust CTP库");
    println!();
    println!("🔗 相关资源:");
    println!("   - SimNow官网: http://www.simnow.com.cn/");
    println!("   - CTP API文档: 上期技术官网");
    println!("   - openctp-client: https://crates.io/crates/openctp-client");
    println!();

    Ok(())
}
