pub mod binance;
pub mod ctp;
pub mod mock_broker;
pub mod okex;
pub mod types;

#[allow(unused_imports)]
pub use binance::BinanceBroker;
#[allow(unused_imports)]
pub use ctp::CtpBroker;
#[allow(unused_imports)]
pub use mock_broker::MockBroker;
#[allow(unused_imports)]
pub use okex::OkexBroker;
pub use types::*;

use std::future::Future;

/// 行情数据接口
/// 提供市场行情、价格等数据
pub trait MarketData: Send + Sync {
    /// 获取实时价格
    fn get_prices(&self)
        -> impl Future<Output = Result<Prices, Box<dyn std::error::Error>>> + Send;

    /// 获取市场深度
    fn get_orderbook(
        &self,
        symbol: &str,
    ) -> impl Future<Output = Result<Orderbook, Box<dyn std::error::Error>>> + Send;

    /// 获取K线数据
    fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<i32>,
    ) -> impl Future<Output = Result<Klines, Box<dyn std::error::Error>>> + Send;

    /// 获取24小时ticker
    fn get_ticker_24h(
        &self,
        symbol: &str,
    ) -> impl Future<Output = Result<Ticker24h, Box<dyn std::error::Error>>> + Send;
}

/// 交易接口
/// 提供下单、撤单等交易功能
pub trait Trading: Send + Sync {
    /// 下单
    fn place_order(
        &self,
        order: OrderRequest,
    ) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send;

    /// 撤单
    fn cancel_order(
        &self,
        order_id: &str,
    ) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send;

    /// 查询订单
    fn get_order(
        &self,
        order_id: &str,
    ) -> impl Future<Output = Result<Order, Box<dyn std::error::Error>>> + Send;

    /// 查询所有订单
    fn get_orders(
        &self,
        symbol: Option<&str>,
    ) -> impl Future<Output = Result<Orders, Box<dyn std::error::Error>>> + Send;

    /// 查询成交记录
    fn get_trades(&self)
        -> impl Future<Output = Result<Trades, Box<dyn std::error::Error>>> + Send;
}

/// 账户管理接口
/// 提供账户信息、持仓、资金等数据
pub trait AccountManagement: Send + Sync {
    /// 获取账户历史数据（用于图表）
    /// 返回所有 AI 模型在指定时间范围内的账户净值历史
    fn get_account_totals(
        &self,
        last_marker: Option<i32>,
    ) -> impl Future<Output = Result<AccountTotals, Box<dyn std::error::Error>>> + Send;

    /// 获取模型账户摘要（用于账户卡片显示）
    /// 返回所有 AI 模型的当前账户状态
    fn get_model_accounts(
        &self,
    ) -> impl Future<Output = Result<ModelAccounts, Box<dyn std::error::Error>>> + Send;

    /// 获取持仓数据
    fn get_positions(
        &self,
        limit: Option<i32>,
    ) -> impl Future<Output = Result<Positions, Box<dyn std::error::Error>>> + Send;

    /// 获取账户余额
    fn get_balance(
        &self,
    ) -> impl Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send;

    /// 获取经纪商总账户信息
    fn get_broker_account(
        &self,
    ) -> impl Future<Output = Result<BrokerAccount, Box<dyn std::error::Error>>> + Send;
}

/// 分析与统计接口
/// 提供策略分析、排行榜等统计数据
pub trait Analytics: Send + Sync {
    /// 获取分析数据
    fn get_analytics(
        &self,
    ) -> impl Future<Output = Result<AnalyticsData, Box<dyn std::error::Error>>> + Send;

    /// 获取排行榜数据
    fn get_leaderboard(
        &self,
    ) -> impl Future<Output = Result<Leaderboard, Box<dyn std::error::Error>>> + Send;

    /// 获取初始值数据
    fn get_since_inception_values(
        &self,
    ) -> impl Future<Output = Result<SinceInceptionValues, Box<dyn std::error::Error>>> + Send;

    /// 获取 AI 对话记录
    fn get_conversations(
        &self,
    ) -> impl Future<Output = Result<Conversations, Box<dyn std::error::Error>>> + Send;

    /// 获取模型列表
    fn get_models_list(
        &self,
    ) -> impl Future<Output = Result<Models, Box<dyn std::error::Error>>> + Send;
}

/// Broker 完整接口
/// 聚合所有功能接口
pub trait Broker: MarketData + Trading + AccountManagement + Analytics + Send + Sync {
    /// 获取经纪商 ID
    fn broker_id(&self) -> &str;

    /// 获取经纪商名称
    fn broker_name(&self) -> &str;
}

/// 经纪商枚举类型
/// 由于使用了 impl Future，trait 不能是 dyn-compatible，所以使用 enum 替代
pub enum BrokerInstance {
    Mock(MockBroker),
    Ctp(CtpBroker),
    Binance(BinanceBroker),
    Okex(OkexBroker),
}

