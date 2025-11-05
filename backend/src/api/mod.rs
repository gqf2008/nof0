pub mod adapters;
pub mod middleware;
pub mod response;
pub mod router;
pub mod traits;
/// 统一交易 API 模块
///
/// 提供标准化的交易接口,支持多交易所统一调用
pub mod types;

pub use response::{ApiResponse, ErrorCode};
pub use router::{create_unified_routes, ExchangeManager};
pub use traits::{AccountProvider, ExchangeAdapter, MarketDataProvider, TradingProvider};
pub use types::*;
