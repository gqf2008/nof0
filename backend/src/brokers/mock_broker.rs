// Mock Broker - 简化版本实现所有 traituse super::{use super::{

use super::*;

use rand::Rng;    AccountManagement, AccountTotal, AccountTotals, Analytics, AnalyticsData, Balance, Broker,    AccountManagement, AccountTotal, AccountTotals, Analytics, AnalyticsData, Balance, Broker,

use std::collections::HashMap;

    BrokerAccount, Conversations, Kline, Klines, Leaderboard, LeaderboardEntry, MarketData,    BrokerAccount, Conversations, Kline, Klines, Leaderboard, LeaderboardEntry, MarketData,

pub struct MockBroker {

    id: String,    ModelAccount, ModelAccounts, Models, ModelInfo, Order, OrderRequest, OrderResponse, Orders,    ModelAccount, ModelAccounts, Models, Order, OrderRequest, OrderResponse, Orders, Orderbook,

    name: String,

}    Orderbook, OrderbookLevel, OrderSide, OrderStatus, OrderType, Position, Positions, Prices,    OrderbookLevel, Position, Positions, Prices, SinceInceptionValues, Ticker24h, Trade, Trades,



impl MockBroker {    SinceInceptionValues, Ticker24h, Trade, Trades, Trading,    Trading,

    pub fn new() -> Self {

        Self {};};

            id: "mock".to_string(),

            name: "Mock Broker".to_string(),use rand::Rng;use rand::Rng;

        }

    }use std::collections::HashMap;use serde_json::json;

}

use std::future::Future;use std::collections::HashMap;

impl Broker for MockBroker {

    fn broker_id(&self) -> &str { &self.id }use std::future::Future;

    fn broker_name(&self) -> &str { &self.name }

}/// 模拟经纪商实现



impl MarketData for MockBroker {pub struct MockBroker {/// 模拟经纪商实现

    fn get_prices(&self) -> impl std::future::Future<Output = Result<Prices, Box<dyn std::error::Error>>> + Send {

        async move {    id: String,#[allow(dead_code)]

            let mut prices = HashMap::new();

            prices.insert("BTC".to_string(), 50000.0);    name: String,pub struct MockBroker {

            prices.insert("ETH".to_string(), 3000.0);

            Ok(Prices { prices })}    id: String,

        }

    }    name: String,



