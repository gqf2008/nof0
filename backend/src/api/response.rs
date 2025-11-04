/// 统一 API 响应格式
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 标准 API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 操作是否成功
    pub success: bool,

    /// 业务状态码
    pub code: i32,

    /// 提示信息
    pub message: String,

    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// 时间戳(毫秒)
    pub timestamp: u64,

    /// 请求追踪ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            code: ErrorCode::Success as i32,
            message: "success".to_string(),
            data: Some(data),
            timestamp: current_timestamp_millis(),
            request_id: None,
        }
    }

    /// 创建成功响应(带消息)
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            code: ErrorCode::Success as i32,
            message: message.into(),
            data: Some(data),
            timestamp: current_timestamp_millis(),
            request_id: None,
        }
    }

    /// 创建错误响应
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            success: false,
            code: code as i32,
            message: message.into(),
            data: None,
            timestamp: current_timestamp_millis(),
            request_id: None,
        }
    }

    /// 设置请求ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

/// 业务错误码
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Success = 0,

    // 通用错误 1xxx
    InvalidParams = 1001,
    Unauthorized = 1002,
    NotFound = 1003,
    MethodNotAllowed = 1004,
    TooManyRequests = 1005,

    // 交易所相关 2xxx
    ExchangeOffline = 2001,
    ExchangeNotFound = 2002,
    ConnectionFailed = 2003,
    AlreadyConnected = 2004,
    NotConnected = 2005,

    // 交易相关 3xxx
    InsufficientBalance = 3001,
    OrderFailed = 3002,
    OrderNotFound = 3003,
    InvalidSymbol = 3004,
    InvalidPrice = 3005,
    InvalidVolume = 3006,
    MarketClosed = 3007,

    // 系统错误 5xxx
    InternalError = 5000,
    ServiceUnavailable = 5001,
    DatabaseError = 5002,
    NetworkError = 5003,
}

impl ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::Success => "成功",
            ErrorCode::InvalidParams => "参数错误",
            ErrorCode::Unauthorized => "未授权",
            ErrorCode::NotFound => "资源不存在",
            ErrorCode::MethodNotAllowed => "方法不允许",
            ErrorCode::TooManyRequests => "请求过于频繁",

            ErrorCode::ExchangeOffline => "交易所离线",
            ErrorCode::ExchangeNotFound => "交易所不存在",
            ErrorCode::ConnectionFailed => "连接失败",
            ErrorCode::AlreadyConnected => "已经连接",
            ErrorCode::NotConnected => "未连接",

            ErrorCode::InsufficientBalance => "余额不足",
            ErrorCode::OrderFailed => "下单失败",
            ErrorCode::OrderNotFound => "订单不存在",
            ErrorCode::InvalidSymbol => "合约代码无效",
            ErrorCode::InvalidPrice => "价格无效",
            ErrorCode::InvalidVolume => "数量无效",
            ErrorCode::MarketClosed => "市场已关闭",

            ErrorCode::InternalError => "内部错误",
            ErrorCode::ServiceUnavailable => "服务不可用",
            ErrorCode::DatabaseError => "数据库错误",
            ErrorCode::NetworkError => "网络错误",
        }
    }
}

/// 获取当前时间戳(毫秒)
fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let resp = ApiResponse::success(vec!["test"]);
        assert!(resp.success);
        assert_eq!(resp.code, 0);
        assert_eq!(resp.data, Some(vec!["test"]));
    }

    #[test]
    fn test_error_response() {
        let resp: ApiResponse<()> = ApiResponse::error(ErrorCode::InvalidParams, "参数错误");
        assert!(!resp.success);
        assert_eq!(resp.code, 1001);
        assert_eq!(resp.data, None);
    }
}
