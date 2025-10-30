use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 行情数据类型
// ============================================================================

/// 价格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String,
    pub price: f64,
    pub timestamp: i64,
}

/// 价格列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub prices: HashMap<String, f64>,
}

/// 订单簿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub symbol: String,
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: f64,
    pub quantity: f64,
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Klines {
    pub symbol: String,
    pub interval: String,
    pub klines: Vec<Kline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<i64>,
}

/// 24小时行情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker24h {
    pub symbol: String,
    pub last_price: f64,
    pub change_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub volume_24h: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<i64>,
    pub timestamp: i64,
}

// ============================================================================
// 交易数据类型
// ============================================================================

/// 订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// 订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: OrderStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Accepted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// 订单信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub price: Option<f64>,
    pub avg_price: Option<f64>,
    pub status: OrderStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 订单列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orders {
    pub orders: Vec<Order>,
}

/// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub fee: f64,
    pub timestamp: i64,
}

/// 成交列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trades {
    pub trades: Vec<Trade>,
}

// ============================================================================
// 账户管理数据类型
// ============================================================================

/// 账户历史数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTotal {
    pub model_id: String,
    pub model_name: String,
    pub strategy: String,
    pub risk_level: String,
    pub broker_id: String,
    pub timestamp: i64,
    pub account_value: f64,
    pub dollar_equity: f64,
    pub equity: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub return_pct: f64,
    pub cum_pnl_pct: f64,
    pub sharpe_ratio: f64,
    pub since_inception_minute_marker: i32,
    pub since_inception_hourly_marker: i32,
}

/// 账户历史数据列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTotals {
    #[serde(rename = "accountTotals")]
    pub account_totals: Vec<AccountTotal>,
}

/// 模型账户摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAccount {
    pub model_id: String,
    pub model_name: String,
    pub strategy: String,
    pub risk_level: String,
    pub broker_id: String,
    pub timestamp: i64,
    pub account_value: f64,
    pub dollar_equity: f64,
    pub equity: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub return_pct: f64,
    pub cum_pnl_pct: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
}

/// 模型账户列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAccounts {
    pub accounts: Vec<ModelAccount>,
}

/// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub entry_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub unrealized_pnl: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leverage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<f64>,
    pub timestamp: i64,
}

/// 持仓列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Positions {
    pub positions: HashMap<String, Position>,
}

/// 账户余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub total_balance: f64,
    pub available: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_used: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen_margin: Option<f64>,
    pub currency: String,
    pub timestamp: i64,
}

/// 经纪商账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerAccount {
    pub broker_id: String,
    pub broker_name: String,
    pub broker_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    pub timestamp: i64,
}

// ============================================================================
// 分析统计数据类型
// ============================================================================

/// 分析数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    // TODO: 添加具体的分析字段
    pub metrics: std::collections::HashMap<String, f64>,
}

/// 排行榜条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub model_id: String,
    pub model_name: String,
    pub return_pct: f64,
    pub sharpe_ratio: f64,
    pub total_trades: i32,
    pub win_rate: f64,
}

/// 排行榜
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    pub leaderboard: Vec<LeaderboardEntry>,
}

/// 初始值数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinceInceptionValues {
    #[serde(rename = "sinceInception")]
    pub since_inception: Vec<AccountTotal>,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub conversation_id: String,
    pub model_id: String,
    pub messages: Vec<Message>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

/// 对话列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversations {
    pub conversations: Vec<Conversation>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub model_name: String,
    pub strategy: String,
    pub description: String,
    pub risk_level: String,
    pub base_capital: f64,
}

/// 模型列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Models {
    pub models: Vec<ModelInfo>,
}