    fn get_orderbook(&self, symbol: &str) -> impl std::future::Future<Output = Result<Orderbook, Box<dyn std::error::Error>>> + Send {impl MockBroker {    broker_type: BrokerType,

        let symbol = symbol.to_string();

        async move {    pub fn new() -> Self {}

            let mut rng = rand::thread_rng();

            let base = 50000.0;        Self {

            let bids = (0..10).map(|i| OrderbookLevel { price: base * (1.0 - 0.0001 * (i + 1) as f64), quantity: rng.gen_range(0.1..10.0) }).collect();

            let asks = (0..10).map(|i| OrderbookLevel { price: base * (1.0 + 0.0001 * (i + 1) as f64), quantity: rng.gen_range(0.1..10.0) }).collect();            id: "mock".to_string(),#[derive(Clone)]

            Ok(Orderbook { symbol, bids, asks, timestamp: chrono::Utc::now().timestamp() })

        }            name: "Mock Broker".to_string(),#[allow(dead_code)]

    }

        }pub enum BrokerType {

    fn get_klines(&self, symbol: &str, interval: &str, limit: Option<i32>) -> impl std::future::Future<Output = Result<Klines, Box<dyn std::error::Error>>> + Send {

        let (symbol, interval) = (symbol.to_string(), interval.to_string());    }    Crypto,

        async move {

            let mut rng = rand::thread_rng();}    Futures,

            let klines = (0..limit.unwrap_or(100)).map(|_| Kline {

                timestamp: chrono::Utc::now().timestamp(),    Stock,

                open: 50000.0, high: 50500.0, low: 49500.0, close: 50000.0,

                volume: rng.gen_range(1000.0..10000.0), open_interest: Noneimpl Broker for MockBroker {}

            }).collect();

            Ok(Klines { symbol, interval, klines })    fn broker_id(&self) -> &str {

        }

    }        &self.id#[allow(dead_code)]



    fn get_ticker_24h(&self, symbol: &str) -> impl std::future::Future<Output = Result<Ticker24h, Box<dyn std::error::Error>>> + Send {    }impl MockBroker {

        let symbol = symbol.to_string();

        async move {    pub fn new(id: String, name: String, broker_type: BrokerType) -> Self {

            let mut rng = rand::thread_rng();

            Ok(Ticker24h { symbol, last_price: 50000.0, change_24h: rng.gen_range(-5.0..5.0), high_24h: 51500.0, low_24h: 48500.0, volume_24h: 1000000.0, open_interest: None, timestamp: chrono::Utc::now().timestamp() })    fn broker_name(&self) -> &str {        Self {

        }

    }        &self.name            id,

}

    }            name,

impl Trading for MockBroker {

    fn place_order(&self, _order: OrderRequest) -> impl std::future::Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {}            broker_type,

        async move { Ok(OrderResponse { order_id: format!("ORDER_{}", chrono::Utc::now().timestamp_millis()), status: OrderStatus::Accepted, timestamp: chrono::Utc::now().timestamp() }) }

    }        }



    fn cancel_order(&self, order_id: &str) -> impl std::future::Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {impl MarketData for MockBroker {    }

        let order_id = order_id.to_string();

        async move { Ok(OrderResponse { order_id, status: OrderStatus::Cancelled, timestamp: chrono::Utc::now().timestamp() }) }    fn get_prices(&self) -> impl Future<Output = Result<Prices, Box<dyn std::error::Error>>> + Send {

    }

        async move {    /// 获取该经纪商的 AI 模型配置

    fn get_order(&self, order_id: &str) -> impl std::future::Future<Output = Result<Order, Box<dyn std::error::Error>>> + Send {

        let order_id = order_id.to_string();            let mut prices = HashMap::new();    fn get_models(&self) -> Vec<ModelInfo> {

        async move { Ok(Order { order_id, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, order_type: OrderType::Limit, quantity: 0.1, filled_quantity: 0.1, price: Some(50000.0), avg_price: Some(50000.0), status: OrderStatus::Filled, created_at: chrono::Utc::now().timestamp(), updated_at: chrono::Utc::now().timestamp() }) }

    }            prices.insert("BTC".to_string(), 50000.0);        match self.id.as_str() {



    fn get_orders(&self, _symbol: Option<&str>) -> impl std::future::Future<Output = Result<Orders, Box<dyn std::error::Error>>> + Send {            prices.insert("ETH".to_string(), 3000.0);            "crypto" => vec![

        async move { Ok(Orders { orders: vec![] }) }

    }            Ok(Prices { prices })                ModelInfo {



    fn get_trades(&self) -> impl std::future::Future<Output = Result<Trades, Box<dyn std::error::Error>>> + Send {        }                    id: "crypto_gpt4".to_string(),

        async move { Ok(Trades { trades: vec![] }) }

    }    }                    name: "GPT-4 趋势追踪".to_string(),

}

                    strategy: "趋势跟踪".to_string(),

impl AccountManagement for MockBroker {

    fn get_account_totals(&self, _: Option<i32>) -> impl std::future::Future<Output = Result<AccountTotals, Box<dyn std::error::Error>>> + Send {    fn get_orderbook(&self, symbol: &str) -> impl Future<Output = Result<Orderbook, Box<dyn std::error::Error>>> + Send {                    description: "使用GPT-4分析市场趋势，捕捉中长期方向".to_string(),

        async move { Ok(AccountTotals { accounts: vec![AccountTotal { model_id: 1, model_name: "GPT-4".to_string(), broker_name: "mock".to_string(), broker_id: "mock".to_string(), timestamp: chrono::Utc::now().timestamp(), account_value: 105000.0, equity: 105500.0, realized_pnl: 5000.0, unrealized_pnl: 500.0, return_pct: 5.5 }] }) }

    }        let symbol = symbol.to_string();                    risk_level: "MEDIUM".to_string(),



    fn get_model_accounts(&self) -> impl std::future::Future<Output = Result<ModelAccounts, Box<dyn std::error::Error>>> + Send {        async move {                    base_capital: 100000.0,

        async move { Ok(ModelAccounts { accounts: vec![ModelAccount { model_id: 1, model_name: "GPT-4".to_string(), broker_id: "mock".to_string(), broker_name: "Mock".to_string(), account_value: 105000.0, equity: 105500.0, unrealized_pnl: 500.0, return_pct: 5.5, sharpe_ratio: Some(1.5), max_drawdown: Some(-0.02) }] }) }

    }            let mut rng = rand::thread_rng();                },



    fn get_positions(&self, _: Option<i32>) -> impl std::future::Future<Output = Result<Positions, Box<dyn std::error::Error>>> + Send {            let base_price = 50000.0;                ModelInfo {

        async move { Ok(Positions { positions: vec![Position { symbol: "BTCUSDT".to_string(), side: "long".to_string(), quantity: 0.5, entry_price: 49000.0, current_price: 50000.0, unrealized_pnl: 500.0, leverage: Some(1.0) }] }) }

    }            let mut bids = vec![];                    id: "crypto_claude".to_string(),



    fn get_balance(&self) -> impl std::future::Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send {            let mut asks = vec![];                    name: "Claude 动量交易".to_string(),

        async move { Ok(Balance { total: 105000.0, available: 50000.0, locked: 55000.0, currency: "USDT".to_string() }) }

    }            for i in 0..10 {                    strategy: "动量交易".to_string(),



    fn get_broker_account(&self) -> impl std::future::Future<Output = Result<BrokerAccount, Box<dyn std::error::Error>>> + Send {                bids.push(OrderbookLevel { price: base_price * (1.0 - 0.0001 * (i + 1) as f64), quantity: rng.gen_range(0.1..10.0) });                    description: "基于Claude的短期动量策略".to_string(),

        async move { Ok(BrokerAccount { broker_id: "mock".to_string(), broker_name: "Mock".to_string(), account_type: "spot".to_string(), total_equity: 105500.0, available_balance: 50000.0, margin_used: 55000.0, unrealized_pnl: 500.0 }) }

    }                asks.push(OrderbookLevel { price: base_price * (1.0 + 0.0001 * (i + 1) as f64), quantity: rng.gen_range(0.1..10.0) });                    risk_level: "HIGH".to_string(),

}

            }                    base_capital: 120000.0,

impl Analytics for MockBroker {

    fn get_analytics(&self) -> impl std::future::Future<Output = Result<AnalyticsData, Box<dyn std::error::Error>>> + Send {            Ok(Orderbook { symbol, bids, asks, timestamp: chrono::Utc::now().timestamp() })                },

        async move { let mut s = HashMap::new(); s.insert("total".to_string(), 3.0); Ok(AnalyticsData { summary: s, model_performance: vec![], market_metrics: HashMap::new() }) }

    }        }                ModelInfo {



    fn get_leaderboard(&self) -> impl std::future::Future<Output = Result<Leaderboard, Box<dyn std::error::Error>>> + Send {    }                    id: "crypto_gemini".to_string(),

        async move { Ok(Leaderboard { entries: vec![LeaderboardEntry { rank: 1, model_id: 1, model_name: "GPT-4".to_string(), total_return_pct: 5.5, sharpe_ratio: 1.5, max_drawdown: -0.02, win_rate: 0.65 }] }) }

    }                    name: "Gemini 价值投资".to_string(),



    fn get_since_inception_values(&self) -> impl std::future::Future<Output = Result<SinceInceptionValues, Box<dyn std::error::Error>>> + Send {    fn get_klines(&self, symbol: &str, interval: &str, limit: Option<i32>) -> impl Future<Output = Result<Klines, Box<dyn std::error::Error>>> + Send {                    strategy: "价值投资".to_string(),

        async move { Ok(SinceInceptionValues { models: vec![], start_date: chrono::Utc::now().timestamp(), end_date: chrono::Utc::now().timestamp() }) }

    }        let symbol = symbol.to_string();                    description: "Gemini驱动的长期价值投资".to_string(),



    fn get_conversations(&self) -> impl std::future::Future<Output = Result<Conversations, Box<dyn std::error::Error>>> + Send {        let interval = interval.to_string();                    risk_level: "LOW".to_string(),

        async move { Ok(Conversations { conversations: vec![] }) }

    }        async move {                    base_capital: 150000.0,



    fn get_models_list(&self) -> impl std::future::Future<Output = Result<Models, Box<dyn std::error::Error>>> + Send {            let mut rng = rand::thread_rng();                },

        async move { Ok(Models { models: vec![ModelInfo { model_id: 1, model_name: "GPT-4".to_string(), provider: "OpenAI".to_string(), strategy: "Trend".to_string(), status: "active".to_string() }] }) }

    }            let mut klines_data = vec![];            ],

}

            let now = chrono::Utc::now().timestamp();            "ctp" => vec![

            let interval_secs = match interval.as_str() { "1m" => 60, "1h" => 3600, _ => 60 };                ModelInfo {

            for i in 0..limit.unwrap_or(100) {                    id: "ctp_quant1".to_string(),

                let timestamp = now - ((limit.unwrap_or(100) - i) as i64 * interval_secs);                    name: "量化模型 Alpha".to_string(),

                let price = 50000.0 * (1.0 + rng.gen_range(-0.01..0.01));                    strategy: "套利策略".to_string(),

                klines_data.push(Kline { timestamp, open: price, high: price * 1.005, low: price * 0.995, close: price, volume: rng.gen_range(1000.0..10000.0), open_interest: None });                    description: "基于统计套利的量化模型".to_string(),

            }                    risk_level: "LOW".to_string(),

            Ok(Klines { symbol, interval, klines: klines_data })                    base_capital: 500000.0,

        }                },

    }                ModelInfo {

                    id: "ctp_quant2".to_string(),

    fn get_ticker_24h(&self, symbol: &str) -> impl Future<Output = Result<Ticker24h, Box<dyn std::error::Error>>> + Send {                    name: "量化模型 Beta".to_string(),

        let symbol = symbol.to_string();                    strategy: "趋势跟踪".to_string(),

        async move {                    description: "趋势跟踪量化模型".to_string(),

            let mut rng = rand::thread_rng();                    risk_level: "MEDIUM".to_string(),

            let price = 50000.0;                    base_capital: 600000.0,

            Ok(Ticker24h { symbol, last_price: price, change_24h: rng.gen_range(-5.0..5.0), high_24h: price * 1.03, low_24h: price * 0.97, volume_24h: rng.gen_range(1000000.0..10000000.0), open_interest: None, timestamp: chrono::Utc::now().timestamp() })                },

        }                ModelInfo {

    }                    id: "ctp_arbitrage".to_string(),

}                    name: "套利引擎".to_string(),

                    strategy: "跨期套利".to_string(),

impl Trading for MockBroker {                    description: "自动化跨期套利系统".to_string(),

    fn place_order(&self, _order: OrderRequest) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {                    risk_level: "VERY_LOW".to_string(),

        async move {                    base_capital: 800000.0,

            Ok(OrderResponse { order_id: format!("ORDER_{}", chrono::Utc::now().timestamp_millis()), status: OrderStatus::Accepted, timestamp: chrono::Utc::now().timestamp() })                },

        }                ModelInfo {

    }                    id: "ctp_hedging".to_string(),

                    name: "对冲系统".to_string(),

    fn cancel_order(&self, order_id: &str) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {                    strategy: "风险对冲".to_string(),

        let order_id = order_id.to_string();                    description: "多策略风险对冲系统".to_string(),

        async move { Ok(OrderResponse { order_id, status: OrderStatus::Cancelled, timestamp: chrono::Utc::now().timestamp() }) }                    risk_level: "LOW".to_string(),

    }                    base_capital: 700000.0,

                },

    fn get_order(&self, order_id: &str) -> impl Future<Output = Result<Order, Box<dyn std::error::Error>>> + Send {            ],

        let order_id = order_id.to_string();            "binance" => vec![

        async move {                ModelInfo {

            Ok(Order { order_id, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, order_type: OrderType::Limit, quantity: 0.1, filled_quantity: 0.1, price: Some(50000.0), avg_price: Some(50000.0), status: OrderStatus::Filled, created_at: chrono::Utc::now().timestamp(), updated_at: chrono::Utc::now().timestamp() })                    id: "binance_alpha".to_string(),

        }                    name: "Alpha AI".to_string(),

    }                    strategy: "动量交易".to_string(),

                    description: "高频动量交易AI".to_string(),

    fn get_orders(&self, _symbol: Option<&str>) -> impl Future<Output = Result<Orders, Box<dyn std::error::Error>>> + Send {                    risk_level: "VERY_HIGH".to_string(),

        async move { Ok(Orders { orders: vec![] }) }                    base_capital: 80000.0,

    }                },

                ModelInfo {

    fn get_trades(&self) -> impl Future<Output = Result<Trades, Box<dyn std::error::Error>>> + Send {                    id: "binance_beta".to_string(),

        async move { Ok(Trades { trades: vec![] }) }                    name: "Beta AI".to_string(),

    }                    strategy: "波段交易".to_string(),

}                    description: "中频波段交易AI".to_string(),

                    risk_level: "HIGH".to_string(),

impl AccountManagement for MockBroker {                    base_capital: 90000.0,

    fn get_account_totals(&self, _last_marker: Option<i32>) -> impl Future<Output = Result<AccountTotals, Box<dyn std::error::Error>>> + Send {                },

        async move {                ModelInfo {

            Ok(AccountTotals { accounts: vec![AccountTotal { model_id: 1, model_name: "GPT-4".to_string(), broker_name: "mock".to_string(), broker_id: "mock".to_string(), timestamp: chrono::Utc::now().timestamp(), account_value: 105000.0, equity: 105500.0, realized_pnl: 5000.0, unrealized_pnl: 500.0, return_pct: 5.5 }] })                    id: "binance_gamma".to_string(),

        }                    name: "Gamma AI".to_string(),

    }                    strategy: "网格交易".to_string(),

                    description: "网格交易AI".to_string(),

    fn get_model_accounts(&self) -> impl Future<Output = Result<ModelAccounts, Box<dyn std::error::Error>>> + Send {                    risk_level: "MEDIUM".to_string(),

        async move {                    base_capital: 100000.0,

            Ok(ModelAccounts { accounts: vec![ModelAccount { model_id: 1, model_name: "GPT-4".to_string(), broker_id: "mock".to_string(), broker_name: "Mock".to_string(), account_value: 105000.0, equity: 105500.0, unrealized_pnl: 500.0, return_pct: 5.5, sharpe_ratio: Some(1.5), max_drawdown: Some(-0.02) }] })                },

        }            ],

    }            "bybit" => vec![

                ModelInfo {

    fn get_positions(&self, _limit: Option<i32>) -> impl Future<Output = Result<Positions, Box<dyn std::error::Error>>> + Send {                    id: "bybit_deepseek".to_string(),

        async move { Ok(Positions { positions: vec![Position { symbol: "BTCUSDT".to_string(), side: "long".to_string(), quantity: 0.5, entry_price: 49000.0, current_price: 50000.0, unrealized_pnl: 500.0, leverage: Some(1.0) }] }) }                    name: "DeepSeek V3".to_string(),

    }                    strategy: "波段交易".to_string(),

                    description: "DeepSeek驱动的波段交易".to_string(),

    fn get_balance(&self) -> impl Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send {                    risk_level: "MEDIUM".to_string(),

        async move { Ok(Balance { total: 105000.0, available: 50000.0, locked: 55000.0, currency: "USDT".to_string() }) }                    base_capital: 110000.0,

    }                },

                ModelInfo {

    fn get_broker_account(&self) -> impl Future<Output = Result<BrokerAccount, Box<dyn std::error::Error>>> + Send {                    id: "bybit_qwen".to_string(),

        async move { Ok(BrokerAccount { broker_id: "mock".to_string(), broker_name: "Mock".to_string(), account_type: "spot".to_string(), total_equity: 105500.0, available_balance: 50000.0, margin_used: 55000.0, unrealized_pnl: 500.0 }) }                    name: "Qwen Max".to_string(),

    }                    strategy: "趋势跟踪".to_string(),

}                    description: "Qwen驱动的趋势跟踪".to_string(),

                    risk_level: "HIGH".to_string(),

impl Analytics for MockBroker {                    base_capital: 130000.0,

    fn get_analytics(&self) -> impl Future<Output = Result<AnalyticsData, Box<dyn std::error::Error>>> + Send {                },

        async move {                ModelInfo {

            let mut summary = HashMap::new();                    id: "bybit_mistral".to_string(),

            summary.insert("total_models".to_string(), 3.0);                    name: "Mistral Large".to_string(),

            Ok(AnalyticsData { summary, model_performance: vec![], market_metrics: HashMap::new() })                    strategy: "套利策略".to_string(),

        }                    description: "Mistral驱动的套利策略".to_string(),

    }                    risk_level: "LOW".to_string(),

                    base_capital: 120000.0,

    fn get_leaderboard(&self) -> impl Future<Output = Result<Leaderboard, Box<dyn std::error::Error>>> + Send {                },

        async move {            ],

            Ok(Leaderboard { entries: vec![LeaderboardEntry { rank: 1, model_id: 1, model_name: "GPT-4".to_string(), total_return_pct: 5.5, sharpe_ratio: 1.5, max_drawdown: -0.02, win_rate: 0.65 }] })            "kraken" => vec![

        }                ModelInfo {

    }                    id: "kraken_conservative".to_string(),

                    name: "Conservative AI".to_string(),

    fn get_since_inception_values(&self) -> impl Future<Output = Result<SinceInceptionValues, Box<dyn std::error::Error>>> + Send {                    strategy: "价值投资".to_string(),

        async move { Ok(SinceInceptionValues { models: vec![], start_date: chrono::Utc::now().timestamp(), end_date: chrono::Utc::now().timestamp() }) }                    description: "保守型价值投资AI".to_string(),

    }                    risk_level: "VERY_LOW".to_string(),

                    base_capital: 100000.0,

    fn get_conversations(&self) -> impl Future<Output = Result<Conversations, Box<dyn std::error::Error>>> + Send {                },

        async move { Ok(Conversations { conversations: vec![] }) }                ModelInfo {

    }                    id: "kraken_balanced".to_string(),

                    name: "Balanced AI".to_string(),

    fn get_models_list(&self) -> impl Future<Output = Result<Models, Box<dyn std::error::Error>>> + Send {                    strategy: "平衡配置".to_string(),

        async move {                    description: "平衡型配置AI".to_string(),

            Ok(Models { models: vec![ModelInfo { model_id: 1, model_name: "GPT-4".to_string(), provider: "OpenAI".to_string(), strategy: "Trend".to_string(), status: "active".to_string() }] })                    risk_level: "LOW".to_string(),

        }                    base_capital: 110000.0,

    }                },

}            ],

            _ => vec![ModelInfo {
                id: format!("{}_default", self.id),
                name: "Default Model".to_string(),
                strategy: "默认策略".to_string(),
                description: "默认交易模型".to_string(),
                risk_level: "MEDIUM".to_string(),
                base_capital: 50000.0,
            }],
        }
    }

