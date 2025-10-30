use async_trait::async_trait;
use std::sync::Arc;

use super::{OrderInfo, RiskCheckResult, RiskConfig, RiskLevel, RiskMetrics};

/// 风险规则 trait
#[async_trait]
pub trait RiskRule: Send + Sync {
    /// 规则名称
    fn name(&self) -> &str;

    /// 检查订单是否符合风险规则
    async fn check_order(
        &self,
        order: &OrderInfo,
        metrics: &RiskMetrics,
    ) -> Result<RiskCheckResult, Box<dyn std::error::Error>>;

    /// 规则优先级(数字越小优先级越高)
    fn priority(&self) -> u32 {
        100
    }
}

/// 仓位限制规则
pub struct PositionLimitRule {
    config: RiskConfig,
}

impl PositionLimitRule {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl RiskRule for PositionLimitRule {
    fn name(&self) -> &str {
        "PositionLimitRule"
    }

    fn priority(&self) -> u32 {
        10 // 高优先级
    }

    async fn check_order(
        &self,
        order: &OrderInfo,
        metrics: &RiskMetrics,
    ) -> Result<RiskCheckResult, Box<dyn std::error::Error>> {
        let limits = &self.config.position_limits;

        // 检查单品种仓位限制
        let current_position = metrics
            .position_by_symbol
            .get(&order.symbol)
            .map(|p| p.value_usd)
            .unwrap_or(0.0);

        let order_value = order.quantity
            * order.price.unwrap_or(
                metrics
                    .current_prices
                    .get(&order.symbol)
                    .copied()
                    .unwrap_or(0.0),
            );
        let new_position = current_position.abs() + order_value;

        if new_position > limits.max_position_per_symbol {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Position limit exceeded for {}: current={:.2}, order={:.2}, limit={:.2}",
                    order.symbol, current_position, order_value, limits.max_position_per_symbol
                ),
                RiskLevel::High,
            ));
        }

        // 检查总仓位限制
        let total_position = metrics.total_position_value();
        let new_total = total_position + order_value;

        if new_total > limits.max_total_position {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Total position limit exceeded: current={:.2}, order={:.2}, limit={:.2}",
                    total_position, order_value, limits.max_total_position
                ),
                RiskLevel::High,
            ));
        }

        // 检查仓位比例
        let account_balance = metrics.account_balance;
        if account_balance > 0.0 {
            let position_ratio = new_position / account_balance;
            if position_ratio > limits.max_position_ratio {
                return Ok(RiskCheckResult::fail(
                    self.name(),
                    format!(
                        "Position ratio exceeded for {}: ratio={:.2}%, limit={:.2}%",
                        order.symbol,
                        position_ratio * 100.0,
                        limits.max_position_ratio * 100.0
                    ),
                    RiskLevel::Medium,
                ));
            }
        }

        Ok(RiskCheckResult::pass(self.name()))
    }
}

/// 每日亏损限制规则
pub struct DailyLossLimitRule {
    config: RiskConfig,
}

impl DailyLossLimitRule {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl RiskRule for DailyLossLimitRule {
    fn name(&self) -> &str {
        "DailyLossLimitRule"
    }

    fn priority(&self) -> u32 {
        5 // 最高优先级
    }

    async fn check_order(
        &self,
        _order: &OrderInfo,
        metrics: &RiskMetrics,
    ) -> Result<RiskCheckResult, Box<dyn std::error::Error>> {
        let limits = &self.config.loss_limits;
        let daily_pnl = metrics.daily_pnl;

        // 如果已经亏损超过限制,禁止新订单
        if daily_pnl < 0.0 && daily_pnl.abs() >= limits.max_daily_loss {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Daily loss limit reached: loss={:.2}, limit={:.2}",
                    daily_pnl.abs(),
                    limits.max_daily_loss
                ),
                RiskLevel::Critical,
            ));
        }

        // 警告:接近每日亏损限制
        if daily_pnl < 0.0 && daily_pnl.abs() >= limits.max_daily_loss * 0.8 {
            return Ok(RiskCheckResult {
                passed: true,
                reason: Some(format!(
                    "Warning: Approaching daily loss limit ({:.2}/{:.2})",
                    daily_pnl.abs(),
                    limits.max_daily_loss
                )),
                rule_name: self.name().to_string(),
                risk_level: RiskLevel::Medium,
            });
        }

        Ok(RiskCheckResult::pass(self.name()))
    }
}

/// 最大回撤限制规则
pub struct MaxDrawdownRule {
    config: RiskConfig,
}

impl MaxDrawdownRule {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl RiskRule for MaxDrawdownRule {
    fn name(&self) -> &str {
        "MaxDrawdownRule"
    }

    fn priority(&self) -> u32 {
        5 // 最高优先级
    }

