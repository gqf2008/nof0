/// 三个交易所演示示例
///
/// 展示如何同时使用 Binance (币安), OKEX, 和 CTP (期货) 三个经纪商
///
/// 运行方式:
/// ```
/// cargo run --example three_brokers_demo
/// ```

use nof0_backend::brokers::{
    AccountManagement, Analytics, BinanceBroker, Broker, BrokerInstance, BrokerRegistry, CtpBroker,
    MarketData, OkexBroker, Trading,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NOF0 三大交易所演示 ===\n");

    // 1. 创建 BrokerRegistry
    let mut registry = BrokerRegistry::new();

    // 2. 创建并注册 Binance (币安)
    let binance_config = nof0_backend::brokers::binance::BinanceConfig::default();
    let binance = BinanceBroker::new(
        "binance".to_string(),
        "Binance (币安)".to_string(),
        binance_config,
    );
    registry.register(BrokerInstance::Binance(binance));
    println!("✅ Binance (币安) 已注册");

    // 3. 创建并注册 OKEX
    let okex_config = nof0_backend::brokers::okex::OkexConfig::default();
    let okex = OkexBroker::new("okex".to_string(), "OKEX".to_string(), okex_config);
    registry.register(BrokerInstance::Okex(okex));
    println!("✅ OKEX 已注册");

    // 4. 创建并注册 CTP (中国期货)
    let ctp_config = nof0_backend::brokers::ctp::CtpConfig::default();
    let ctp = CtpBroker::new("ctp".to_string(), "CTP (中国期货)".to_string(), ctp_config);
    registry.register(BrokerInstance::Ctp(ctp));
    println!("✅ CTP (中国期货) 已注册");

    println!("\n已注册的交易所: {:?}\n", registry.list_ids());

    // 5. 演示 Binance
    println!("=== Binance (币安) 行情 ===");
    let binance_broker = registry.get("binance").expect("Binance broker not found");
    demonstrate_broker(binance_broker, "BTCUSDT").await?;

    // 6. 演示 OKEX
    println!("\n=== OKEX 行情 ===");
    let okex_broker = registry.get("okex").expect("OKEX broker not found");
    demonstrate_broker(okex_broker, "BTC-USDT").await?;

    // 7. 演示 CTP
    println!("\n=== CTP (中国期货) 行情 ===");
    let ctp_broker = registry.get("ctp").expect("CTP broker not found");
    demonstrate_broker(ctp_broker, "IF2501").await?;

    // 8. 比较三个交易所的数据
    println!("\n=== 三大交易所对比 ===");
    compare_brokers(&registry).await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示单个交易所的功能
async fn demonstrate_broker(
    broker: &BrokerInstance,
    symbol: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 获取价格
    let prices = broker.get_prices().await?;
    println!("📊 价格数据 (前3个):");
    for (sym, price) in prices.prices.iter().take(3) {
        println!("  {}: ${:.2}", sym, price);
    }

    // 获取24小时行情
    let ticker = broker.get_ticker_24h(symbol).await?;
    println!("\n📈 {} 24小时行情:", symbol);
    println!("  最新价: ${:.2}", ticker.last_price);
    println!("  24h涨跌: {:.2}%", ticker.change_24h);
    println!("  24h最高: ${:.2}", ticker.high_24h);
    println!("  24h最低: ${:.2}", ticker.low_24h);
    println!("  24h成交量: {:.0}", ticker.volume_24h);
    if let Some(oi) = ticker.open_interest {
        println!("  持仓量: {}", oi);
    }

    // 获取订单簿
    let orderbook = broker.get_orderbook(symbol).await?;
    println!("\n📋 {} 订单簿 (前3档):", symbol);
    println!("  买盘:");
    for (i, bid) in orderbook.bids.iter().take(3).enumerate() {
        println!("    [{}] 价格: ${:.2}, 数量: {:.2}", i + 1, bid.price, bid.quantity);
    }
    println!("  卖盘:");
    for (i, ask) in orderbook.asks.iter().take(3).enumerate() {
        println!("    [{}] 价格: ${:.2}, 数量: {:.2}", i + 1, ask.price, ask.quantity);
    }

    Ok(())
}

/// 比较三个交易所
async fn compare_brokers(registry: &BrokerRegistry) -> Result<(), Box<dyn std::error::Error>> {
    println!("交易所          | AI模型数 | 总成交量24h      | 平均交易次数");
    println!("---------------|--------|-----------------|------------");

    for broker_id in registry.list_ids() {
        if let Some(broker) = registry.get(&broker_id) {
            let analytics = broker.get_analytics().await?;
            let models = analytics.metrics.get("total_models").unwrap_or(&0.0);
            let volume = analytics.metrics.get("total_volume_24h").unwrap_or(&0.0);
            let avg_trades = analytics
                .metrics
                .get("avg_trades_per_model")
                .unwrap_or(&0.0);

            println!(
                "{:<15} | {:>6.0} | ${:>14.0} | {:>10.0}",
                broker.broker_name(),
                models,
                volume,
                avg_trades
            );
        }
    }

    // 获取每个交易所的模型账户摘要
    println!("\n=== AI 交易模型表现 ===");
    for broker_id in registry.list_ids() {
        if let Some(broker) = registry.get(&broker_id) {
            println!("\n{} 的 AI 模型:", broker.broker_name());
            let model_accounts = broker.get_model_accounts().await?;
            for account in model_accounts.accounts.iter().take(2) {
                // 只显示前2个
                println!(
                    "  {} - 收益: {:.2}% | 胜率: {:.1}% | 交易: {}次",
                    account.model_name,
                    account.return_pct,
                    account.win_rate * 100.0,
                    account.total_trades
                );
            }
        }
    }

    Ok(())
}