    /// 获取交易品种
    fn get_symbols(&self) -> Vec<String> {
        match self.broker_type {
            BrokerType::Crypto => vec![
                "BTC".to_string(),
                "ETH".to_string(),
                "SOL".to_string(),
                "AVAX".to_string(),
            ],
            BrokerType::Futures => vec![
                "IF2504".to_string(),
                "IH2504".to_string(),
                "IC2504".to_string(),
                "IM2504".to_string(),
            ],
            BrokerType::Stock => vec![
                "AAPL".to_string(),
                "MSFT".to_string(),
                "GOOGL".to_string(),
                "TSLA".to_string(),
            ],
        }
    }

    /// 获取基础价格
    fn get_base_price(&self, symbol: &str) -> f64 {
        match symbol {
            "BTC" => 106000.0,
            "ETH" => 3800.0,
            "SOL" => 230.0,
            "AVAX" => 45.0,
            "IF2504" => 4200.0,
            "IH2504" => 3100.0,
            "IC2504" => 5800.0,
            "IM2504" => 4500.0,
            "AAPL" => 180.0,
            "MSFT" => 420.0,
            "GOOGL" => 150.0,
            "TSLA" => 250.0,
            _ => 100.0,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct ModelInfo {
    id: String,
    name: String,
    strategy: String,
    description: String,
    risk_level: String,
    base_capital: f64,
}

// ============================================================================
// MarketData Trait Implementation (行情接口实现)
// ============================================================================
impl MarketData for MockBroker {
    fn get_prices(&self) -> impl Future<Output = Result<Prices, Box<dyn std::error::Error>>> + Send {
        async move {
            let mut prices = HashMap::new();
            prices.insert("BTC".to_string(), 50000.0);
            prices.insert("ETH".to_string(), 3000.0);
            prices.insert("SOL".to_string(), 100.0);
            prices.insert("BNB".to_string(), 300.0);
            
            Ok(Prices { prices })
        }
    }

    fn get_orderbook(&self, symbol: &str) -> impl Future<Output = Result<Orderbook, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        async move {
            let mut rng = rand::thread_rng();
            let base_price = 50000.0; // 简化：使用固定基础价格
            
            let mut bids = vec![];
            let mut asks = vec![];
            
            // 生成10档买单
            for i in 0..10 {
                let price = base_price * (1.0 - 0.0001 * (i + 1) as f64);
                let quantity = rng.gen_range(0.1..10.0);
                bids.push(OrderbookLevel { price, quantity });
            }
            
            // 生成10档卖单
            for i in 0..10 {
                let price = base_price * (1.0 + 0.0001 * (i + 1) as f64);
                let quantity = rng.gen_range(0.1..10.0);
                asks.push(OrderbookLevel { price, quantity });
            }
            
            Ok(Orderbook {
                symbol,
                bids,
                asks,
                timestamp: chrono::Utc::now().timestamp(),
            })
        }
    }

    fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<i32>,
    ) -> impl Future<Output = Result<Klines, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        let interval = interval.to_string();
        async move {
            let mut rng = rand::thread_rng();
            let base_price = 50000.0;
            let limit = limit.unwrap_or(100);
            let mut data = vec![];
            
            let mut current_price = base_price;
            let now = chrono::Utc::now().timestamp();
            
            // 根据时间间隔计算秒数
            let interval_seconds = match interval.as_str() {
                "1m" => 60,
                "5m" => 300,
                "15m" => 900,
                "1h" => 3600,
                "4h" => 14400,
                "1d" => 86400,
                _ => 60,
            };
            
            for i in (0..limit).rev() {
                let timestamp = now - (i as i64 * interval_seconds);
                let change = rng.gen_range(-0.01..0.01);
                current_price *= 1.0 + change;
                
                let high = current_price * rng.gen_range(1.0..1.005);
                let low = current_price * rng.gen_range(0.995..1.0);
                let volume = rng.gen_range(1000.0..10000.0);
                
                data.push(Kline {
                    timestamp,
                    open: current_price,
                    high,
                    low,
                    close: current_price,
                    volume,
                    open_interest: None,
                });
            }
            
            Ok(Klines {
                symbol,
                interval,
                klines: data,
            })
        }
    }

