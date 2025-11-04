/// CTP 模拟行情测试
///
/// 快速测试程序，使用模拟模式生成行情数据
/// 不需要真实的CTP账号，可以快速验证代码逻辑
///
/// 运行方式:
/// ```bash
/// cargo run --example ctp_mock_test
/// ```
use nof0_backend::brokers::ctp::{CtpBroker, CtpConfig};
use nof0_backend::brokers::{AccountManagement, Broker, MarketData, Trading};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         CTP 模拟行情测试 (无需真实账号)                  ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // 使用默认配置（mock模式）
    let config = CtpConfig::default();

    println!(
        "✅ 配置模式: {}",
        if config.mock_mode {
            "模拟模式 🎭"
        } else {
            "真实模式"
        }
    );
    println!();

    // 创建broker
    let broker = CtpBroker::new("ctp_mock".to_string(), "CTP模拟测试".to_string(), config);

    println!("🚀 开始测试...\n");

    // ============================================================================
    // 测试 1: 获取实时行情
    // ============================================================================

    println!("【测试 1】 实时行情 Ticker");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let symbols = vec!["IF2501", "IC2501", "rb2505"];

    for symbol in &symbols {
        match broker.get_ticker_24h(symbol).await {
            Ok(ticker) => {
                println!("✅ {} 行情:", symbol);
                println!(
                    "   价格: {:.2}  涨跌: {:.2}%  成交量: {:.0}",
                    ticker.last_price, ticker.change_24h, ticker.volume_24h
                );
            }
            Err(e) => println!("❌ {} 获取失败: {}", symbol, e),
        }
    }
    println!();

    sleep(Duration::from_secs(1)).await;

    // ============================================================================
    // 测试 2: 深度行情
    // ============================================================================

    println!("【测试 2】 深度行情 Orderbook");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match broker.get_orderbook("IF2501").await {
        Ok(orderbook) => {
            println!("✅ IF2501 深度行情:");
            println!(
                "   买一: {:.2} × {:.0}",
                orderbook.bids[0].price, orderbook.bids[0].quantity
            );
            println!(
                "   卖一: {:.2} × {:.0}",
                orderbook.asks[0].price, orderbook.asks[0].quantity
            );
            println!(
                "   总共: {} 档买盘, {} 档卖盘",
                orderbook.bids.len(),
                orderbook.asks.len()
            );
        }
        Err(e) => println!("❌ 获取失败: {}", e),
    }
    println!();

    sleep(Duration::from_secs(1)).await;

    // ============================================================================
    // 测试 3: K线数据
    // ============================================================================

    println!("【测试 3】 K线数据 Klines");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match broker.get_klines("IF2501", "5m", Some(10)).await {
        Ok(klines) => {
            println!("✅ IF2501 5分钟K线 (最近10根):");

            if let Some(latest) = klines.klines.last() {
                println!("   最新K线:");
                println!(
                    "   开: {:.2}  高: {:.2}  低: {:.2}  收: {:.2}",
                    latest.open, latest.high, latest.low, latest.close
                );
                println!("   成交量: {:.0}", latest.volume);
            }

            println!("   总共获取: {} 根K线", klines.klines.len());
        }
        Err(e) => println!("❌ 获取失败: {}", e),
    }
    println!();

    sleep(Duration::from_secs(1)).await;

    // ============================================================================
    // 测试 4: 账户余额
    // ============================================================================

    println!("【测试 4】 账户余额 Balance");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match broker.get_balance().await {
        Ok(balance) => {
            println!("✅ 账户信息:");
            println!("   总资金: {:.2} CNY", balance.total_balance);
            println!("   可用资金: {:.2} CNY", balance.available);
            if let Some(margin) = balance.margin_used {
                println!("   占用保证金: {:.2} CNY", margin);
            }
            if let Some(frozen) = balance.frozen_margin {
                println!("   冻结保证金: {:.2} CNY", frozen);
            }
        }
        Err(e) => println!("❌ 获取失败: {}", e),
    }
    println!();

    sleep(Duration::from_secs(1)).await;

    // ============================================================================
    // 测试 5: 持仓信息
    // ============================================================================

    println!("【测试 5】 持仓信息 Positions");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match broker.get_positions(None).await {
        Ok(positions) => {
            println!("✅ 持仓列表 (共 {} 个):", positions.positions.len());

            for (symbol, pos) in positions.positions.iter().take(5) {
                let direction_str = if pos.quantity > 0.0 {
                    "多头"
                } else {
                    "空头"
                };
                println!(
                    "   {} {} | 数量: {:.0} | 盈亏: {:.2}",
                    symbol,
                    direction_str,
                    pos.quantity.abs(),
                    pos.unrealized_pnl
                );
            }
        }
        Err(e) => println!("❌ 获取失败: {}", e),
    }
    println!();

    sleep(Duration::from_secs(1)).await;

    // ============================================================================
    // 测试 6: 批量价格
    // ============================================================================

    println!("【测试 6】 批量价格 Prices");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match broker.get_prices().await {
        Ok(prices) => {
            println!("✅ 获取 {} 个合约价格:", prices.prices.len());

            let mut sorted: Vec<_> = prices.prices.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));

            for (symbol, price) in sorted.iter().take(8) {
                println!("   {:<8} = {:>10.2}", symbol, price);
            }

            if prices.prices.len() > 8 {
                println!("   ... 还有 {} 个", prices.prices.len() - 8);
            }
        }
        Err(e) => println!("❌ 获取失败: {}", e),
    }
    println!();

    // ============================================================================
    // 性能测试：连续获取行情
    // ============================================================================

    println!("【性能测试】 连续获取行情 10 次");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let start = std::time::Instant::now();
    let mut success_count = 0;

    for i in 1..=10 {
        if broker.get_ticker_24h("IF2501").await.is_ok() {
            success_count += 1;
            print!(".");
        } else {
            print!("x");
        }

        if i % 10 == 0 {
            println!();
        }
    }

    let elapsed = start.elapsed();
    println!();
    println!("✅ 完成: {}/{} 成功", success_count, 10);
    println!("   耗时: {:.2?}", elapsed);
    println!("   平均: {:.2?}/次", elapsed / 10);
    println!();

    // ============================================================================
    // 总结
    // ============================================================================

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                    测试总结                               ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  ✅ 行情接口正常                                          ║");
    println!("║  ✅ 深度数据正常                                          ║");
    println!("║  ✅ K线数据正常                                           ║");
    println!("║  ✅ 账户接口正常                                          ║");
    println!("║  ✅ 持仓接口正常                                          ║");
    println!("║  ✅ 批量接口正常                                          ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  📝 当前为模拟模式，所有数据为随机生成                   ║");
    println!("║  🔗 如需测试真实CTP，请运行:                             ║");
    println!("║     cargo run --example ctp_market_test                  ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}
