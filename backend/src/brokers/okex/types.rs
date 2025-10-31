use serde::{Deserialize, Serialize};

/// OKEX连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexConfig {
    /// API Key
    pub api_key: String,

    /// API Secret
    pub api_secret: String,

    /// Passphrase
    pub passphrase: String,

    /// 是否使用模拟盘
    #[serde(default)]
    pub simulated: bool,

    /// 基础URL (如果需要自定义)
    #[serde(default)]
    pub base_url: Option<String>,

    /// WebSocket URL (如果需要自定义)
    #[serde(default)]
    pub ws_url: Option<String>,

    /// 是否启用模拟模式 (不连接真实OKEX)
    #[serde(default = "default_true")]
    pub mock_mode: bool,
}

fn default_true() -> bool {
    true
}

impl Default for OkexConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_secret: String::new(),
            passphrase: String::new(),
            simulated: true,
            base_url: None,
            ws_url: None,
            mock_mode: true,
        }
    }
}

impl OkexConfig {
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
        if !self.mock_mode {
            if self.api_key.is_empty() {
                return Err("api_key cannot be empty".to_string());
            }

            if self.api_secret.is_empty() {
                return Err("api_secret cannot be empty".to_string());
            }

            if self.passphrase.is_empty() {
                return Err("passphrase cannot be empty".to_string());
            }
        }

        Ok(())
    }
}
