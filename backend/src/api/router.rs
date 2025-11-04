/// 统一 API 路由
///
/// 提供 `/api/v1/*` 的统一路由层
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{
    response::{ApiResponse, ErrorCode},
    traits::*,
    types::*,
};

/// 交易所管理器
pub struct ExchangeManager {
    exchanges: std::collections::HashMap<String, Arc<dyn FullExchangeAdapter>>,
}

impl ExchangeManager {
    pub fn new() -> Self {
        Self {
            exchanges: std::collections::HashMap::new(),
        }
    }

    /// 注册交易所
    pub fn register(&mut self, adapter: Arc<dyn FullExchangeAdapter>) {
        let id = adapter.exchange_id().to_string();
        self.exchanges.insert(id, adapter);
    }

    /// 获取交易所
    pub fn get(&self, exchange_id: &str) -> Option<&Arc<dyn FullExchangeAdapter>> {
        self.exchanges.get(exchange_id)
    }

    /// 列出所有交易所
    pub fn list(&self) -> Vec<ExchangeInfo> {
        self.exchanges
            .values()
            .map(|adapter| ExchangeInfo {
                id: adapter.exchange_id().to_string(),
                name: adapter.exchange_name().to_string(),
                exchange_type: adapter.exchange_type(),
                enabled: true,
                status: "online".to_string(),
                features: vec![],
            })
            .collect()
    }
}

/// 创建统一 API 路由
pub fn create_unified_routes(manager: Arc<ExchangeManager>) -> Router {
    Router::new()
        // 交易所配置
        .route("/api/v1/exchanges", get(list_exchanges))
        .route("/api/v1/exchanges/{exchange_id}/status", get(get_status))
        .route("/api/v1/exchanges/{exchange_id}/connect", post(connect))
        .route(
            "/api/v1/exchanges/{exchange_id}/disconnect",
            post(disconnect),
        )
        // 行情数据
        .route(
            "/api/v1/exchanges/{exchange_id}/market/subscribe",
            post(subscribe_market),
        )
        .route(
            "/api/v1/exchanges/{exchange_id}/market/unsubscribe",
            post(unsubscribe_market),
        )
        .route(
            "/api/v1/exchanges/{exchange_id}/market/ticker/{symbol}",
            get(get_ticker),
        )
        .route(
            "/api/v1/exchanges/{exchange_id}/market/tickers",
            get(get_tickers),
        )
        .route(
            "/api/v1/exchanges/{exchange_id}/market/klines/{symbol}",
            get(get_klines),
        )
        // 账户查询
        .route("/api/v1/exchanges/{exchange_id}/account", get(get_account))
        .route(
            "/api/v1/exchanges/{exchange_id}/positions",
            get(get_positions),
        )
        // 交易操作
        .route("/api/v1/exchanges/{exchange_id}/orders", post(place_order))
        .route("/api/v1/exchanges/{exchange_id}/orders", get(get_orders))
        .route(
            "/api/v1/exchanges/{exchange_id}/orders/{order_id}",
            get(get_order),
        )
        .route(
            "/api/v1/exchanges/{exchange_id}/orders/{order_id}",
            delete(cancel_order),
        )
        .route("/api/v1/exchanges/{exchange_id}/trades", get(get_trades))
        // 合约查询
        .route(
            "/api/v1/exchanges/{exchange_id}/instruments",
            get(get_instruments),
        )
        .route(
            "/api/v1/exchanges/{exchange_id}/instruments/{symbol}",
            get(get_instrument),
        )
        .with_state(manager)
}

// ==================== Handler 实现 ====================

/// 列出所有交易所
async fn list_exchanges(
    State(manager): State<Arc<ExchangeManager>>,
) -> Json<ApiResponse<ListExchangesResponse>> {
    let exchanges = manager.list();
    Json(ApiResponse::success(ListExchangesResponse { exchanges }))
}

#[derive(Serialize)]
struct ListExchangesResponse {
    exchanges: Vec<ExchangeInfo>,
}

