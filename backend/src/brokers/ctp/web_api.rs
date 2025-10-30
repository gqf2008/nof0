// CTP Web API 接口
// 提供HTTP RESTful API和WebSocket接口供前端调用

use anyhow::Result;
use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::{
    CtpAccount, CtpConfig, CtpMarketData, CtpOrderRequest, CtpPosition, RealCtpConnection,
};

/// CTP连接管理器 (单例)
pub struct CtpConnectionManager {
    connection: Arc<RwLock<Option<RealCtpConnection>>>,
    config: Arc<RwLock<Option<CtpConfig>>>,
    // 广播通道,用于实时推送更新
    market_data_tx: broadcast::Sender<CtpMarketData>,
    account_update_tx: broadcast::Sender<CtpAccount>,
    position_update_tx: broadcast::Sender<Vec<CtpPosition>>,
}

impl CtpConnectionManager {
    pub fn new() -> Self {
        let (market_data_tx, _) = broadcast::channel(1000);
        let (account_update_tx, _) = broadcast::channel(100);
        let (position_update_tx, _) = broadcast::channel(100);

        Self {
            connection: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(None)),
            market_data_tx,
            account_update_tx,
            position_update_tx,
        }
    }

    /// 获取或创建连接
    pub async fn get_connection(&self) -> Result<Arc<RwLock<RealCtpConnection>>> {
        let conn_guard = self.connection.read().await;
        if conn_guard.is_some() {
            drop(conn_guard);
            // 创建新的Arc指向已有连接
            let conn = self.connection.read().await;
            if let Some(ref c) = *conn {
                // TODO: 这里需要改进,因为RealCtpConnection不能直接clone
                return Err(anyhow::anyhow!("Connection exists but cannot be cloned"));
            }
        }
        Err(anyhow::anyhow!("No active connection"))
    }
}

// ==================== API请求/响应类型 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigRequest {
    pub broker_id: String,
    pub investor_id: String,
    pub password: String,
    pub md_address: String,
    pub td_address: String,
    pub app_id: Option<String>,
    pub auth_code: Option<String>,
    pub user_product_info: String,
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
            message: "操作成功".to_string(),
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

#[derive(Debug, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub md_connected: bool,
    pub td_connected: bool,
    pub md_reconnecting: bool,
    pub td_reconnecting: bool,
    pub md_reconnect_attempts: i32,
    pub td_reconnect_attempts: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub instruments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequest {
    pub instrument_id: String,
    pub direction: char,
    pub offset_flag: char,
    pub price: f64,
    pub volume: i32,
    pub price_type: char,
    pub hedge_flag: char,
}

// ==================== API路由处理器 ====================

/// 创建CTP API路由 (返回带有 state 的 Router)
pub fn create_routes() -> Router {
    let manager = Arc::new(CtpConnectionManager::new());

    Router::new()
        // 配置管理
        .route("/api/ctp/config", post(save_config))
        .route("/api/ctp/config", get(get_config))
        // 连接管理
        .route("/api/ctp/connect", post(connect))
        .route("/api/ctp/disconnect", post(disconnect))
        .route("/api/ctp/status", get(get_status))
        // 行情订阅
        .route("/api/ctp/subscribe", post(subscribe_market_data))
        .route("/api/ctp/unsubscribe", post(unsubscribe_market_data))
        .route("/api/ctp/market/{instrument_id}", get(get_market_data))
        // 账户查询
        .route("/api/ctp/account", get(query_account))
        .route("/api/ctp/positions", get(query_positions))
        // 交易操作
        .route("/api/ctp/order", post(place_order))
        // WebSocket实时推送
        .route("/api/ctp/ws", get(websocket_handler))
        .with_state(manager)
}

/// 保存配置
async fn save_config(
    State(manager): State<Arc<CtpConnectionManager>>,
    Json(req): Json<ConfigRequest>,
) -> Json<ApiResponse<()>> {
    let config = CtpConfig {
        broker_id: req.broker_id,
        investor_id: req.investor_id,
        password: req.password,
        md_address: req.md_address,
        td_address: req.td_address,
        app_id: req.app_id.unwrap_or_default(),
        auth_code: req.auth_code.unwrap_or_default(),
        user_product_info: req.user_product_info,
        mock_mode: false,
    };

    *manager.config.write().await = Some(config);

    Json(ApiResponse::success(()))
}

