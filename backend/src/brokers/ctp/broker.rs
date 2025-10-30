use crate::brokers::{AccountManagement, Analytics, Broker, MarketData, Trading};
use async_trait::async_trait;
use rand::Rng;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::adapter::CtpMarketAdapter;
use super::types::*;

/// CTP 经纪商实现
/// 基于 CTP (Comprehensive Transaction Platform) 柜台协议
/// 支持中国期货市场的交易和行情接口
#[allow(dead_code)]
pub struct CtpBroker {
    id: String,
    name: String,
    config: CtpConfig,
    adapter: CtpMarketAdapter,
}

#[allow(dead_code)]
impl CtpBroker {
    pub fn new(id: String, name: String, config: CtpConfig) -> Self {
        let adapter = CtpMarketAdapter::new(config.clone());
        Self {
            id,
            name,
            config,
            adapter,
        }
    }

    /// 获取 CTP 支持的期货合约列表
    fn get_instruments(&self) -> Vec<String> {
        vec![
            "IF2312".to_string(), // 沪深300股指期货
            "IC2312".to_string(), // 中证500股指期货
            "IH2312".to_string(), // 上证50股指期货
            "IM2312".to_string(), // 中证1000股指期货
            "rb2401".to_string(), // 螺纹钢
            "hc2401".to_string(), // 热轧卷板
            "i2401".to_string(),  // 铁矿石
            "au2312".to_string(), // 黄金
            "ag2312".to_string(), // 白银
            "cu2401".to_string(), // 铜
        ]
    }

    /// 获取合约的基准价格
    fn get_base_price(&self, instrument: &str) -> f64 {
        match instrument {
            s if s.starts_with("IF") => 4200.0,  // 沪深300
            s if s.starts_with("IC") => 6500.0,  // 中证500
            s if s.starts_with("IH") => 2800.0,  // 上证50
            s if s.starts_with("IM") => 5200.0,  // 中证1000
            s if s.starts_with("rb") => 3800.0,  // 螺纹钢
            s if s.starts_with("hc") => 3600.0,  // 热轧卷板
            s if s.starts_with("i") => 850.0,    // 铁矿石
            s if s.starts_with("au") => 480.0,   // 黄金
            s if s.starts_with("ag") => 5800.0,  // 白银
            s if s.starts_with("cu") => 68000.0, // 铜
            _ => 100.0,
        }
    }

    /// 获取模型配置（CTP 特定）
    fn get_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "ctp_model_trend".to_string(),
                name: "趋势追踪".to_string(),
                strategy: "Trend Following".to_string(),
                description: "基于移动平均线的趋势跟踪策略".to_string(),
                risk_level: "MEDIUM".to_string(),
                base_capital: 1000000.0,
            },
            ModelInfo {
                id: "ctp_model_arbitrage".to_string(),
                name: "跨品种套利".to_string(),
                strategy: "Statistical Arbitrage".to_string(),
                description: "利用相关品种间的价差进行套利".to_string(),
                risk_level: "LOW".to_string(),
                base_capital: 2000000.0,
            },
            ModelInfo {
                id: "ctp_model_momentum".to_string(),
                name: "动量突破".to_string(),
                strategy: "Momentum Breakout".to_string(),
                description: "捕捉短期动量突破机会".to_string(),
                risk_level: "HIGH".to_string(),
                base_capital: 800000.0,
            },
        ]
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct ModelInfo {
    id: String,
    name: String,
    strategy: String,
    description: String,
    risk_level: String,
    base_capital: f64,
}

// ============================================================================
// MarketData Trait Implementation (行情接口实现)
// ============================================================================
#[async_trait]
impl MarketData for CtpBroker {
    async fn get_prices(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let instruments = self.get_instruments();
        let mut prices = HashMap::new();
        let mut rng = rand::thread_rng();

        for instrument in instruments {
            let base_price = self.get_base_price(&instrument);
            let price = base_price * (1.0 + rng.gen_range(-0.02..0.02));
            prices.insert(instrument, price);
        }

        Ok(json!({
            "prices": prices
        }))
    }