/// 获取连接状态
async fn get_status(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<ConnectionStatus>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_status().await {
        Ok(status) => Json(ApiResponse::success(status)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 连接交易所
async fn connect(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<()>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.connect().await {
        Ok(_) => Json(ApiResponse::success_with_message((), "连接成功")),
        Err(e) => Json(ApiResponse::error(
            ErrorCode::ConnectionFailed,
            e.to_string(),
        )),
    }
}

/// 断开连接
async fn disconnect(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<()>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.disconnect().await {
        Ok(_) => Json(ApiResponse::success_with_message((), "断开成功")),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 订阅行情
async fn subscribe_market(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Json(req): Json<SubscribeRequest>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<()>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.subscribe_market(req.symbols).await {
        Ok(_) => Json(ApiResponse::success_with_message((), "订阅成功")),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 取消订阅
async fn unsubscribe_market(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Json(req): Json<SubscribeRequest>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<()>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.unsubscribe_market(req.symbols).await {
        Ok(_) => Json(ApiResponse::success_with_message((), "取消订阅成功")),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 获取行情
async fn get_ticker(
    State(manager): State<Arc<ExchangeManager>>,
    Path((exchange_id, symbol)): Path<(String, String)>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<MarketTicker>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_ticker(&symbol).await {
        Ok(ticker) => Json(ApiResponse::success(ticker)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

#[derive(Deserialize)]
struct GetTickersQuery {
    symbols: String, // 逗号分隔的符号列表
}

/// 获取批量行情
async fn get_tickers(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Query(query): Query<GetTickersQuery>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Vec<MarketTicker>>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();

    match adapter.get_tickers(symbols).await {
        Ok(tickers) => Json(ApiResponse::success(tickers)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

#[derive(Deserialize)]
struct GetKlinesQuery {
    interval: String,
    limit: Option<usize>,
}

/// 获取K线
async fn get_klines(
    State(manager): State<Arc<ExchangeManager>>,
    Path((exchange_id, symbol)): Path<(String, String)>,
    Query(query): Query<GetKlinesQuery>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Vec<Kline>>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    let limit = query.limit.unwrap_or(100);

    match adapter.get_klines(&symbol, &query.interval, limit).await {
        Ok(klines) => Json(ApiResponse::success(klines)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 查询账户
async fn get_account(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Account>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_account().await {
        Ok(account) => Json(ApiResponse::success(account)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 查询持仓
async fn get_positions(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Vec<Position>>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_positions().await {
        Ok(positions) => Json(ApiResponse::success(positions)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 下单
async fn place_order(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Json(req): Json<OrderRequest>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Order>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.place_order(req).await {
        Ok(order) => Json(ApiResponse::success(order)),
        Err(e) => Json(ApiResponse::error(ErrorCode::OrderFailed, e.to_string())),
    }
}

#[derive(Deserialize)]
struct GetOrdersQuery {
    status: Option<OrderStatus>,
    limit: Option<usize>,
}

/// 查询订单列表
async fn get_orders(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Query(query): Query<GetOrdersQuery>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Vec<Order>>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    let limit = query.limit.unwrap_or(50);

    match adapter.get_orders(query.status, limit).await {
        Ok(orders) => Json(ApiResponse::success(orders)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 查询单个订单
async fn get_order(
    State(manager): State<Arc<ExchangeManager>>,
    Path((exchange_id, order_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Order>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_order(&order_id).await {
        Ok(order) => Json(ApiResponse::success(order)),
        Err(e) => Json(ApiResponse::error(ErrorCode::OrderNotFound, e.to_string())),
    }
}

/// 撤单
async fn cancel_order(
    State(manager): State<Arc<ExchangeManager>>,
    Path((exchange_id, order_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<()>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.cancel_order(&order_id).await {
        Ok(_) => Json(ApiResponse::success_with_message((), "撤单成功")),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

#[derive(Deserialize)]
struct GetTradesQuery {
    start_time: Option<String>,
    end_time: Option<String>,
}

/// 查询成交记录
async fn get_trades(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Query(query): Query<GetTradesQuery>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Vec<Trade>>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_trades(query.start_time, query.end_time).await {
        Ok(trades) => Json(ApiResponse::success(trades)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

#[derive(Deserialize)]
struct GetInstrumentsQuery {
    #[serde(rename = "type")]
    instrument_type: Option<String>,
}

/// 查询合约列表
async fn get_instruments(
    State(manager): State<Arc<ExchangeManager>>,
    Path(exchange_id): Path<String>,
    Query(query): Query<GetInstrumentsQuery>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Vec<Instrument>>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_instruments(query.instrument_type).await {
        Ok(instruments) => Json(ApiResponse::success(instruments)),
        Err(e) => Json(ApiResponse::error(ErrorCode::InternalError, e.to_string())),
    }
}

/// 查询合约详情
async fn get_instrument(
    State(manager): State<Arc<ExchangeManager>>,
    Path((exchange_id, symbol)): Path<(String, String)>,
) -> impl IntoResponse {
    let adapter = match manager.get(&exchange_id) {
        Some(a) => a,
        None => {
            return Json(ApiResponse::<Instrument>::error(
                ErrorCode::ExchangeNotFound,
                "交易所不存在",
            ))
        }
    };

    match adapter.get_instrument(&symbol).await {
        Ok(instrument) => Json(ApiResponse::success(instrument)),
        Err(e) => Json(ApiResponse::error(ErrorCode::NotFound, e.to_string())),
    }
}
