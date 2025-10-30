use serde::{Deserialize, Serialize};

/// 风险控制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// 仓位限制配置
    #[serde(default)]
    pub position_limits: PositionLimitsConfig,

    /// 损失限制配置
    #[serde(default)]
    pub loss_limits: LossLimitsConfig,

    /// 交易频率限制配置
    #[serde(default)]
    pub frequency_limits: FrequencyLimitsConfig,

    /// 订单大小限制配置
    #[serde(default)]
    pub order_size_limits: OrderSizeLimitsConfig,

    /// 是否启用风控
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            position_limits: PositionLimitsConfig::default(),
            loss_limits: LossLimitsConfig::default(),
            frequency_limits: FrequencyLimitsConfig::default(),
            order_size_limits: OrderSizeLimitsConfig::default(),
            enabled: true,
        }
    }
}

/// 仓位限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLimitsConfig {
    /// 单个品种最大仓位(USD)
    #[serde(default = "default_max_position_per_symbol")]
    pub max_position_per_symbol: f64,

    /// 总仓位限制(USD)
    #[serde(default = "default_max_total_position")]
    pub max_total_position: f64,

    /// 杠杆倍数限制
    #[serde(default = "default_max_leverage")]
    pub max_leverage: f64,

    /// 单个品种最大持仓比例(相对于总资产)
    #[serde(default = "default_max_position_ratio")]
    pub max_position_ratio: f64,
}

fn default_max_position_per_symbol() -> f64 {
    10000.0
}

fn default_max_total_position() -> f64 {
    50000.0
}

fn default_max_leverage() -> f64 {
    3.0
}

fn default_max_position_ratio() -> f64 {
    0.3 // 30%
}

impl Default for PositionLimitsConfig {
    fn default() -> Self {
        Self {
            max_position_per_symbol: default_max_position_per_symbol(),
            max_total_position: default_max_total_position(),
            max_leverage: default_max_leverage(),
            max_position_ratio: default_max_position_ratio(),
        }
    }
}

/// 损失限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossLimitsConfig {
    /// 每日最大亏损(USD)
    #[serde(default = "default_max_daily_loss")]
    pub max_daily_loss: f64,

    /// 最大回撤比例
    #[serde(default = "default_max_drawdown")]
    pub max_drawdown: f64,

    /// 单笔交易最大亏损(USD)
    #[serde(default = "default_max_loss_per_trade")]
    pub max_loss_per_trade: f64,

    /// 止损比例(相对于入场价)
    #[serde(default = "default_stop_loss_ratio")]
    pub stop_loss_ratio: f64,
}

fn default_max_daily_loss() -> f64 {
    1000.0
}

fn default_max_drawdown() -> f64 {
    0.2 // 20%
}

fn default_max_loss_per_trade() -> f64 {
    500.0
}

fn default_stop_loss_ratio() -> f64 {
    0.05 // 5%
}

impl Default for LossLimitsConfig {
    fn default() -> Self {
        Self {
            max_daily_loss: default_max_daily_loss(),
            max_drawdown: default_max_drawdown(),
            max_loss_per_trade: default_max_loss_per_trade(),
            stop_loss_ratio: default_stop_loss_ratio(),
        }
    }
}

/// 交易频率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyLimitsConfig {
    /// 每分钟最大订单数
    #[serde(default = "default_max_orders_per_minute")]
    pub max_orders_per_minute: u32,

    /// 每小时最大订单数
    #[serde(default = "default_max_orders_per_hour")]
    pub max_orders_per_hour: u32,

    /// 每天最大订单数
    #[serde(default = "default_max_orders_per_day")]
    pub max_orders_per_day: u32,

    /// 最小订单间隔(秒)
    #[serde(default = "default_min_order_interval")]
    pub min_order_interval_secs: u64,
}

fn default_max_orders_per_minute() -> u32 {
    10
}

fn default_max_orders_per_hour() -> u32 {
    100
}

fn default_max_orders_per_day() -> u32 {
    500
}

fn default_min_order_interval() -> u64 {
    1 // 1秒
}

impl Default for FrequencyLimitsConfig {
    fn default() -> Self {
        Self {
            max_orders_per_minute: default_max_orders_per_minute(),
            max_orders_per_hour: default_max_orders_per_hour(),
            max_orders_per_day: default_max_orders_per_day(),
            min_order_interval_secs: default_min_order_interval(),
        }
    }
}

/// 订单大小限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSizeLimitsConfig {
    /// 最小订单金额(USD)
    #[serde(default = "default_min_order_value")]
    pub min_order_value: f64,

    /// 最大订单金额(USD)
    #[serde(default = "default_max_order_value")]
    pub max_order_value: f64,

    /// 单笔订单最大占用资金比例
    #[serde(default = "default_max_order_ratio")]
    pub max_order_ratio: f64,
}

fn default_min_order_value() -> f64 {
    10.0
}

fn default_max_order_value() -> f64 {
    10000.0
}

fn default_max_order_ratio() -> f64 {
    0.5 // 50%
}

impl Default for OrderSizeLimitsConfig {
    fn default() -> Self {
        Self {
            min_order_value: default_min_order_value(),
            max_order_value: default_max_order_value(),
            max_order_ratio: default_max_order_ratio(),
        }
    }
}

impl RiskConfig {
    /// 从 YAML 文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到 YAML 文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