impl Broker for BrokerInstance {
    fn broker_id(&self) -> &str {
        match self {
            BrokerInstance::Mock(b) => b.broker_id(),
            BrokerInstance::Ctp(b) => b.broker_id(),
            BrokerInstance::Binance(b) => b.broker_id(),
            BrokerInstance::Okex(b) => b.broker_id(),
        }
    }

    fn broker_name(&self) -> &str {
        match self {
            BrokerInstance::Mock(b) => b.broker_name(),
            BrokerInstance::Ctp(b) => b.broker_name(),
            BrokerInstance::Binance(b) => b.broker_name(),
            BrokerInstance::Okex(b) => b.broker_name(),
        }
    }
}

impl MarketData for BrokerInstance {
    fn get_prices(
        &self,
    ) -> impl Future<Output = Result<Prices, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_prices().await,
                BrokerInstance::Ctp(b) => b.get_prices().await,
                BrokerInstance::Binance(b) => b.get_prices().await,
                BrokerInstance::Okex(b) => b.get_prices().await,
            }
        }
    }

    fn get_orderbook(
        &self,
        symbol: &str,
    ) -> impl Future<Output = Result<Orderbook, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_orderbook(&symbol).await,
                BrokerInstance::Ctp(b) => b.get_orderbook(&symbol).await,
                BrokerInstance::Binance(b) => b.get_orderbook(&symbol).await,
                BrokerInstance::Okex(b) => b.get_orderbook(&symbol).await,
            }
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
            match self {
                BrokerInstance::Mock(b) => b.get_klines(&symbol, &interval, limit).await,
                BrokerInstance::Ctp(b) => b.get_klines(&symbol, &interval, limit).await,
                BrokerInstance::Binance(b) => b.get_klines(&symbol, &interval, limit).await,
                BrokerInstance::Okex(b) => b.get_klines(&symbol, &interval, limit).await,
            }
        }
    }

    fn get_ticker_24h(
        &self,
        symbol: &str,
    ) -> impl Future<Output = Result<Ticker24h, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.to_string();
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_ticker_24h(&symbol).await,
                BrokerInstance::Ctp(b) => b.get_ticker_24h(&symbol).await,
                BrokerInstance::Binance(b) => b.get_ticker_24h(&symbol).await,
                BrokerInstance::Okex(b) => b.get_ticker_24h(&symbol).await,
            }
        }
    }
}

impl Trading for BrokerInstance {
    fn place_order(
        &self,
        order: OrderRequest,
    ) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.place_order(order).await,
                BrokerInstance::Ctp(b) => b.place_order(order).await,
                BrokerInstance::Binance(b) => b.place_order(order).await,
                BrokerInstance::Okex(b) => b.place_order(order).await,
            }
        }
    }

    fn cancel_order(
        &self,
        order_id: &str,
    ) -> impl Future<Output = Result<OrderResponse, Box<dyn std::error::Error>>> + Send {
        let order_id = order_id.to_string();
        async move {
            match self {
                BrokerInstance::Mock(b) => b.cancel_order(&order_id).await,
                BrokerInstance::Ctp(b) => b.cancel_order(&order_id).await,
                BrokerInstance::Binance(b) => b.cancel_order(&order_id).await,
                BrokerInstance::Okex(b) => b.cancel_order(&order_id).await,
            }
        }
    }

    fn get_order(
        &self,
        order_id: &str,
    ) -> impl Future<Output = Result<Order, Box<dyn std::error::Error>>> + Send {
        let order_id = order_id.to_string();
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_order(&order_id).await,
                BrokerInstance::Ctp(b) => b.get_order(&order_id).await,
                BrokerInstance::Binance(b) => b.get_order(&order_id).await,
                BrokerInstance::Okex(b) => b.get_order(&order_id).await,
            }
        }
    }

    fn get_orders(
        &self,
        symbol: Option<&str>,
    ) -> impl Future<Output = Result<Orders, Box<dyn std::error::Error>>> + Send {
        let symbol = symbol.map(|s| s.to_string());
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_orders(symbol.as_deref()).await,
                BrokerInstance::Ctp(b) => b.get_orders(symbol.as_deref()).await,
                BrokerInstance::Binance(b) => b.get_orders(symbol.as_deref()).await,
                BrokerInstance::Okex(b) => b.get_orders(symbol.as_deref()).await,
            }
        }
    }

    fn get_trades(
        &self,
    ) -> impl Future<Output = Result<Trades, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_trades().await,
                BrokerInstance::Ctp(b) => b.get_trades().await,
                BrokerInstance::Binance(b) => b.get_trades().await,
                BrokerInstance::Okex(b) => b.get_trades().await,
            }
        }
    }
}

