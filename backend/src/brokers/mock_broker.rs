// Mock Broker - 模拟经纪商实现

use super::*;
use rand::Rng;
use std::collections::HashMap;

/// 模拟经纪商实现
#[allow(dead_code)]
pub struct MockBroker {
    id: String,
    name: String,
}

#[allow(dead_code)]
impl MockBroker {
    pub fn new() -> Self {
        Self {
            id: "mock".to_string(),
            name: "Mock Broker".to_string(),
        }
    }
}

impl Broker for MockBroker {
    fn broker_id(&self) -> &str {
        &self.id
    }

    fn broker_name(&self) -> &str {
        &self.name
    }
}

impl MarketData for MockBroker {
    fn get_prices(&self) -> impl std::future::Future<Output = Result<Prices, Box<dyn std::error::Error>>> + Send {
        async move {
            let mut prices = HashMap::new();
            prices.insert("BTC".to_string(), 50000.0);
            prices.insert("ETH".to_string(), 3000.0);
            Ok(Prices { prices })
        }
    }

    fn get_orderbook(&self, symbol: &str) -> impl std::future::Future<Output = Result<Orderbook, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        async move {
            let mut rng = rand::thread_rng();
            let base = 50000.0;
            let bids = (0..10).map(|i| OrderbookLevel { price: base * (1.0 - 0.0001 * (i + 1) as f64), quantity: rng.gen_range(0.1..10.0) }).collect();
            let asks = (0..10).map(|i| OrderbookLevel { price: base * (1.0 + 0.0001 * (i + 1) as f64), quantity: rng.gen_range(0.1..10.0) }).collect();
            Ok(Orderbook { symbol, bids, asks, timestamp: chrono::Utc::now().timestamp() })
        }
    }

    fn get_klines(&self, symbol: &str, interval: &str, limit: Option<i32>) -> impl std::future::Future<Output = Result<Klines, Box<dyn std::error::Error>>> + Send {
        let (symbol, interval) = (symbol.to_string(), interval.to_string());
        async move {
            let mut rng = rand::thread_rng();
            let klines = (0..limit.unwrap_or(100)).map(|_| Kline {
                timestamp: chrono::Utc::now().timestamp(),
                open: 50000.0, high: 50500.0, low: 49500.0, close: 50000.0,
                volume: rng.gen_range(1000.0..10000.0), open_interest: None
            }).collect();
            Ok(Klines { symbol, interval, klines })
        }
    }

    fn get_ticker_24h(&self, symbol: &str) -> impl std::future::Future<Output = Result<Ticker24h, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        async move {
            let mut rng = rand::thread_rng();
            Ok(Ticker24h { symbol, last_price: 50000.0, change_24h: rng.gen_range(-5.0..5.0), high_24h: 51500.0, low_24h: 48500.0, volume_24h: 1000000.0, open_interest: None, timestamp: chrono::Utc::now().timestamp() })
        }
    }
}

impl Trading for MockBroker {
    fn place_order(&self, _order: OrderRequest) -> impl std::future::Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {
        async move { Ok(OrderResponse { order_id: format!("ORDER_{}", chrono::Utc::now().timestamp_millis()), status: OrderStatus::Accepted, timestamp: chrono::Utc::now().timestamp() }) }
    }

    fn cancel_order(&self, order_id: &str) -> impl std::future::Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {
        let order_id = order_id.to_string();
        async move { Ok(OrderResponse { order_id, status: OrderStatus::Cancelled, timestamp: chrono::Utc::now().timestamp() }) }
    }

    fn get_order(&self, order_id: &str) -> impl std::future::Future<Output = Result<Order, Box<dyn std::error::Error>>> + Send {
        let order_id = order_id.to_string();
        async move { Ok(Order { order_id, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, order_type: OrderType::Limit, quantity: 0.1, filled_quantity: 0.1, price: Some(50000.0), avg_price: Some(50000.0), status: OrderStatus::Filled, created_at: chrono::Utc::now().timestamp(), updated_at: chrono::Utc::now().timestamp() }) }
    }

    fn get_orders(&self, _symbol: Option<&str>) -> impl std::future::Future<Output = Result<Orders, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Orders { orders: vec![] }) }
    }

    fn get_trades(&self) -> impl std::future::Future<Output = Result<Trades, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Trades { trades: vec![] }) }
    }
}

impl AccountManagement for MockBroker {
    fn get_account_totals(&self, _: Option<i32>) -> impl std::future::Future<Output = Result<AccountTotals, Box<dyn std::error::Error>>> + Send {
        async move { Ok(AccountTotals { account_totals: vec![] }) }
    }

    fn get_model_accounts(&self) -> impl std::future::Future<Output = Result<ModelAccounts, Box<dyn std::error::Error>>> + Send {
        async move { Ok(ModelAccounts { accounts: vec![] }) }
    }

    fn get_positions(&self, _: Option<i32>) -> impl std::future::Future<Output = Result<Positions, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Positions { positions: HashMap::new() }) }
    }

    fn get_balance(&self) -> impl std::future::Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Balance { total_balance: 105000.0, available: 50000.0, margin_used: None, frozen_margin: None, currency: "USDT".to_string(), timestamp: chrono::Utc::now().timestamp() }) }
    }

    fn get_broker_account(&self) -> impl std::future::Future<Output = Result<BrokerAccount, Box<dyn std::error::Error>>> + Send {
        async move { Ok(BrokerAccount { broker_id: "mock".to_string(), broker_name: "Mock".to_string(), broker_type: "mock".to_string(), protocol: None, timestamp: chrono::Utc::now().timestamp() }) }
    }
}

impl Analytics for MockBroker {
    fn get_analytics(&self) -> impl std::future::Future<Output = Result<AnalyticsData, Box<dyn std::error::Error>>> + Send {
        async move { Ok(AnalyticsData { metrics: HashMap::new() }) }
    }

    fn get_leaderboard(&self) -> impl std::future::Future<Output = Result<Leaderboard, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Leaderboard { leaderboard: vec![] }) }
    }

    fn get_since_inception_values(&self) -> impl std::future::Future<Output = Result<SinceInceptionValues, Box<dyn std::error::Error>>> + Send {
        async move { Ok(SinceInceptionValues { since_inception: vec![] }) }
    }

    fn get_conversations(&self) -> impl std::future::Future<Output = Result<Conversations, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Conversations { conversations: vec![] }) }
    }

    fn get_models_list(&self) -> impl std::future::Future<Output = Result<Models, Box<dyn std::error::Error>>> + Send {
        async move { Ok(Models { models: vec![] }) }
    }
}
