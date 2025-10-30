// 风险控制模块

pub mod config;
pub mod manager;
pub mod metrics;
pub mod rules;

pub use config::*;
pub use manager::*;
pub use metrics::*;
pub use rules::*;

/// 风险检查结果
#[derive(Debug, Clone)]
pub struct RiskCheckResult {
    /// 是否通过检查
    pub passed: bool,
    /// 失败原因(如果未通过)
    pub reason: Option<String>,
    /// 触发的规则名称
    pub rule_name: String,
    /// 风险级别: Low, Medium, High
    pub risk_level: RiskLevel,
}

impl RiskCheckResult {
    /// 创建通过的结果
    pub fn pass(rule_name: impl Into<String>) -> Self {
        Self {
            passed: true,
            reason: None,
            rule_name: rule_name.into(),
            risk_level: RiskLevel::Low,
        }
    }

    /// 创建失败的结果
    pub fn fail(
        rule_name: impl Into<String>,
        reason: impl Into<String>,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            passed: false,
            reason: Some(reason.into()),
            rule_name: rule_name.into(),
            risk_level,
        }
    }
}

/// 风险级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中等风险
    Medium,
    /// 高风险
    High,
    /// 严重风险
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// 订单信息(用于风控检查)
#[derive(Debug, Clone)]
pub struct OrderInfo {
    /// 交易对/合约
    pub symbol: String,
    /// 订单方向: buy/sell
    pub side: String,
    /// 订单数量
    pub quantity: f64,
    /// 订单价格(可选,市价单为None)
    pub price: Option<f64>,
    /// 订单类型: market/limit
    pub order_type: String,
    /// 账户ID
    pub account_id: String,
}

/// 风险事件
#[derive(Debug, Clone)]
pub struct RiskEvent {
    /// 事件时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 事件类型
    pub event_type: RiskEventType,
    /// 规则名称
    pub rule_name: String,
    /// 风险级别
    pub risk_level: RiskLevel,
    /// 描述
    pub description: String,
    /// 相关的订单信息(如果有)
    pub order_info: Option<OrderInfo>,
}

/// 风险事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskEventType {
    /// 订单被拒绝
    OrderRejected,
    /// 触发风险警告
    RiskWarning,
    /// 触发止损
    StopLossTriggered,
    /// 触发最大回撤
    MaxDrawdownHit,
    /// 超过仓位限制
    PositionLimitExceeded,
    /// 超过交易频率
    FrequencyLimitExceeded,
}

impl std::fmt::Display for RiskEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskEventType::OrderRejected => write!(f, "ORDER_REJECTED"),
            RiskEventType::RiskWarning => write!(f, "RISK_WARNING"),
            RiskEventType::StopLossTriggered => write!(f, "STOP_LOSS_TRIGGERED"),
            RiskEventType::MaxDrawdownHit => write!(f, "MAX_DRAWDOWN_HIT"),
            RiskEventType::PositionLimitExceeded => write!(f, "POSITION_LIMIT_EXCEEDED"),
            RiskEventType::FrequencyLimitExceeded => write!(f, "FREQUENCY_LIMIT_EXCEEDED"),
        }
    }
}
