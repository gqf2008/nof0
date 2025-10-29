use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub llm_provider: String,
    pub market: String,
    pub symbols: Vec<String>,
    pub enabled: bool,
}

impl Agent {
    pub fn new(id: String, name: String, llm_provider: String, market: String) -> Self {
        Self {
            id,
            name,
            llm_provider,
            market,
            symbols: Vec::new(),
            enabled: true,
        }
    }
}
