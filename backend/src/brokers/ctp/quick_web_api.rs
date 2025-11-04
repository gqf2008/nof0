/// CTP Web API 快速修复版本
/// 直接使用 CtpBroker 提供数据给前端
use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

use crate::brokers::ctp::{CtpBroker, CtpConfig};
use crate::brokers::{MarketData, Prices, Ticker24h};

/// CTP API 状态管理（使用懒加载）
pub struct CtpApiState {
    config: CtpConfig,
    broker: Arc<OnceCell<CtpBroker>>,
}

impl CtpApiState {
    pub fn new(config: CtpConfig) -> Self {
        Self {
            config,
            broker: Arc::new(OnceCell::new()),
        }
    }

    /// 懒加载获取 broker
    async fn get_broker(&self) -> &CtpBroker {
        self.broker
            .get_or_init(|| async {
                tracing::info!("🚀 懒加载初始化 CTP Broker (Mock 模式)");
                CtpBroker::new(
                    "ctp_web".to_string(),
                    "CTP Web API".to_string(),
                    self.config.clone(),
                )
            })
            .await
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "成功".to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub instruments: Vec<String>,
}

/// 创建 CTP API 路由（简化版）
pub fn create_quick_ctp_routes() -> Router {
    // 使用默认的 Mock 配置
    let config = CtpConfig {
        broker_id: "9999".to_string(),
        investor_id: "test".to_string(),
        password: "test".to_string(),
        md_address: "tcp://trading.openctp.cn:30011".to_string(),
        td_address: "tcp://trading.openctp.cn:30011".to_string(),
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
        user_product_info: "nof0".to_string(),
        mock_mode: true, // 使用 Mock 模式
    };

    let state = Arc::new(CtpApiState::new(config));

    Router::new()
        // 获取单个合约行情
        .route("/api/ctp/market/{instrument}", get(get_market_data))
        // 获取状态
        .route("/api/ctp/status", get(get_status))
        // 获取多个合约价格
        .route("/api/ctp/prices", get(get_prices))
        // 订阅行情（Mock 模式下无需实际订阅）
        .route("/api/ctp/subscribe", post(subscribe))
        .with_state(state)
}

/// 获取行情数据
async fn get_market_data(
    State(state): State<Arc<CtpApiState>>,
    Path(instrument): Path<String>,
) -> Json<ApiResponse<Ticker24h>> {
    let broker = state.get_broker().await;
    match broker.get_ticker_24h(&instrument).await {
        Ok(ticker) => Json(ApiResponse::success(ticker)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取连接状态
async fn get_status(State(_state): State<Arc<CtpApiState>>) -> Json<ApiResponse<StatusInfo>> {
    // Mock 模式始终返回已连接
    let status = StatusInfo {
        connected: true,
        mode: "Mock".to_string(),
        broker_id: "9999".to_string(),
    };

    Json(ApiResponse::success(status))
}

/// 获取批量价格
async fn get_prices(State(state): State<Arc<CtpApiState>>) -> Json<ApiResponse<Prices>> {
    let broker = state.get_broker().await;
    match broker.get_prices().await {
        Ok(prices) => Json(ApiResponse::success(prices)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 订阅行情
async fn subscribe(
    State(_state): State<Arc<CtpApiState>>,
    Json(_req): Json<SubscribeRequest>,
) -> Json<ApiResponse<String>> {
    // Mock 模式下直接返回成功
    Json(ApiResponse::success("已订阅（Mock模式）".to_string()))
}

#[derive(Debug, Serialize)]
pub struct StatusInfo {
    pub connected: bool,
    pub mode: String,
    pub broker_id: String,
}

#[derive(Debug, Serialize)]
pub struct PricesResponse {
    pub prices: std::collections::HashMap<String, f64>,
    pub timestamp: String,
}
