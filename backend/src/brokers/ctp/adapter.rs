use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::*;
use crate::markets::{Balance, MarketAdapter, Order, Price};

/// CTP市场适配器
pub struct CtpMarketAdapter {
    /// CTP配置
    config: CtpConfig,

    /// 行情数据缓存
    market_data: Arc<RwLock<HashMap<String, CtpMarketData>>>,

    /// 持仓信息缓存
    positions: Arc<RwLock<HashMap<String, CtpPosition>>>,

    /// 账户信息缓存
    account: Arc<RwLock<Option<CtpAccount>>>,

    /// 连接状态
    connected: Arc<RwLock<bool>>,

    /// 模拟模式下的订单计数器
    order_counter: Arc<RwLock<u64>>,
}

impl CtpMarketAdapter {
    /// 创建新的CTP适配器
    pub fn new(config: CtpConfig) -> Self {
        Self {
            config,
            market_data: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            account: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
            order_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// 连接到CTP服务器
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 验证配置
        self.config.validate()?;

        if self.config.mock_mode {
            // 模拟模式
            println!("🎭 CTP Mock Mode: Simulating connection...");
            self.init_mock_data().await;
            *self.connected.write().await = true;
            println!("✅ CTP Mock Mode: Connected successfully");
        } else {
            // 真实CTP连接
            // TODO: 集成 ctp2rs 库
            println!("🔌 CTP Real Mode: Connecting to CTP servers...");
            println!("   MD Address: {}", self.config.md_address);
            println!("   TD Address: {}", self.config.td_address);

            // 这里需要实际的CTP连接代码
            return Err("Real CTP mode not implemented yet. Please use mock_mode=true".into());
        }

        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        *self.connected.write().await = false;
        println!("🔌 CTP: Disconnected");
        Ok(())
    }

    /// 检查连接状态
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// 订阅行情
    pub async fn subscribe_market_data(
        &self,
        instruments: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_connected().await {
            return Err("Not connected to CTP".into());
        }

        if self.config.mock_mode {
            println!(
                "🎭 CTP Mock: Subscribing to {} instruments",
                instruments.len()
            );
            for instrument in instruments {
                println!("   📊 {}", instrument);
            }
        } else {
            // TODO: 真实行情订阅
        }

        Ok(())
    }

    /// 初始化模拟数据
    async fn init_mock_data(&self) {
        // 初始化模拟账户
        let account = CtpAccount {
            account_id: self.config.investor_id.clone(),
            available: 1000000.0,
            margin: 0.0,
            frozen_margin: 0.0,
            close_profit: 0.0,
            position_profit: 0.0,
            commission: 0.0,
            pre_balance: 1000000.0,
            balance: 1000000.0,
        };
        *self.account.write().await = Some(account);

        // 初始化一些模拟行情
        let mut market_data = self.market_data.write().await;

        // IF沪深300股指期货
        market_data.insert(
            "IF2501".to_string(),
            CtpMarketData {
                instrument_id: "IF2501".to_string(),
                last_price: 3500.0,
                bid_price: 3499.8,
                ask_price: 3500.2,
                bid_volume: 10,
                ask_volume: 15,
                volume: 125000,
                open_interest: 85000,
                highest_price: 3520.0,
                lowest_price: 3480.0,
                update_time: chrono::Local::now().format("%H:%M:%S").to_string(),
            },
        );

        // IC中证500股指期货
        market_data.insert(
            "IC2501".to_string(),
            CtpMarketData {
                instrument_id: "IC2501".to_string(),
                last_price: 5200.0,
                bid_price: 5199.6,
                ask_price: 5200.4,
                bid_volume: 8,
                ask_volume: 12,
                volume: 98000,
                open_interest: 72000,
                highest_price: 5220.0,
                lowest_price: 5180.0,
                update_time: chrono::Local::now().format("%H:%M:%S").to_string(),
            },
        );

        // IH上证50股指期货
        market_data.insert(
            "IH2501".to_string(),
            CtpMarketData {
                instrument_id: "IH2501".to_string(),
                last_price: 2400.0,
                bid_price: 2399.8,
                ask_price: 2400.2,
                bid_volume: 12,
                ask_volume: 10,
                volume: 78000,
                open_interest: 56000,
                highest_price: 2415.0,
                lowest_price: 2385.0,
                update_time: chrono::Local::now().format("%H:%M:%S").to_string(),
            },
        );
    }

    /// 模拟下单
    async fn place_order_mock(
        &self,
        request: CtpOrderRequest,
    ) -> Result<CtpOrderResponse, Box<dyn std::error::Error>> {
        // 增加订单计数
        let mut counter = self.order_counter.write().await;
        *counter += 1;
        let order_ref = format!("{:08}", *counter);
        let order_sys_id = format!("MOCK{}", order_ref);

        println!("\n🎭 CTP Mock: Placing order");
        println!("   Instrument: {}", request.instrument_id);
        println!(
            "   Direction: {}",
            if request.direction == '0' {
                "Buy"
            } else {
                "Sell"
            }
        );
        println!(
            "   Offset: {}",
            match request.offset_flag {
                '0' => "Open",
                '1' => "Close",
                '3' => "CloseToday",
                _ => "Unknown",
            }
        );
        println!("   Price: {:.2}", request.price);
        println!("   Volume: {}", request.volume);
        println!("   Order Ref: {}", order_ref);

        // 模拟订单成交
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(CtpOrderResponse {
            order_sys_id,
            order_ref,
            instrument_id: request.instrument_id,
            order_status: CtpOrderStatus::AllTraded,
            status_msg: "Mock order filled".to_string(),
        })
    }

    /// 获取持仓信息
    pub async fn query_position(&self) -> Result<Vec<CtpPosition>, Box<dyn std::error::Error>> {
        if !self.is_connected().await {
            return Err("Not connected to CTP".into());
        }

        let positions = self.positions.read().await;
        Ok(positions.values().cloned().collect())
    }

    /// 获取账户信息
    pub async fn query_account(&self) -> Result<CtpAccount, Box<dyn std::error::Error>> {
        if !self.is_connected().await {
            return Err("Not connected to CTP".into());
        }

        let account = self.account.read().await;
        account
            .clone()
            .ok_or_else(|| "Account info not available".into())
    }

    /// 获取行情数据
    pub async fn get_market_data(
        &self,
        instrument_id: &str,
    ) -> Result<CtpMarketData, Box<dyn std::error::Error>> {
        let market_data = self.market_data.read().await;
        market_data
            .get(instrument_id)
            .cloned()
            .ok_or_else(|| format!("Market data not found for {}", instrument_id).into())
    }
}

#[async_trait]
impl MarketAdapter for CtpMarketAdapter {
    async fn get_price(&self, symbol: &str) -> Result<Price, anyhow::Error> {
        let market_data = self
            .get_market_data(symbol)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(Price {
            symbol: symbol.to_string(),
            price: market_data.last_price,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn place_order(&self, order: Order) -> Result<String, anyhow::Error> {
        if !self.is_connected().await {
            return Err(anyhow::anyhow!("Not connected to CTP"));
        }

        // 将通用Order转换为CTP订单请求
        let direction = match order.side {
            crate::markets::OrderSide::Buy => '0',
            crate::markets::OrderSide::Sell => '1',
        };

        // 默认开仓
        let offset_flag = '0';

        let price = order.price.unwrap_or(0.0);
        let price_type = match order.order_type {
            crate::markets::OrderType::Limit => '2',
            crate::markets::OrderType::Market => '1',
        };

        let ctp_request = CtpOrderRequest {
            instrument_id: order.symbol.clone(),
            direction,
            offset_flag,
            price,
            volume: order.quantity as i32,
            price_type,
            hedge_flag: '1', // 投机
        };

        let response = if self.config.mock_mode {
            self.place_order_mock(ctp_request)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?
        } else {
            // TODO: 真实CTP下单
            return Err(anyhow::anyhow!("Real CTP mode not implemented yet"));
        };

        println!("✅ Order placed successfully: {}", response.order_sys_id);
        Ok(response.order_sys_id)
    }

    async fn get_balance(&self, _account_id: &str) -> Result<Vec<Balance>, anyhow::Error> {
        let account = self
            .query_account()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(vec![Balance {
            asset: "CNY".to_string(), // 人民币
            free: account.available,
            locked: account.frozen_margin,
        }])
    }

    fn market_name(&self) -> &str {
        if self.config.mock_mode {
            "CTP (Mock)"
        } else {
            "CTP"
        }
    }
}