    async fn check_order(
        &self,
        _order: &OrderInfo,
        metrics: &RiskMetrics,
    ) -> Result<RiskCheckResult, Box<dyn std::error::Error>> {
        let limits = &self.config.loss_limits;
        let drawdown = metrics.current_drawdown;

        if drawdown >= limits.max_drawdown {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Max drawdown exceeded: current={:.2}%, limit={:.2}%",
                    drawdown * 100.0,
                    limits.max_drawdown * 100.0
                ),
                RiskLevel::Critical,
            ));
        }

        // 警告:接近最大回撤
        if drawdown >= limits.max_drawdown * 0.8 {
            return Ok(RiskCheckResult {
                passed: true,
                reason: Some(format!(
                    "Warning: Approaching max drawdown ({:.2}%/{:.2}%)",
                    drawdown * 100.0,
                    limits.max_drawdown * 100.0
                )),
                rule_name: self.name().to_string(),
                risk_level: RiskLevel::High,
            });
        }

        Ok(RiskCheckResult::pass(self.name()))
    }
}

/// 交易频率限制规则
pub struct TradingFrequencyRule {
    config: RiskConfig,
    order_history: Arc<tokio::sync::RwLock<Vec<chrono::DateTime<chrono::Utc>>>>,
}

impl TradingFrequencyRule {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            order_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// 记录订单时间
    pub async fn record_order(&self, timestamp: chrono::DateTime<chrono::Utc>) {
        let mut history = self.order_history.write().await;
        history.push(timestamp);

        // 只保留最近24小时的记录
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        history.retain(|&t| t > cutoff);
    }
}

#[async_trait]
impl RiskRule for TradingFrequencyRule {
    fn name(&self) -> &str {
        "TradingFrequencyRule"
    }

    fn priority(&self) -> u32 {
        20
    }

    async fn check_order(
        &self,
        _order: &OrderInfo,
        _metrics: &RiskMetrics,
    ) -> Result<RiskCheckResult, Box<dyn std::error::Error>> {
        let limits = &self.config.frequency_limits;
        let now = chrono::Utc::now();
        let history = self.order_history.read().await;

        // 检查最小订单间隔
        if let Some(&last_order_time) = history.last() {
            let interval = (now - last_order_time).num_seconds();
            if interval < limits.min_order_interval_secs as i64 {
                return Ok(RiskCheckResult::fail(
                    self.name(),
                    format!(
                        "Order interval too short: {}s, minimum: {}s",
                        interval, limits.min_order_interval_secs
                    ),
                    RiskLevel::Medium,
                ));
            }
        }

        // 检查每分钟订单数
        let one_minute_ago = now - chrono::Duration::minutes(1);
        let orders_last_minute = history.iter().filter(|&&t| t > one_minute_ago).count();
        if orders_last_minute >= limits.max_orders_per_minute as usize {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Too many orders in last minute: {}/{}",
                    orders_last_minute, limits.max_orders_per_minute
                ),
                RiskLevel::High,
            ));
        }

        // 检查每小时订单数
        let one_hour_ago = now - chrono::Duration::hours(1);
        let orders_last_hour = history.iter().filter(|&&t| t > one_hour_ago).count();
        if orders_last_hour >= limits.max_orders_per_hour as usize {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Too many orders in last hour: {}/{}",
                    orders_last_hour, limits.max_orders_per_hour
                ),
                RiskLevel::High,
            ));
        }

        // 检查每天订单数
        let one_day_ago = now - chrono::Duration::hours(24);
        let orders_last_day = history.iter().filter(|&&t| t > one_day_ago).count();
        if orders_last_day >= limits.max_orders_per_day as usize {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Too many orders in last 24 hours: {}/{}",
                    orders_last_day, limits.max_orders_per_day
                ),
                RiskLevel::Critical,
            ));
        }

        Ok(RiskCheckResult::pass(self.name()))
    }
}

/// 订单大小限制规则
pub struct OrderSizeRule {
    config: RiskConfig,
}

impl OrderSizeRule {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl RiskRule for OrderSizeRule {
    fn name(&self) -> &str {
        "OrderSizeRule"
    }

    fn priority(&self) -> u32 {
        15
    }

    async fn check_order(
        &self,
        order: &OrderInfo,
        metrics: &RiskMetrics,
    ) -> Result<RiskCheckResult, Box<dyn std::error::Error>> {
        let limits = &self.config.order_size_limits;

        // 计算订单价值
        let price = order.price.unwrap_or_else(|| {
            metrics
                .current_prices
                .get(&order.symbol)
                .copied()
                .unwrap_or(0.0)
        });
        let order_value = order.quantity * price;

        // 检查最小订单金额
        if order_value < limits.min_order_value {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Order value too small: {:.2}, minimum: {:.2}",
                    order_value, limits.min_order_value
                ),
                RiskLevel::Low,
            ));
        }

        // 检查最大订单金额
        if order_value > limits.max_order_value {
            return Ok(RiskCheckResult::fail(
                self.name(),
                format!(
                    "Order value too large: {:.2}, maximum: {:.2}",
                    order_value, limits.max_order_value
                ),
                RiskLevel::High,
            ));
        }

        // 检查订单占用资金比例
        let account_balance = metrics.account_balance;
        if account_balance > 0.0 {
            let order_ratio = order_value / account_balance;
            if order_ratio > limits.max_order_ratio {
                return Ok(RiskCheckResult::fail(
                    self.name(),
                    format!(
                        "Order size ratio too large: {:.2}%, maximum: {:.2}%",
                        order_ratio * 100.0,
                        limits.max_order_ratio * 100.0
                    ),
                    RiskLevel::High,
                ));
            }
        }

        Ok(RiskCheckResult::pass(self.name()))
    }
}
