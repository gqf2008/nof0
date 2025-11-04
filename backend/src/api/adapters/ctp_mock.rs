use anyhow::{anyhow, Result};
/// CTP Mock 适配器
///
/// 实现统一 API 接口,用于测试和演示
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::{traits::*, types::*};

/// CTP Mock 适配器
pub struct CtpMockAdapter {
    id: String,
    name: String,
    connected: Arc<RwLock<bool>>,
}

impl CtpMockAdapter {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            connected: Arc::new(RwLock::new(false)),
        }
    }
}

#[async_trait]
impl ExchangeAdapter for CtpMockAdapter {
    fn exchange_id(&self) -> &str {
        &self.id
    }

    fn exchange_name(&self) -> &str {
        &self.name
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Futures
    }

    async fn connect(&self) -> Result<()> {
        *self.connected.write().await = true;
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        *self.connected.write().await = false;
        Ok(())
    }

    async fn get_status(&self) -> Result<ConnectionStatus> {
        let connected = *self.connected.read().await;

        Ok(ConnectionStatus {
            exchange_id: self.id.clone(),
            connected,
            mode: "mock".to_string(),
            connections: ConnectionDetails {
                market_data: ConnectionState {
                    connected,
                    reconnecting: false,
                    reconnect_attempts: 0,
                },
                trading: ConnectionState {
                    connected,
                    reconnecting: false,
                    reconnect_attempts: 0,
                },
            },
            broker_info: Some(BrokerInfo {
                broker_id: "9999".to_string(),
                broker_name: "SimNow".to_string(),
            }),
        })
    }

    async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

#[async_trait]
impl MarketDataProvider for CtpMockAdapter {
    async fn subscribe_market(&self, symbols: Vec<String>) -> Result<()> {
        tracing::info!("🔔 订阅行情: {:?}", symbols);
        Ok(())
    }

    async fn unsubscribe_market(&self, symbols: Vec<String>) -> Result<()> {
        tracing::info!("🔕 取消订阅: {:?}", symbols);
        Ok(())
    }

    async fn get_ticker(&self, symbol: &str) -> Result<MarketTicker> {
        Ok(MarketTicker {
            symbol: symbol.to_string(),
            exchange_id: self.id.clone(),
            last_price: 3500.0,
            bid_price: 3498.0,
            ask_price: 3502.0,
            volume_24h: 125000.0,
            high_24h: 3550.0,
            low_24h: 3450.0,
            change_24h: 50.0,
            change_percent_24h: 1.45,
            timestamp: Utc::now().timestamp_millis() as u64,
            update_time: Utc::now().to_rfc3339(),
        })
    }

    async fn get_tickers(&self, symbols: Vec<String>) -> Result<Vec<MarketTicker>> {
        let mut tickers = Vec::new();
        for symbol in symbols {
            tickers.push(self.get_ticker(&symbol).await?);
        }
        Ok(tickers)
    }

