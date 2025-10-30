use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 经纪商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    /// 经纪商ID
    pub id: String,

    /// 经纪商名称
    pub name: String,

    /// 英文名称
    pub name_en: String,

    /// 描述
    pub description: String,

    /// 图标（emoji或图标代码）
    pub icon: String,

    /// 主题色
    pub color: String,

    /// 是否启用
    pub enabled: bool,

    /// 运行状态
    pub status: String,

    /// 路由路径
    pub route: String,

    /// API端点
    pub api_endpoint: String,

    /// 功能列表
    pub features: Vec<String>,

    /// 特定配置（可选）
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// WebSocket配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// 重连间隔（秒）
    pub reconnect_interval: u64,

    /// 最大重连次数
    pub max_reconnect_attempts: u32,
}

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// 默认主题
    pub default: String,

    /// 可用主题列表
    pub available: Vec<String>,
}

/// 全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// 页面刷新间隔（秒）
    pub refresh_interval: u64,

    /// 是否自动连接
    pub auto_connect: bool,

    /// WebSocket配置
    pub websocket: WebSocketConfig,

    /// 主题配置
    pub theme: ThemeConfig,
}

/// 经纪商配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokersConfig {
    /// 经纪商列表
    pub brokers: Vec<BrokerConfig>,

    /// 全局设置
    pub settings: GlobalSettings,
}

impl BrokersConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: BrokersConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 从默认路径加载配置
    pub fn load_default() -> Result<Self> {
        // 尝试多个可能的路径
        let possible_paths = vec![
            "config/brokers.yaml",
            "backend/config/brokers.yaml",
            "../config/brokers.yaml",
            "./brokers.yaml",
            // 保留旧路径以保持向后兼容
            "config/exchanges.yaml",
            "backend/config/exchanges.yaml",
            "../config/exchanges.yaml",
            "./exchanges.yaml",
        ];

        for path in possible_paths {
            if Path::new(path).exists() {
                return Self::from_file(path);
            }
        }

        // 如果找不到配置文件，返回默认配置
        tracing::warn!("No brokers config file found, using default configuration");
        Ok(Self::default())
    }

    /// 获取已启用的经纪商列表
    pub fn get_enabled_brokers(&self) -> Vec<BrokerConfig> {
        self.brokers.iter().filter(|e| e.enabled).cloned().collect()
    }

    /// 根据ID查找经纪商配置
    pub fn get_broker(&self, id: &str) -> Option<&BrokerConfig> {
        self.brokers.iter().find(|e| e.id == id)
    }
}

impl Default for BrokersConfig {
    fn default() -> Self {
        Self {
            brokers: vec![
                BrokerConfig {
                    id: "crypto".to_string(),
                    name: "数字货币经纪商".to_string(),
                    name_en: "Crypto Broker".to_string(),
                    description: "实时监控数字货币交易账户".to_string(),
                    icon: "₿".to_string(),
                    color: "#f7931a".to_string(),
                    enabled: true,
                    status: "运行中".to_string(),
                    route: "/crypto".to_string(),
                    api_endpoint: "http://localhost:8788/api/nof1".to_string(),
                    features: vec![
                        "实时行情".to_string(),
                        "持仓管理".to_string(),
                        "成交记录".to_string(),
                    ],
                    config: None,
                },
                BrokerConfig {
                    id: "ctp".to_string(),
                    name: "CTP 期货经纪商".to_string(),
                    name_en: "CTP Futures Broker".to_string(),
                    description: "中国期货市场交易终端".to_string(),
                    icon: "📊".to_string(),
                    color: "#00a0e9".to_string(),
                    enabled: true,
                    status: "运行中".to_string(),
                    route: "/ctp".to_string(),
                    api_endpoint: "http://localhost:8788/api/ctp".to_string(),
                    features: vec![
                        "期货行情".to_string(),
                        "持仓监控".to_string(),
                        "交易执行".to_string(),
                    ],
                    config: None,
                },
            ],
            settings: GlobalSettings {
                refresh_interval: 5,
                auto_connect: false,
                websocket: WebSocketConfig {
                    reconnect_interval: 3,
                    max_reconnect_attempts: 5,
                },
                theme: ThemeConfig {
                    default: "dark".to_string(),
                    available: vec!["dark".to_string(), "light".to_string()],
                },
            },
        }
    }
}
