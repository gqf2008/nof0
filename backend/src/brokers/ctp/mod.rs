// CTP (China Futures Market) broker implementation

pub mod adapter;
pub mod broker;
pub mod error_codes;
pub mod real_connection;
pub mod types;

// Real mode SPI implementations (only when ctp-real feature is enabled)
#[cfg(feature = "ctp-real")]
pub mod md_spi;

#[cfg(feature = "ctp-real")]
pub mod trader_spi;

// Web API (only when ctp-real feature is enabled)
#[cfg(feature = "ctp-real")]
pub mod web_api;

// Quick Web API - 简化版本，直接使用 Broker
#[cfg(feature = "ctp-real")]
pub mod quick_web_api;

pub use adapter::CtpMarketAdapter;
pub use broker::CtpBroker;
pub use real_connection::RealCtpConnection;
pub use types::*;

#[cfg(feature = "ctp-real")]
pub use web_api::{create_routes as create_ctp_routes, CtpConnectionManager};

#[cfg(feature = "ctp-real")]
pub use quick_web_api::create_quick_ctp_routes;