    async fn get_klines(&self, symbol: &str, interval: &str, limit: usize) -> Result<Vec<Kline>> {
        let mut klines = Vec::new();
        let now = Utc::now().timestamp_millis() as u64;

        for i in 0..limit {
            let offset = (limit - i) as u64 * 60000; // 1分钟
            klines.push(Kline {
                symbol: symbol.to_string(),
                interval: interval.to_string(),
                open_time: now - offset,
                close_time: now - offset + 60000,
                open: 3500.0,
                high: 3520.0,
                low: 3480.0,
                close: 3510.0,
                volume: 1000.0,
                turnover: 3500000.0,
            });
        }

        Ok(klines)
    }
}

#[async_trait]
impl AccountProvider for CtpMockAdapter {
    async fn get_account(&self) -> Result<Account> {
        if !self.is_connected().await {
            return Err(anyhow!("未连接"));
        }

        Ok(Account {
            exchange_id: self.id.clone(),
            account_id: "123456".to_string(),
            balance: 1000000.0,
            available: 950000.0,
            frozen: 50000.0,
            equity: 1050000.0,
            margin: 80000.0,
            profit_loss: 50000.0,
            currency: "CNY".to_string(),
            update_time: Utc::now().to_rfc3339(),
        })
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        if !self.is_connected().await {
            return Err(anyhow!("未连接"));
        }

        Ok(vec![Position {
            symbol: "rb2501".to_string(),
            exchange_id: self.id.clone(),
            direction: PositionDirection::Long,
            volume: 10,
            available_volume: 8,
            frozen_volume: 2,
            avg_price: 3500.0,
            last_price: 3550.0,
            profit_loss: 5000.0,
            margin: 35000.0,
            open_time: Utc::now().to_rfc3339(),
            update_time: Utc::now().to_rfc3339(),
        }])
    }
}

#[async_trait]
impl TradingProvider for CtpMockAdapter {
    async fn place_order(&self, request: OrderRequest) -> Result<Order> {
        if !self.is_connected().await {
            return Err(anyhow!("未连接"));
        }

        Ok(Order {
            order_id: format!("2024110412{:08}", rand::random::<u32>()),
            client_order_id: "client_001".to_string(),
            symbol: request.symbol,
            exchange_id: self.id.clone(),
            direction: request.direction,
            offset: request.offset,
            price: request.price,
            volume: request.volume,
            filled_volume: 0,
            status: OrderStatus::Pending,
            submit_time: Utc::now().to_rfc3339(),
            update_time: None,
        })
    }

    async fn cancel_order(&self, order_id: &str) -> Result<()> {
        tracing::info!("撤单: {}", order_id);
        Ok(())
    }

    async fn get_order(&self, order_id: &str) -> Result<Order> {
        Ok(Order {
            order_id: order_id.to_string(),
            client_order_id: "client_001".to_string(),
            symbol: "rb2501".to_string(),
            exchange_id: self.id.clone(),
            direction: OrderDirection::Buy,
            offset: OffsetFlag::Open,
            price: 3500.0,
            volume: 5,
            filled_volume: 5,
            status: OrderStatus::Filled,
            submit_time: Utc::now().to_rfc3339(),
            update_time: Some(Utc::now().to_rfc3339()),
        })
    }

    async fn get_orders(&self, _status: Option<OrderStatus>, _limit: usize) -> Result<Vec<Order>> {
        Ok(vec![])
    }

    async fn get_trades(
        &self,
        _start_time: Option<String>,
        _end_time: Option<String>,
    ) -> Result<Vec<Trade>> {
        Ok(vec![])
    }
}

#[async_trait]
impl InstrumentProvider for CtpMockAdapter {
    async fn get_instruments(&self, _instrument_type: Option<String>) -> Result<Vec<Instrument>> {
        Ok(vec![Instrument {
            symbol: "rb2501".to_string(),
            name: "螺纹钢2501".to_string(),
            exchange: "SHFE".to_string(),
            exchange_id: self.id.clone(),
            instrument_type: "futures".to_string(),
            contract_size: 10,
            price_tick: 1.0,
            margin_ratio: 0.08,
            commission_ratio: 0.0001,
            trading_hours: vec!["09:00-15:00".to_string(), "21:00-23:00".to_string()],
            expire_date: Some("2025-01-15".to_string()),
        }])
    }

    async fn get_instrument(&self, symbol: &str) -> Result<Instrument> {
        Ok(Instrument {
            symbol: symbol.to_string(),
            name: format!("{} 合约", symbol),
            exchange: "SHFE".to_string(),
            exchange_id: self.id.clone(),
            instrument_type: "futures".to_string(),
            contract_size: 10,
            price_tick: 1.0,
            margin_ratio: 0.08,
            commission_ratio: 0.0001,
            trading_hours: vec!["09:00-15:00".to_string(), "21:00-23:00".to_string()],
            expire_date: Some("2025-01-15".to_string()),
        })
    }
}