impl AccountManagement for BrokerInstance {
    fn get_account_totals(
        &self,
        last_marker: Option<i32>,
    ) -> impl Future<Output = Result<AccountTotals, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_account_totals(last_marker).await,
                BrokerInstance::Ctp(b) => b.get_account_totals(last_marker).await,
                BrokerInstance::Binance(b) => b.get_account_totals(last_marker).await,
                BrokerInstance::Okex(b) => b.get_account_totals(last_marker).await,
            }
        }
    }

    fn get_model_accounts(
        &self,
    ) -> impl Future<Output = Result<ModelAccounts, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_model_accounts().await,
                BrokerInstance::Ctp(b) => b.get_model_accounts().await,
                BrokerInstance::Binance(b) => b.get_model_accounts().await,
                BrokerInstance::Okex(b) => b.get_model_accounts().await,
            }
        }
    }

    fn get_positions(
        &self,
        limit: Option<i32>,
    ) -> impl Future<Output = Result<Positions, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_positions(limit).await,
                BrokerInstance::Ctp(b) => b.get_positions(limit).await,
                BrokerInstance::Binance(b) => b.get_positions(limit).await,
                BrokerInstance::Okex(b) => b.get_positions(limit).await,
            }
        }
    }

    fn get_balance(
        &self,
    ) -> impl Future<Output = Result<Balance, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_balance().await,
                BrokerInstance::Ctp(b) => b.get_balance().await,
                BrokerInstance::Binance(b) => b.get_balance().await,
                BrokerInstance::Okex(b) => b.get_balance().await,
            }
        }
    }

    fn get_broker_account(
        &self,
    ) -> impl Future<Output = Result<BrokerAccount, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_broker_account().await,
                BrokerInstance::Ctp(b) => b.get_broker_account().await,
                BrokerInstance::Binance(b) => b.get_broker_account().await,
                BrokerInstance::Okex(b) => b.get_broker_account().await,
            }
        }
    }
}

impl Analytics for BrokerInstance {
    fn get_analytics(
        &self,
    ) -> impl Future<Output = Result<AnalyticsData, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_analytics().await,
                BrokerInstance::Ctp(b) => b.get_analytics().await,
                BrokerInstance::Binance(b) => b.get_analytics().await,
                BrokerInstance::Okex(b) => b.get_analytics().await,
            }
        }
    }

    fn get_leaderboard(
        &self,
    ) -> impl Future<Output = Result<Leaderboard, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_leaderboard().await,
                BrokerInstance::Ctp(b) => b.get_leaderboard().await,
                BrokerInstance::Binance(b) => b.get_leaderboard().await,
                BrokerInstance::Okex(b) => b.get_leaderboard().await,
            }
        }
    }

    fn get_since_inception_values(
        &self,
    ) -> impl Future<Output = Result<SinceInceptionValues, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_since_inception_values().await,
                BrokerInstance::Ctp(b) => b.get_since_inception_values().await,
                BrokerInstance::Binance(b) => b.get_since_inception_values().await,
                BrokerInstance::Okex(b) => b.get_since_inception_values().await,
            }
        }
    }

    fn get_conversations(
        &self,
    ) -> impl Future<Output = Result<Conversations, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_conversations().await,
                BrokerInstance::Ctp(b) => b.get_conversations().await,
                BrokerInstance::Binance(b) => b.get_conversations().await,
                BrokerInstance::Okex(b) => b.get_conversations().await,
            }
        }
    }

    fn get_models_list(
        &self,
    ) -> impl Future<Output = Result<Models, Box<dyn std::error::Error>>> + Send {
        async move {
            match self {
                BrokerInstance::Mock(b) => b.get_models_list().await,
                BrokerInstance::Ctp(b) => b.get_models_list().await,
                BrokerInstance::Binance(b) => b.get_models_list().await,
                BrokerInstance::Okex(b) => b.get_models_list().await,
            }
        }
    }
}

/// 经纪商注册表
pub struct BrokerRegistry {
    brokers: std::collections::HashMap<String, BrokerInstance>,
}

impl BrokerRegistry {
    pub fn new() -> Self {
        Self {
            brokers: std::collections::HashMap::new(),
        }
    }

    /// 注册经纪商
    pub fn register(&mut self, broker: BrokerInstance) {
        let id = broker.broker_id().to_string();
        self.brokers.insert(id, broker);
    }

    /// 获取经纪商
    pub fn get(&self, broker_id: &str) -> Option<&BrokerInstance> {
        self.brokers.get(broker_id)
    }

    /// 列出所有经纪商 ID
    pub fn list_ids(&self) -> Vec<String> {
        self.brokers.keys().cloned().collect()
    }
}

impl Default for BrokerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