    fn get_ticker_24h(&self, symbol: &str) -> impl Future<Output = Result<Ticker24h, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        async move {
            let mut rng = rand::thread_rng();
            let price = 50000.0;
            let change_pct = rng.gen_range(-5.0..5.0);
            let volume = rng.gen_range(1000000.0..10000000.0);
            
            Ok(Ticker24h {
                symbol,
                last_price: price,
                change_24h: change_pct,
                high_24h: price * 1.03,
                low_24h: price * 0.97,
                volume_24h: volume,
                open_interest: None,
                timestamp: chrono::Utc::now().timestamp(),
            })
        }
    }
}

// ============================================================================
// Trading Trait Implementation (交易接口实现)
// ============================================================================
impl Trading for MockBroker {
    fn place_order(&self, _order: OrderRequest) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {
        async move {
            use super::{OrderStatus};
            let order_id = format!("ORDER_{}", chrono::Utc::now().timestamp_millis());
            Ok(OrderResponse {
                order_id,
                status: OrderStatus::Accepted,
                timestamp: chrono::Utc::now().timestamp(),
            })
        }
    }

    fn cancel_order(&self, order_id: &str) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {
        let order_id = order_id.to_string();
        async move {
            use super::{OrderStatus};
            Ok(OrderResponse {
                order_id,
                status: OrderStatus::Cancelled,
                timestamp: chrono::Utc::now().timestamp(),
            })
        }
    }

