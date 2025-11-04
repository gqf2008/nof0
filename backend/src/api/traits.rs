use anyhow::Result;
/// 统一交易接口 Trait 定义
///
/// 所有交易所适配器都需要实现这些接口
use async_trait::async_trait;

use super::types::*;

/// 交易所适配器核心接口
#[async_trait]
pub trait ExchangeAdapter: Send + Sync {
    /// 获取交易所ID
    fn exchange_id(&self) -> &str;

    /// 获取交易所名称
    fn exchange_name(&self) -> &str;

    /// 获取交易所类型
    fn exchange_type(&self) -> ExchangeType;

    /// 连接交易所
    async fn connect(&self) -> Result<()>;

    /// 断开连接
    async fn disconnect(&self) -> Result<()>;

    /// 获取连接状态
    async fn get_status(&self) -> Result<ConnectionStatus>;

    /// 是否已连接
    async fn is_connected(&self) -> bool;
}

/// 行情数据接口
#[async_trait]
pub trait MarketDataProvider: ExchangeAdapter {
    /// 订阅行情
    async fn subscribe_market(&self, symbols: Vec<String>) -> Result<()>;

    /// 取消订阅
    async fn unsubscribe_market(&self, symbols: Vec<String>) -> Result<()>;

    /// 获取单个行情
    async fn get_ticker(&self, symbol: &str) -> Result<MarketTicker>;

    /// 获取批量行情
    async fn get_tickers(&self, symbols: Vec<String>) -> Result<Vec<MarketTicker>>;

    /// 获取K线数据
    async fn get_klines(&self, symbol: &str, interval: &str, limit: usize) -> Result<Vec<Kline>>;
}

/// 账户查询接口
#[async_trait]
pub trait AccountProvider: ExchangeAdapter {
    /// 查询账户信息
    async fn get_account(&self) -> Result<Account>;

    /// 查询持仓
    async fn get_positions(&self) -> Result<Vec<Position>>;
}

/// 交易操作接口
#[async_trait]
pub trait TradingProvider: ExchangeAdapter {
    /// 下单
    async fn place_order(&self, request: OrderRequest) -> Result<Order>;

    /// 撤单
    async fn cancel_order(&self, order_id: &str) -> Result<()>;

    /// 查询订单
    async fn get_order(&self, order_id: &str) -> Result<Order>;

    /// 查询所有订单
    async fn get_orders(&self, status: Option<OrderStatus>, limit: usize) -> Result<Vec<Order>>;

    /// 查询成交记录
    async fn get_trades(
        &self,
        start_time: Option<String>,
        end_time: Option<String>,
    ) -> Result<Vec<Trade>>;
}

/// 合约查询接口
#[async_trait]
pub trait InstrumentProvider: ExchangeAdapter {
    /// 查询合约列表
    async fn get_instruments(&self, instrument_type: Option<String>) -> Result<Vec<Instrument>>;

    /// 查询合约详情
    async fn get_instrument(&self, symbol: &str) -> Result<Instrument>;
}

/// 完整交易所接口(组合所有接口)
#[async_trait]
pub trait FullExchangeAdapter:
    ExchangeAdapter + MarketDataProvider + AccountProvider + TradingProvider + InstrumentProvider
{
    // 可以添加额外的组合方法
}

// 为实现了所有子接口的类型自动实现 FullExchangeAdapter
impl<T> FullExchangeAdapter for T where
    T: ExchangeAdapter
        + MarketDataProvider
        + AccountProvider
        + TradingProvider
        + InstrumentProvider
{
}