/// 获取配置
async fn get_config(
    State(manager): State<Arc<CtpConnectionManager>>,
) -> Json<ApiResponse<ConfigRequest>> {
    let config_guard = manager.config.read().await;

    if let Some(config) = config_guard.as_ref() {
        let req = ConfigRequest {
            broker_id: config.broker_id.clone(),
            investor_id: config.investor_id.clone(),
            password: "******".to_string(), // 不返回真实密码
            md_address: config.md_address.clone(),
            td_address: config.td_address.clone(),
            app_id: Some(config.app_id.clone()),
            auth_code: Some("******".to_string()),
            user_product_info: config.user_product_info.clone(),
        };
        Json(ApiResponse::success(req))
    } else {
        Json(ApiResponse::error("配置未设置".to_string()))
    }
}

/// 连接CTP
async fn connect(State(manager): State<Arc<CtpConnectionManager>>) -> Json<ApiResponse<()>> {
    let config_guard = manager.config.read().await;

    if config_guard.is_none() {
        return Json(ApiResponse::error("请先配置连接参数".to_string()));
    }

    let config = config_guard.as_ref().unwrap().clone();
    drop(config_guard);

    let mut conn = RealCtpConnection::new(config);

    match conn.connect().await {
        Ok(_) => {
            *manager.connection.write().await = Some(conn);
            Json(ApiResponse::success(()))
        }
        Err(e) => Json(ApiResponse::error(format!("连接失败: {}", e))),
    }
}

/// 断开连接
async fn disconnect(State(manager): State<Arc<CtpConnectionManager>>) -> Json<ApiResponse<()>> {
    *manager.connection.write().await = None;
    Json(ApiResponse::success(()))
}

/// 获取连接状态
async fn get_status(
    State(manager): State<Arc<CtpConnectionManager>>,
) -> Json<ApiResponse<ConnectionStatus>> {
    let conn_guard = manager.connection.read().await;

    if let Some(conn) = conn_guard.as_ref() {
        let connected = conn.is_connected().await;
        let (md_reconnecting, td_reconnecting) = conn.get_reconnect_status().await;
        let (md_attempts, td_attempts) = conn.get_reconnect_attempts();

        let status = ConnectionStatus {
            connected,
            md_connected: connected,
            td_connected: connected,
            md_reconnecting,
            td_reconnecting,
            md_reconnect_attempts: md_attempts,
            td_reconnect_attempts: td_attempts,
        };

        Json(ApiResponse::success(status))
    } else {
        let status = ConnectionStatus {
            connected: false,
            md_connected: false,
            td_connected: false,
            md_reconnecting: false,
            td_reconnecting: false,
            md_reconnect_attempts: 0,
            td_reconnect_attempts: 0,
        };
        Json(ApiResponse::success(status))
    }
}

/// 订阅行情
async fn subscribe_market_data(
    State(manager): State<Arc<CtpConnectionManager>>,
    Json(req): Json<SubscribeRequest>,
) -> Json<ApiResponse<()>> {
    let conn_guard = manager.connection.read().await;

    if let Some(conn) = conn_guard.as_ref() {
        match conn.subscribe_market_data(req.instruments).await {
            Ok(_) => Json(ApiResponse::success(())),
            Err(e) => Json(ApiResponse::error(format!("订阅失败: {}", e))),
        }
    } else {
        Json(ApiResponse::error("未连接".to_string()))
    }
}

/// 取消订阅
async fn unsubscribe_market_data(
    State(manager): State<Arc<CtpConnectionManager>>,
    Json(req): Json<SubscribeRequest>,
) -> Json<ApiResponse<()>> {
    // TODO: 实现取消订阅逻辑
    Json(ApiResponse::success(()))
}

