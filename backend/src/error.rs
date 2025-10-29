use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // 暂时注释掉 sqlx，等需要时再启用
    // #[error("Database error: {0}")]
    // Database(#[from] sqlx::Error),
    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("Market adapter error: {0}")]
    MarketAdapter(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::LlmProvider(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::MarketAdapter(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::McpProtocol(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