    async fn get_orderbook(&self, symbol: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let base_price = self.get_base_price(symbol);

        let mut bids = vec![];
        let mut asks = vec![];

        // 生成5档买单
        for i in 0..5 {
            let price = base_price * (1.0 - 0.0001 * (i + 1) as f64);
            let quantity = rng.gen_range(10..100);
            bids.push(json!([price, quantity]));
        }

        // 生成5档卖单
        for i in 0..5 {
            let price = base_price * (1.0 + 0.0001 * (i + 1) as f64);
            let quantity = rng.gen_range(10..100);
            asks.push(json!([price, quantity]));
        }

        Ok(json!({
            "instrument_id": symbol,
            "bids": bids,
            "asks": asks,
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<i32>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let base_price = self.get_base_price(symbol);
        let limit = limit.unwrap_or(100);
        let mut klines = vec![];

        let mut current_price = base_price;
        let now = chrono::Utc::now().timestamp();

        // 根据时间间隔计算秒数
        let interval_seconds = match interval {
            "1m" => 60,
            "5m" => 300,
            "15m" => 900,
            "1h" => 3600,
            "1d" => 86400,
            _ => 60,
        };

        for i in (0..limit).rev() {
            let timestamp = now - (i as i64 * interval_seconds);
            let change = rng.gen_range(-0.015..0.015);
            current_price *= 1.0 + change;

            let high = current_price * rng.gen_range(1.0..1.008);
            let low = current_price * rng.gen_range(0.992..1.0);
            let volume = rng.gen_range(500..5000);
            let open_interest = rng.gen_range(10000..50000);

            klines.push(json!({
                "timestamp": timestamp,
                "open": current_price,
                "high": high,
                "low": low,
                "close": current_price,
                "volume": volume,
                "open_interest": open_interest
            }));
        }

        Ok(json!({
            "instrument_id": symbol,
            "interval": interval,
            "klines": klines
        }))
    }

    async fn get_ticker_24h(&self, symbol: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let price = self.get_base_price(symbol);
        let change_pct = rng.gen_range(-3.0..3.0);
        let volume = rng.gen_range(50000..500000);

        Ok(json!({
            "instrument_id": symbol,
            "last_price": price,
            "change_24h": change_pct,
            "high_24h": price * 1.02,
            "low_24h": price * 0.98,
            "volume_24h": volume,
            "open_interest": rng.gen_range(100000..1000000),
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }
}

// ============================================================================
// Trading Trait Implementation (交易接口实现)
// ============================================================================
#[async_trait]
impl Trading for CtpBroker {
    async fn place_order(&self, order: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let order_id = format!("CTP_{}", chrono::Utc::now().timestamp_millis());
        Ok(json!({
            "order_id": order_id,
            "status": "accepted",
            "exchange": "CTP",
            "timestamp": chrono::Utc::now().timestamp(),
            "order": order
        }))
    }

    async fn cancel_order(&self, order_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "order_id": order_id,
            "status": "cancelled",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn get_order(&self, order_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "order_id": order_id,
            "status": "filled",
            "filled_qty": 10,
            "avg_price": 4200.0,
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn get_orders(&self, _symbol: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "orders": []
        }))
    }

    async fn get_trades(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "trades": []
        }))
    }
}

// ============================================================================
// AccountManagement Trait Implementation (账户管理接口实现)
// ============================================================================
#[async_trait]
impl AccountManagement for CtpBroker {
    async fn get_account_totals(
        &self,
        _last_marker: Option<i32>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let now = chrono::Utc::now().timestamp();
        let models = self.get_models();
        let mut account_totals = vec![];

        // 生成过去120分钟的历史数据
        for model in &models {
            let base_value = model.base_capital;

            let (volatility, trend_bias) = match model.risk_level.as_str() {
                "LOW" => (0.0008, 0.0001),
                "MEDIUM" => (0.0015, 0.0),
                "HIGH" => (0.0025, -0.0001),
                _ => (0.0015, 0.0),
            };

            let mut current_value = base_value;

            for i in 0..120 {
                let minutes_ago = 119 - i;
                let timestamp = now - (minutes_ago * 60);

                let random_change = rng.gen_range(-volatility..volatility);
                let change_pct = random_change + trend_bias;
                current_value *= 1.0 + change_pct;

                let cumulative_return = (current_value - base_value) / base_value;
                let unrealized_pnl = current_value * rng.gen_range(-0.003..0.003);
                let dollar_equity = current_value + unrealized_pnl;

                account_totals.push(json!({
                    "model_id": &model.id,
                    "model_name": &model.name,
                    "strategy": &model.strategy,
                    "risk_level": &model.risk_level,
                    "broker_id": self.id,
                    "timestamp": timestamp,
                    "account_value": current_value,
                    "dollar_equity": dollar_equity,
                    "equity": dollar_equity,
                    "realized_pnl": current_value - base_value,
                    "unrealized_pnl": unrealized_pnl,
                    "total_unrealized_pnl": unrealized_pnl,
                    "return_pct": cumulative_return * 100.0,
                    "cum_pnl_pct": cumulative_return * 100.0,
                    "sharpe_ratio": if i > 30 {
                        if cumulative_return > 0.0 {
                            rng.gen_range(0.8..2.2)
                        } else {
                            rng.gen_range(-0.5..0.8)
                        }
                    } else {
                        0.0
                    },
                    "since_inception_minute_marker": i,
                    "since_inception_hourly_marker": i / 60,
                    "positions": {}
                }));
            }
        }

        Ok(json!({
            "accountTotals": account_totals
        }))
    }

    async fn get_model_accounts(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let now = chrono::Utc::now().timestamp();
        let models = self.get_models();
        let mut accounts = vec![];

        for model in models {
            let base_value = model.base_capital;

            let pnl_pct = match model.risk_level.as_str() {
                "LOW" => rng.gen_range(-0.02..0.05),
                "MEDIUM" => rng.gen_range(-0.04..0.08),
                "HIGH" => rng.gen_range(-0.06..0.12),
                _ => rng.gen_range(-0.04..0.08),
            };

            let realized_pnl = base_value * pnl_pct;
            let total_unrealized_pnl = base_value * rng.gen_range(-0.015..0.02);
            let dollar_equity = base_value + realized_pnl + total_unrealized_pnl;
            let return_pct = (realized_pnl / base_value) * 100.0;

            let (win_rate, trade_count) = match model.risk_level.as_str() {
                "LOW" => (rng.gen_range(0.55..0.68), rng.gen_range(50..120)),
                "MEDIUM" => (rng.gen_range(0.48..0.62), rng.gen_range(80..200)),
                "HIGH" => (rng.gen_range(0.42..0.58), rng.gen_range(150..400)),
                _ => (rng.gen_range(0.45..0.60), rng.gen_range(80..200)),
            };

            accounts.push(json!({
                "model_id": model.id,
                "model_name": model.name,
                "strategy": model.strategy,
                "risk_level": model.risk_level,
                "broker_id": self.id,
                "timestamp": now,
                "account_value": base_value,
                "dollar_equity": dollar_equity,
                "equity": dollar_equity,
                "realized_pnl": realized_pnl,
                "unrealized_pnl": total_unrealized_pnl,
                "total_unrealized_pnl": total_unrealized_pnl,
                "return_pct": return_pct,
                "cum_pnl_pct": return_pct,
                "sharpe_ratio": if return_pct > 0.0 { rng.gen_range(0.8..2.2) } else { rng.gen_range(-0.5..0.8) },
                "win_rate": win_rate,
                "total_trades": trade_count,
                "winning_trades": (trade_count as f64 * win_rate) as i32,
                "losing_trades": trade_count - (trade_count as f64 * win_rate) as i32,
                "positions": {}
            }));
        }

        Ok(json!({
            "accounts": accounts
        }))
    }

    async fn get_positions(
        &self,
        _limit: Option<i32>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let instruments = self.get_instruments();
        let mut rng = rand::thread_rng();
        let now = chrono::Utc::now().timestamp();
        let mut positions = HashMap::new();

        let num_positions = rng.gen_range(2..=4).min(instruments.len());

        for i in 0..num_positions {
            let instrument = &instruments[i];
            let entry_price = self.get_base_price(instrument);
            let current_price = entry_price * rng.gen_range(0.98..1.02);
            let quantity = rng.gen_range(5..50);
            let direction = if rng.gen_bool(0.5) { 1 } else { -1 };
            let signed_qty = quantity * direction;
            let unrealized_pnl = (current_price - entry_price) * quantity as f64 * direction as f64;

            positions.insert(
                instrument.clone(),
                json!({
                    "instrument_id": instrument,
                    "entry_price": entry_price,
                    "current_price": current_price,
                    "quantity": signed_qty,
                    "direction": if direction > 0 { "long" } else { "short" },
                    "unrealized_pnl": unrealized_pnl,
                    "margin": entry_price * quantity as f64 * 0.1, // 10% 保证金率
                    "timestamp": now,
                }),
            );
        }

        Ok(json!({
            "positions": positions
        }))
    }

    async fn get_balance(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        Ok(json!({
            "total_balance": rng.gen_range(1000000.0..5000000.0),
            "available": rng.gen_range(500000.0..2000000.0),
            "margin_used": rng.gen_range(300000.0..1000000.0),
            "frozen_margin": rng.gen_range(50000.0..200000.0),
            "currency": "CNY",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    async fn get_broker_account(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "broker_id": self.id,
            "broker_name": self.name,
            "broker_type": "futures",
            "protocol": "CTP",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }
}

// ============================================================================
// Analytics Trait Implementation (分析与统计接口实现)
// ============================================================================
#[async_trait]
impl Analytics for CtpBroker {
    async fn get_analytics(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "analytics": {}
        }))
    }

    async fn get_leaderboard(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "leaderboard": []
        }))
    }

    async fn get_since_inception_values(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "sinceInception": []
        }))
    }

    async fn get_conversations(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(json!({
            "conversations": []
        }))
    }

    async fn get_models_list(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let models = self.get_models();
        let models_json: Vec<Value> = models
            .iter()
            .map(|m| {
                json!({
                    "model_id": m.id,
                    "model_name": m.name,
                    "strategy": m.strategy,
                    "description": m.description,
                    "risk_level": m.risk_level,
                    "base_capital": m.base_capital,
                })
            })
            .collect();

        Ok(json!({
            "models": models_json
        }))
    }
}

// ============================================================================
// Broker Trait Implementation (组合特性实现)
// ============================================================================
impl Broker for CtpBroker {
    fn broker_id(&self) -> &str {
        &self.id
    }

    fn broker_name(&self) -> &str {
        &self.name
    }
}