/// 获取行情数据
async fn get_market_data(
    State(manager): State<Arc<CtpConnectionManager>>,
    Path(instrument_id): Path<String>,
) -> Json<ApiResponse<CtpMarketData>> {
    let conn_guard = manager.connection.read().await;

    if let Some(conn) = conn_guard.as_ref() {
        match conn.get_market_data(&instrument_id).await {
            Ok(data) => Json(ApiResponse::success(data)),
            Err(e) => Json(ApiResponse::error(format!("获取行情失败: {}", e))),
        }
    } else {
        Json(ApiResponse::error("未连接".to_string()))
    }
}

/// 查询账户
async fn query_account(
    State(manager): State<Arc<CtpConnectionManager>>,
) -> Json<ApiResponse<CtpAccount>> {
    let conn_guard = manager.connection.read().await;

    if let Some(conn) = conn_guard.as_ref() {
        match conn.query_account().await {
            Ok(account) => Json(ApiResponse::success(account)),
            Err(e) => Json(ApiResponse::error(format!("查询账户失败: {}", e))),
        }
    } else {
        Json(ApiResponse::error("未连接".to_string()))
    }
}

/// 查询持仓
async fn query_positions(
    State(manager): State<Arc<CtpConnectionManager>>,
) -> Json<ApiResponse<Vec<CtpPosition>>> {
    let conn_guard = manager.connection.read().await;

    if let Some(conn) = conn_guard.as_ref() {
        match conn.query_position().await {
            Ok(positions) => Json(ApiResponse::success(positions)),
            Err(e) => Json(ApiResponse::error(format!("查询持仓失败: {}", e))),
        }
    } else {
        Json(ApiResponse::error("未连接".to_string()))
    }
}

/// 下单
async fn place_order(
    State(manager): State<Arc<CtpConnectionManager>>,
    Json(req): Json<OrderRequest>,
) -> Json<ApiResponse<String>> {
    let conn_guard = manager.connection.read().await;

    if let Some(conn) = conn_guard.as_ref() {
        let order_req = CtpOrderRequest {
            instrument_id: req.instrument_id,
            direction: req.direction,
            offset_flag: req.offset_flag,
            price: req.price,
            volume: req.volume,
            price_type: req.price_type,
            hedge_flag: req.hedge_flag,
        };

        match conn.place_order(order_req).await {
            Ok(response) => Json(ApiResponse::success(response.order_ref)),
            Err(e) => Json(ApiResponse::error(format!("下单失败: {}", e))),
        }
    } else {
        Json(ApiResponse::error("未连接".to_string()))
    }
}

/// WebSocket处理器
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(manager): State<Arc<CtpConnectionManager>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, manager))
}

/// 处理WebSocket连接
async fn handle_websocket(mut socket: WebSocket, manager: Arc<CtpConnectionManager>) {
    use axum::extract::ws::Message;

    // 订阅广播通道
    let mut market_rx = manager.market_data_tx.subscribe();
    let mut account_rx = manager.account_update_tx.subscribe();
    let mut position_rx = manager.position_update_tx.subscribe();

    // 启动接收任务
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // 接收行情更新
                Ok(data) = market_rx.recv() => {
                    let json = serde_json::to_string(&data).unwrap_or_default();
                    let text = format!(r#"{{"type":"market","data":{}}}"#, json);
                    let msg = Message::Text(text.into());
                    if socket.send(msg).await.is_err() {
                        break;
                    }
                }

                // 接收账户更新
                Ok(account) = account_rx.recv() => {
                    let json = serde_json::to_string(&account).unwrap_or_default();
                    let text = format!(r#"{{"type":"account","data":{}}}"#, json);
                    let msg = Message::Text(text.into());
                    if socket.send(msg).await.is_err() {
                        break;
                    }
                }

                // 接收持仓更新
                Ok(positions) = position_rx.recv() => {
                    let json = serde_json::to_string(&positions).unwrap_or_default();
                    let text = format!(r#"{{"type":"positions","data":{}}}"#, json);
                    let msg = Message::Text(text.into());
                    if socket.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    });
}
