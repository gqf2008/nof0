use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub llm_providers: Vec<LlmProviderConfig>,
    pub markets: Vec<MarketConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmProviderConfig {
    pub name: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub market_type: String,
    pub symbols: Vec<String>,
    pub enabled: bool,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        // TODO: 从配置文件加载
        Ok(Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8788,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://nof0:nof0@localhost:5432/nof0".to_string()),
                max_connections: 10,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            llm_providers: vec![],
            markets: vec![],
        })
    }
}