    fn get_order(&self, order_id: &str) -> impl Future<Output = Result<Order, Box<dyn std::error::Error>>> + Send {
        let order_id = order_id.to_string();
        async move {
            use super::{OrderSide, OrderType, OrderStatus};
            Ok(Order {
                order_id,
                symbol: "BTCUSDT".to_string(),
                side: OrderSide::Buy,
                order_type: OrderType::Limit,
                quantity: 0.1,
                filled_quantity: 0.1,
                price: Some(50000.0),
                avg_price: Some(50000.0),
                status: OrderStatus::Filled,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
            })
        }
    }

    fn get_orders(&self, _symbol: Option<&str>) -> impl Future<Output = Result<Orders, Box<dyn std::error::Error>>> + Send {
        async move {
            Ok(Orders { orders: vec![] })
        }
    }

    fn get_trades(&self) -> impl Future<Output = Result<Trades, Box<dyn std::error::Error>>> + Send {
        async move {
            Ok(Trades { trades: vec![] })
        }
    }
}

// ============================================================================
// AccountManagement Trait Implementation (账户管理接口实现)
// ============================================================================
impl AccountManagement for MockBroker {
    fn get_account_totals(
        &self,
        _last_marker: Option<i32>,
    ) -> impl Future<Output = Result<AccountTotals, Box<dyn std::error::Error>>> + Send {
        async move {
        let mut rng = rand::thread_rng();
        let now = chrono::Utc::now().timestamp();
        let models = self.get_models();
        let mut account_totals = vec![];

        // 生成过去120分钟的历史数据（每分钟一个点）
        for model in &models {
            let base_value = model.base_capital;

            // 根据风险等级设置波动率和趋势
            let (volatility, trend_bias) = match model.risk_level.as_str() {
                "VERY_LOW" => (0.0005, 0.0001),
                "LOW" => (0.001, 0.0002),
                "MEDIUM" => (0.002, 0.0001),
                "HIGH" => (0.003, -0.0001),
                "VERY_HIGH" => (0.005, -0.0002),
                _ => (0.002, 0.0001),
            };

            let mut current_value = base_value;

            for i in 0..120 {
                let minutes_ago = 119 - i;
                let timestamp = now - (minutes_ago * 60);

                // 随机游走 + 小趋势
                let random_change = rng.gen_range(-volatility..volatility);
                let change_pct = random_change + trend_bias;
                current_value *= 1.0 + change_pct;

                let cumulative_return = (current_value - base_value) / base_value;
                let unrealized_pnl = current_value * rng.gen_range(-0.005..0.005);
                let dollar_equity = current_value + unrealized_pnl;

                account_totals.push(json!({
                    "model_id": &model.id,
                    "model_name": &model.name,
                    "strategy": &model.strategy,
                    "risk_level": &model.risk_level,
                    "exchange_id": self.id,
                    "timestamp": timestamp,
                    "account_value": current_value,
                    "dollar_equity": dollar_equity,
                    "equity": dollar_equity,
                    "realized_pnl": current_value - base_value,
                    "unrealized_pnl": unrealized_pnl,
                    "total_unrealized_pnl": unrealized_pnl,
                    "return_pct": cumulative_return * 100.0,
                    "cum_pnl_pct": cumulative_return * 100.0,
                    "sharpe_ratio": if i > 30 {
                        if cumulative_return > 0.0 {
                            rng.gen_range(0.8..2.5)
                        } else {
                            rng.gen_range(-0.5..0.8)
                        }
                    } else {
                        0.0
                    },
                    "since_inception_minute_marker": i,
                    "since_inception_hourly_marker": i / 60,
                    "positions": {}
                }));
            }
        }

        Ok(json!({
            "accountTotals": account_totals
        }))
    }

    async fn get_model_accounts(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let now = chrono::Utc::now().timestamp();
        let models = self.get_models();
        let mut accounts = vec![];

        for model in models {
            let base_value = model.base_capital;

            // 根据风险等级生成盈亏幅度
            let pnl_pct = match model.risk_level.as_str() {
                "VERY_LOW" => rng.gen_range(-0.02..0.04),
                "LOW" => rng.gen_range(-0.03..0.06),
                "MEDIUM" => rng.gen_range(-0.05..0.10),
                "HIGH" => rng.gen_range(-0.08..0.15),
                "VERY_HIGH" => rng.gen_range(-0.12..0.25),
                _ => rng.gen_range(-0.05..0.10),
            };

            let realized_pnl = base_value * pnl_pct;
            let total_unrealized_pnl = base_value * rng.gen_range(-0.02..0.03);
            let dollar_equity = base_value + realized_pnl + total_unrealized_pnl;
            let return_pct = (realized_pnl / base_value) * 100.0;

            // 根据风险等级生成胜率和交易次数
            let (win_rate, trade_count) = match model.risk_level.as_str() {
                "VERY_LOW" => (rng.gen_range(0.55..0.70), rng.gen_range(20..50)),
                "LOW" => (rng.gen_range(0.52..0.65), rng.gen_range(30..80)),
                "MEDIUM" => (rng.gen_range(0.48..0.62), rng.gen_range(50..150)),
                "HIGH" => (rng.gen_range(0.45..0.60), rng.gen_range(80..250)),
                "VERY_HIGH" => (rng.gen_range(0.42..0.58), rng.gen_range(100..500)),
                _ => (rng.gen_range(0.45..0.60), rng.gen_range(50..200)),
            };

            accounts.push(json!({
                "model_id": model.id,
                "model_name": model.name,
                "strategy": model.strategy,
                "risk_level": model.risk_level,
                "exchange_id": self.id,
                "timestamp": now,
                "account_value": base_value,
                "dollar_equity": dollar_equity,
                "equity": dollar_equity,
                "realized_pnl": realized_pnl,
                "unrealized_pnl": total_unrealized_pnl,
                "total_unrealized_pnl": total_unrealized_pnl,
                "return_pct": return_pct,
                "cum_pnl_pct": return_pct,
                "sharpe_ratio": if return_pct > 0.0 { rng.gen_range(0.8..2.5) } else { rng.gen_range(-0.5..0.8) },
                "win_rate": win_rate,
                "total_trades": trade_count,
                "winning_trades": (trade_count as f64 * win_rate) as i32,
                "losing_trades": trade_count - (trade_count as f64 * win_rate) as i32,
                "positions": {}
            }));
        }

        Ok(json!({
            "accounts": accounts
        }))
    }

    async fn get_positions(
        &self,
        _limit: Option<i32>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let symbols = self.get_symbols();
        let mut rng = rand::thread_rng();
        let now = chrono::Utc::now().timestamp();
        let mut positions = HashMap::new();

        // 根据交易所类型设置不同的持仓特征
        let (num_positions, leverage_range, risk_profile) = match self.id.as_str() {
            "crypto" => (rng.gen_range(2..=4), (5, 20), "aggressive"),
            "ctp" => (rng.gen_range(3..=5), (1, 10), "conservative"),
            "binance" => (rng.gen_range(3..=6), (10, 25), "very_aggressive"),
            "bybit" => (rng.gen_range(2..=5), (5, 15), "moderate"),
            "kraken" => (rng.gen_range(1..=3), (1, 5), "very_conservative"),
            _ => (rng.gen_range(2..=4), (1, 10), "moderate"),
        };

        let num_positions = num_positions.min(symbols.len());

        for i in 0..num_positions {
            let symbol = &symbols[i];
            let entry_price = self.get_base_price(symbol);
            let price_volatility = match risk_profile {
                "very_aggressive" => rng.gen_range(0.90..1.10),
                "aggressive" => rng.gen_range(0.93..1.07),
                "moderate" => rng.gen_range(0.95..1.05),
                "conservative" => rng.gen_range(0.97..1.03),
                _ => rng.gen_range(0.98..1.02),
            };

            let current_price = entry_price * price_volatility;
            let quantity = rng.gen_range(0.5..10.0);
            let side = if rng.gen_bool(0.6) { 1.0 } else { -1.0 };
            let signed_qty = quantity * side;
            let unrealized_pnl = (current_price - entry_price) * quantity * side;
            let leverage = rng.gen_range(leverage_range.0..=leverage_range.1);

            positions.insert(
                symbol.clone(),
                json!({
                    "symbol": symbol,
                    "entry_price": entry_price,
                    "current_price": current_price,
                    "quantity": signed_qty,
                    "unrealized_pnl": unrealized_pnl,
                    "leverage": leverage,
                    "timestamp": now,
                }),
            );
        }

        Ok(json!({
            "positions": positions
        }))
    }

    async fn get_balance(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        Ok(json!({
            "total_balance": rng.gen_range(10000.0..100000.0),
            "available_balance": rng.gen_range(5000.0..50000.0),
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn get_broker_account(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "broker_id": self.id,
            "broker_name": self.name,
            "account_type": match self.broker_type {
                BrokerType::Crypto => "crypto",
                BrokerType::Futures => "futures",
                BrokerType::Stock => "stock",
            },
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }
}

// ============================================================================
// Analytics Trait Implementation (分析与统计接口实现)
// ============================================================================
#[async_trait]
impl Analytics for MockBroker {
    async fn get_analytics(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "analytics": {}
        }))
    }

    async fn get_leaderboard(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "leaderboard": []
        }))
    }

    async fn get_since_inception_values(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "sinceInception": []
        }))
    }

    async fn get_conversations(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "conversations": []
        }))
    }

    async fn get_models_list(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let models = self.get_models();
        let models_json: Vec<Value> = models
            .iter()
            .map(|m| {
                json!({
                    "model_id": m.id,
                    "model_name": m.name,
                    "strategy": m.strategy,
                    "description": m.description,
                    "risk_level": m.risk_level,
                    "base_capital": m.base_capital,
                })
            })
            .collect();

        Ok(json!({
            "models": models_json
        }))
    }
}

// ============================================================================
// Broker Trait Implementation (组合特性实现)
// ============================================================================
impl Broker for MockBroker {
    fn broker_id(&self) -> &str {
        &self.id
    }

    fn broker_name(&self) -> &str {
        &self.name
    }
}
