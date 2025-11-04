/// 统一数据类型定义
use serde::{Deserialize, Serialize};

// ==================== 交易所相关 ====================

/// 交易所类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExchangeType {
    /// 期货
    Futures,
    /// 数字货币
    Crypto,
    /// 股票
    Stock,
    /// 外汇
    Forex,
}

/// 交易所信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub id: String,
    pub name: String,
    pub exchange_type: ExchangeType,
    pub enabled: bool,
    pub status: String,
    pub features: Vec<String>,
}

/// 连接状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub exchange_id: String,
    pub connected: bool,
    pub mode: String,
    pub connections: ConnectionDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broker_info: Option<BrokerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDetails {
    pub market_data: ConnectionState,
    pub trading: ConnectionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    pub connected: bool,
    pub reconnecting: bool,
    pub reconnect_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerInfo {
    pub broker_id: String,
    pub broker_name: String,
}

// ==================== 行情数据 ====================

/// 行情快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTicker {
    pub symbol: String,
    pub exchange_id: String,
    pub last_price: f64,
    pub bid_price: f64,
    pub ask_price: f64,
    pub volume_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub change_24h: f64,
    pub change_percent_24h: f64,
    pub timestamp: u64,
    pub update_time: String,
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub symbol: String,
    pub interval: String,
    pub open_time: u64,
    pub close_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub turnover: f64,
}

/// 订阅请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub symbols: Vec<String>,
}

// ==================== 账户相关 ====================

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub exchange_id: String,
    pub account_id: String,
    pub balance: f64,
    pub available: f64,
    pub frozen: f64,
    pub equity: f64,
    pub margin: f64,
    pub profit_loss: f64,
    pub currency: String,
    pub update_time: String,
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub exchange_id: String,
    pub direction: PositionDirection,
    pub volume: i32,
    pub available_volume: i32,
    pub frozen_volume: i32,
    pub avg_price: f64,
    pub last_price: f64,
    pub profit_loss: f64,
    pub margin: f64,
    pub open_time: String,
    pub update_time: String,
}

/// 持仓方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionDirection {
    Long,
    Short,
}

// ==================== 交易相关 ====================

/// 下单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub direction: OrderDirection,
    pub offset: OffsetFlag,
    pub price_type: PriceType,
    pub price: f64,
    pub volume: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

/// 订单信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub exchange_id: String,
    pub direction: OrderDirection,
    pub offset: OffsetFlag,
    pub price: f64,
    pub volume: i32,
    pub filled_volume: i32,
    pub status: OrderStatus,
    pub submit_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderDirection {
    Buy,
    Sell,
}

/// 开平标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OffsetFlag {
    Open,
    Close,
    CloseToday,
    CloseYesterday,
}

/// 价格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceType {
    Limit,
    Market,
    Stop,
    StopLimit,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    PartialFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub order_id: String,
    pub symbol: String,
    pub exchange_id: String,
    pub direction: OrderDirection,
    pub price: f64,
    pub volume: i32,
    pub trade_time: String,
}

// ==================== 合约信息 ====================

/// 合约信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub exchange_id: String,
    pub instrument_type: String,
    pub contract_size: i32,
    pub price_tick: f64,
    pub margin_ratio: f64,
    pub commission_ratio: f64,
    pub trading_hours: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_date: Option<String>,
}

// ==================== WebSocket 消息 ====================

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// 订阅请求
    Subscribe {
        channel: String,
        symbols: Vec<String>,
    },
    /// 取消订阅
    Unsubscribe {
        channel: String,
        symbols: Vec<String>,
    },
    /// 行情推送
    Market {
        exchange_id: String,
        data: MarketTicker,
    },
    /// 订单推送
    Order { exchange_id: String, data: Order },
    /// 持仓推送
    Position { exchange_id: String, data: Position },
    /// 账户推送
    Account { exchange_id: String, data: Account },
    /// Ping
    Ping,
    /// Pong
    Pong,
}
