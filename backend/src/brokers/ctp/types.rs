use serde::{Deserialize, Serialize};

/// CTP连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpConfig {
    /// 经纪商ID
    pub broker_id: String,

    /// 投资者ID
    pub investor_id: String,

    /// 密码
    pub password: String,

    /// 行情前置地址
    pub md_address: String,

    /// 交易前置地址
    pub td_address: String,

    /// 应用ID (用于穿透式监管)
    #[serde(default)]
    pub app_id: String,

    /// 认证码
    #[serde(default)]
    pub auth_code: String,

    /// 用户产品信息
    #[serde(default = "default_product_info")]
    pub user_product_info: String,

    /// 是否启用模拟模式 (不连接真实CTP)
    #[serde(default)]
    pub mock_mode: bool,
}

fn default_product_info() -> String {
    "nof0".to_string()
}

impl Default for CtpConfig {
    fn default() -> Self {
        Self {
            broker_id: "9999".to_string(),
            investor_id: "000000".to_string(),
            password: "password".to_string(),
            md_address: "tcp://180.168.146.187:10131".to_string(),
            td_address: "tcp://180.168.146.187:10130".to_string(),
            app_id: String::new(),
            auth_code: String::new(),
            user_product_info: default_product_info(),
            mock_mode: true, // 默认使用模拟模式
        }
    }
}

impl CtpConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.broker_id.is_empty() {
            return Err("broker_id cannot be empty".to_string());
        }

        if self.investor_id.is_empty() {
            return Err("investor_id cannot be empty".to_string());
        }

        if self.password.is_empty() {
            return Err("password cannot be empty".to_string());
        }

        if self.md_address.is_empty() {
            return Err("md_address cannot be empty".to_string());
        }

        if self.td_address.is_empty() {
            return Err("td_address cannot be empty".to_string());
        }

        Ok(())
    }
}

/// CTP订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpOrderRequest {
    /// 合约代码 (例如: "IF2501")
    pub instrument_id: String,

    /// 买卖方向 ('0'=买, '1'=卖)
    pub direction: char,

    /// 开平标志 ('0'=开仓, '1'=平仓, '3'=平今)
    pub offset_flag: char,

    /// 价格
    pub price: f64,

    /// 数量
    pub volume: i32,

    /// 价格类型 ('1'=市价, '2'=限价)
    pub price_type: char,

    /// 投机套保标志 ('1'=投机, '2'=套利, '3'=套保)
    pub hedge_flag: char,
}

/// CTP订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpOrderResponse {
    /// 订单系统编号
    pub order_sys_id: String,

    /// 本地报单编号
    pub order_ref: String,

    /// 合约代码
    pub instrument_id: String,

    /// 订单状态
    pub order_status: CtpOrderStatus,

    /// 状态信息
    pub status_msg: String,
}

/// CTP订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CtpOrderStatus {
    /// 未知
    Unknown,
    /// 全部成交
    AllTraded,
    /// 部分成交
    PartTraded,
    /// 未成交
    NoTraded,
    /// 已撤单
    Canceled,
    /// 错误
    Error,
}

impl std::fmt::Display for CtpOrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CtpOrderStatus::Unknown => write!(f, "Unknown"),
            CtpOrderStatus::AllTraded => write!(f, "AllTraded"),
            CtpOrderStatus::PartTraded => write!(f, "PartTraded"),
            CtpOrderStatus::NoTraded => write!(f, "NoTraded"),
            CtpOrderStatus::Canceled => write!(f, "Canceled"),
            CtpOrderStatus::Error => write!(f, "Error"),
        }
    }
}

/// CTP行情数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpMarketData {
    /// 合约代码
    pub instrument_id: String,

    /// 最新价
    pub last_price: f64,

    /// 买一价
    pub bid_price: f64,

    /// 卖一价
    pub ask_price: f64,

    /// 买一量
    pub bid_volume: i32,

    /// 卖一量
    pub ask_volume: i32,

    /// 成交量
    pub volume: i32,

    /// 持仓量
    pub open_interest: i32,

    /// 最高价
    pub highest_price: f64,

    /// 最低价
    pub lowest_price: f64,

    /// 更新时间
    pub update_time: String,
}

/// CTP持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpPosition {
    /// 合约代码
    pub instrument_id: String,

    /// 持仓方向 ('2'=多头, '3'=空头)
    pub direction: char,

    /// 持仓数量
    pub position: i32,

    /// 今日持仓
    pub today_position: i32,

    /// 可用持仓
    pub available: i32,

    /// 开仓均价
    pub open_cost: f64,

    /// 持仓盈亏
    pub position_profit: f64,
}

/// CTP资金信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtpAccount {
    /// 账户ID
    pub account_id: String,

    /// 可用资金
    pub available: f64,

    /// 占用保证金
    pub margin: f64,

    /// 冻结保证金
    pub frozen_margin: f64,

    /// 平仓盈亏
    pub close_profit: f64,

    /// 持仓盈亏
    pub position_profit: f64,

    /// 手续费
    pub commission: f64,

    /// 期初余额
    pub pre_balance: f64,

    /// 当前余额
    pub balance: f64,
}

impl CtpAccount {
    /// 计算权益
    pub fn equity(&self) -> f64 {
        self.balance + self.position_profit
    }

    /// 计算风险度
    pub fn risk_ratio(&self) -> f64 {
        if self.balance > 0.0 {
            self.margin / self.balance
        } else {
            0.0
        }
    }
}
