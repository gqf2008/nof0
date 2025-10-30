/// 经纪商使用示例
///
/// 展示如何使用 BrokerRegistry 和 BrokerInstance enum
///
/// 运行方式:
/// ```
/// cargo run --example broker_usage
/// ```
use nof0_backend::brokers::{
    Analytics, BrokerInstance, BrokerRegistry, MarketData, MockBroker, Trading,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 BrokerRegistry
    let mut registry = BrokerRegistry::new();

    // 2. 创建 Mock Broker
    let mock_broker = MockBroker::new();

    // 3. 将 Mock Broker 包装为 BrokerInstance 并注册
    registry.register(BrokerInstance::Mock(mock_broker));

    // 4. 从注册表获取 broker
    let broker = registry
        .get("mock")
        .expect("Mock broker should be registered");

    // 5. 使用 MarketData trait
    println!("=== 获取价格 ===");
    let prices = broker.get_prices().await?;
    println!("BTC 价格: ${}", prices.btc.price);
    println!("ETH 价格: ${}", prices.eth.price);

    // 6. 使用 Trading trait - 下单
    println!("\n=== 下单 ===");
    use nof0_backend::brokers::{OrderRequest, OrderSide, OrderType};
    let order_request = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: 0.1,
        price: Some(50000.0),
        time_in_force: Some("GTC".to_string()),
        client_order_id: Some("test-order-001".to_string()),
    };

    let order_response = broker.place_order(order_request).await?;
    println!("订单ID: {}", order_response.order_id);
    println!("订单状态: {:?}", order_response.status);

    // 7. 使用 Analytics trait - 获取排行榜
    println!("\n=== 排行榜 ===");
    let leaderboard = broker.get_leaderboard().await?;
    for entry in leaderboard.entries.iter().take(3) {
        println!(
            "排名 {}: {} - 总收益率: {:.2}%",
            entry.rank, entry.model_name, entry.total_return_pct
        );
    }

    // 8. 列出所有注册的 broker
    println!("\n=== 所有经纪商 ===");
    for broker_id in registry.list_ids() {
        println!("- {}", broker_id);
    }

    Ok(())
}
