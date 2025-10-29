use super::{server::ToolHandler, types::McpTool};
use async_trait::async_trait;

// 示例工具: 获取市场价格
pub struct GetPriceTool;

impl GetPriceTool {
    pub fn schema() -> McpTool {
        McpTool {
            name: "get_price".to_string(),
            description: "Get current price for a symbol".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Trading symbol (e.g., BTC/USDT)"
                    }
                },
                "required": ["symbol"]
            }),
        }
    }
}

#[async_trait]
impl ToolHandler for GetPriceTool {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let symbol = input
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol parameter"))?;

        // TODO: 实际获取价格
        Ok(serde_json::json!({
            "symbol": symbol,
            "price": 50000.0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

// 示例工具: 下单
pub struct PlaceOrderTool;

impl PlaceOrderTool {
    pub fn schema() -> McpTool {
        McpTool {
            name: "place_order".to_string(),
            description: "Place a trading order".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Trading symbol"
                    },
                    "side": {
                        "type": "string",
                        "enum": ["buy", "sell"],
                        "description": "Order side"
                    },
                    "quantity": {
                        "type": "number",
                        "description": "Order quantity"
                    }
                },
                "required": ["symbol", "side", "quantity"]
            }),
        }
    }
}

#[async_trait]
impl ToolHandler for PlaceOrderTool {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        let symbol = input
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol parameter"))?;

        let side = input
            .get("side")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing side parameter"))?;

        let quantity = input
            .get("quantity")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing quantity parameter"))?;

        // TODO: 实际下单
        Ok(serde_json::json!({
            "order_id": "123456",
            "symbol": symbol,
            "side": side,
            "quantity": quantity,
            "status": "filled"
        }))
    }
}
