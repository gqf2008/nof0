use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{
    config::RiskConfig,
    metrics::{PositionInfo, RiskMetrics},
    rules::*,
    OrderInfo, RiskCheckResult, RiskEvent, RiskEventType,
};

/// 风险管理器
pub struct RiskManager {
    /// 风险配置
    config: Arc<RwLock<RiskConfig>>,

    /// 风险指标
    metrics: Arc<RwLock<RiskMetrics>>,

    /// 风险规则列表
    rules: Arc<RwLock<Vec<Arc<dyn RiskRule>>>>,

    /// 数据库连接池(用于记录风险事件)
    pool: Option<Arc<PgPool>>,

    /// 风险事件历史(内存缓存)
    event_history: Arc<RwLock<Vec<RiskEvent>>>,
}

impl RiskManager {
    /// 创建新的风险管理器
    pub fn new(config: RiskConfig) -> Self {
        let metrics = RiskMetrics::new(100000.0); // 默认10万美元余额
        let config_arc = Arc::new(RwLock::new(config.clone()));

        // 初始化所有风险规则
        let rules: Vec<Arc<dyn RiskRule>> = vec![
            Arc::new(DailyLossLimitRule::new(config.clone())),
            Arc::new(MaxDrawdownRule::new(config.clone())),
            Arc::new(PositionLimitRule::new(config.clone())),
            Arc::new(OrderSizeRule::new(config.clone())),
            Arc::new(TradingFrequencyRule::new(config.clone())),
        ];

        Self {
            config: config_arc,
            metrics: Arc::new(RwLock::new(metrics)),
            rules: Arc::new(RwLock::new(rules)),
            pool: None,
            event_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 设置数据库连接池
    pub fn with_pool(mut self, pool: Arc<PgPool>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// 更新风险配置
    pub async fn update_config(&self, config: RiskConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config.clone();

        // 重新初始化规则
        let new_rules: Vec<Arc<dyn RiskRule>> = vec![
            Arc::new(DailyLossLimitRule::new(config.clone())),
            Arc::new(MaxDrawdownRule::new(config.clone())),
            Arc::new(PositionLimitRule::new(config.clone())),
            Arc::new(OrderSizeRule::new(config.clone())),
            Arc::new(TradingFrequencyRule::new(config.clone())),
        ];

        let mut rules = self.rules.write().await;
        *rules = new_rules;
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> RiskConfig {
        self.config.read().await.clone()
    }

    /// 获取当前风险指标
    pub async fn get_metrics(&self) -> RiskMetrics {
        self.metrics.read().await.clone()
    }

    /// 更新账户余额
    pub async fn update_balance(&self, balance: f64, equity: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.update_balance(balance, equity);
    }

    /// 更新持仓
    pub async fn update_position(&self, symbol: String, position: PositionInfo) {
        let mut metrics = self.metrics.write().await;
        metrics.update_position(symbol, position);
    }

    /// 更新市场价格
    pub async fn update_price(&self, symbol: String, price: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.update_price(symbol, price);
    }

    /// 批量更新价格
    pub async fn update_prices(&self, prices: HashMap<String, f64>) {
        let mut metrics = self.metrics.write().await;
        metrics.update_prices(prices);
    }

    /// 更新今日盈亏
    pub async fn update_daily_pnl(&self, pnl: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.update_daily_pnl(pnl);
    }

    /// 重置每日统计
    pub async fn reset_daily_stats(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.reset_daily_stats();
    }

    /// 验证订单
    pub async fn validate_order(
        &self,
        order: &OrderInfo,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let config = self.config.read().await;
        if !config.enabled {
            return Ok(ValidationResult {
                approved: true,
                warnings: vec![],
                rejections: vec![],
            });
        }

        let metrics = self.metrics.read().await.clone();
        let rules = self.rules.read().await;

        let mut warnings = Vec::new();
        let mut rejections = Vec::new();

        // 按优先级排序规则
        let mut sorted_rules: Vec<_> = rules.iter().collect();
        sorted_rules.sort_by_key(|r| r.priority());

        // 执行所有规则检查
        for rule in sorted_rules {
            let result = rule.check_order(order, &metrics).await?;

            if !result.passed {
                // 记录风险事件
                let event = RiskEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: RiskEventType::OrderRejected,
                    rule_name: result.rule_name.clone(),
                    risk_level: result.risk_level,
                    description: result.reason.clone().unwrap_or_default(),
                    order_info: Some(order.clone()),
                };
                self.record_event(event).await?;

                rejections.push(result);
            } else if result.reason.is_some() {
                // 有警告信息
                warnings.push(result);
            }
        }

        let approved = rejections.is_empty();

        // 如果订单通过,记录订单
        if approved {
            self.record_order_approved(order).await?;
        }

        Ok(ValidationResult {
            approved,
            warnings,
            rejections,
        })
    }

    /// 记录订单已批准
    async fn record_order_approved(
        &self,
        _order: &OrderInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut metrics = self.metrics.write().await;
        metrics.increment_order_count();

        // 更新交易频率规则的订单历史
        // 注意:由于 trait object 的限制,我们使用规则名称来判断
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if rule.name() == "TradingFrequencyRule" {
                // 在实际实现中,TradingFrequencyRule 需要提供公共方法来记录订单
                // 这里我们简化处理,假设规则内部会自动跟踪
                // 或者可以使用 Arc<RwLock<Vec<DateTime>>> 共享订单历史
            }
        }

        Ok(())
    }

    /// 记录风险事件
    async fn record_event(&self, event: RiskEvent) -> Result<(), Box<dyn std::error::Error>> {
        // 添加到内存缓存
        let mut history = self.event_history.write().await;
        history.push(event.clone());

        // 只保留最近1000条
        let history_len = history.len();
        if history_len > 1000 {
            history.drain(0..history_len - 1000);
        }

        // 如果有数据库连接,保存到数据库
        if let Some(pool) = &self.pool {
            self.save_event_to_db(pool, &event).await?;
        }

        Ok(())
    }

    /// 保存风险事件到数据库
    async fn save_event_to_db(
        &self,
        pool: &PgPool,
        event: &RiskEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let order_symbol = event.order_info.as_ref().map(|o| o.symbol.as_str());
        let order_side = event.order_info.as_ref().map(|o| o.side.as_str());
        let order_quantity = event.order_info.as_ref().map(|o| o.quantity);
        let order_price = event.order_info.as_ref().and_then(|o| o.price);

        sqlx::query(
            r#"
            INSERT INTO risk_events 
            (timestamp, event_type, rule_name, risk_level, description, 
             order_symbol, order_side, order_quantity, order_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(event.timestamp)
        .bind(event.event_type.to_string())
        .bind(&event.rule_name)
        .bind(event.risk_level.to_string())
        .bind(&event.description)
        .bind(order_symbol)
        .bind(order_side)
        .bind(order_quantity)
        .bind(order_price)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 获取风险事件历史
    pub async fn get_event_history(&self, limit: usize) -> Vec<RiskEvent> {
        let history = self.event_history.read().await;
        let start = history.len().saturating_sub(limit);
        history[start..].to_vec()
    }

    /// 获取风险评分
    pub async fn get_risk_score(&self) -> f64 {
        let metrics = self.metrics.read().await;
        metrics.risk_score()
    }

    /// 获取风险等级描述
    pub async fn get_risk_level_description(&self) -> String {
        let metrics = self.metrics.read().await;
        metrics.risk_level_description().to_string()
    }

    /// 生成风险报告
    pub async fn generate_risk_report(&self) -> RiskReport {
        let metrics = self.metrics.read().await;
        let config = self.config.read().await;
        let recent_events = self.get_event_history(10).await;

        RiskReport {
            timestamp: chrono::Utc::now(),
            account_balance: metrics.account_balance,
            account_equity: metrics.account_equity,
            daily_pnl: metrics.daily_pnl,
            current_drawdown: metrics.current_drawdown,
            leverage: metrics.leverage(),
            margin_usage: metrics.margin_usage_ratio(),
            risk_score: metrics.risk_score(),
            risk_level: metrics.risk_level_description().to_string(),
            total_position_value: metrics.total_position_value(),
            position_count: metrics.position_by_symbol.len(),
            daily_order_count: metrics.daily_order_count,
            config: config.clone(),
            recent_events,
        }
    }
}

/// 订单验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否批准订单
    pub approved: bool,

    /// 警告信息(不阻止订单)
    pub warnings: Vec<RiskCheckResult>,

    /// 拒绝原因(阻止订单)
    pub rejections: Vec<RiskCheckResult>,
}

impl ValidationResult {
    /// 获取所有消息(警告+拒绝)
    pub fn all_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();

        for warning in &self.warnings {
            if let Some(reason) = &warning.reason {
                messages.push(format!("⚠️  {}: {}", warning.rule_name, reason));
            }
        }

        for rejection in &self.rejections {
            if let Some(reason) = &rejection.reason {
                messages.push(format!("❌ {}: {}", rejection.rule_name, reason));
            }
        }

        messages
    }
}

/// 风险报告
#[derive(Debug, Clone)]
pub struct RiskReport {
    /// 报告时间
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// 账户余额
    pub account_balance: f64,

    /// 账户权益
    pub account_equity: f64,

    /// 今日盈亏
    pub daily_pnl: f64,

    /// 当前回撤
    pub current_drawdown: f64,

    /// 杠杆倍数
    pub leverage: f64,

    /// 保证金使用率
    pub margin_usage: f64,

    /// 风险评分
    pub risk_score: f64,

    /// 风险等级
    pub risk_level: String,

    /// 总持仓价值
    pub total_position_value: f64,

    /// 持仓品种数
    pub position_count: usize,

    /// 今日订单数
    pub daily_order_count: u32,

    /// 风险配置
    pub config: RiskConfig,

    /// 最近的风险事件
    pub recent_events: Vec<RiskEvent>,
}

impl RiskReport {
    /// 格式化为文本报告
    pub fn to_text(&self) -> String {
        format!(
            r#"
=== Risk Management Report ===
Timestamp: {}

Account Status:
  Balance: ${:.2}
  Equity: ${:.2}
  Daily P&L: ${:.2} ({:.2}%)
  
Risk Metrics:
  Risk Score: {:.1}/100 - {}
  Drawdown: {:.2}%
  Leverage: {:.2}x
  Margin Usage: {:.2}%
  
Positions:
  Total Value: ${:.2}
  Number of Positions: {}
  
Activity:
  Today's Orders: {}
  
Recent Events:
{}

Configuration Limits:
  Max Position/Symbol: ${:.2}
  Max Total Position: ${:.2}
  Max Daily Loss: ${:.2}
  Max Drawdown: {:.2}%
  Max Leverage: {:.2}x
"#,
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.account_balance,
            self.account_equity,
            self.daily_pnl,
            if self.account_balance > 0.0 {
                (self.daily_pnl / self.account_balance) * 100.0
            } else {
                0.0
            },
            self.risk_score,
            self.risk_level,
            self.current_drawdown * 100.0,
            self.leverage,
            self.margin_usage * 100.0,
            self.total_position_value,
            self.position_count,
            self.daily_order_count,
            self.format_recent_events(),
            self.config.position_limits.max_position_per_symbol,
            self.config.position_limits.max_total_position,
            self.config.loss_limits.max_daily_loss,
            self.config.loss_limits.max_drawdown * 100.0,
            self.config.position_limits.max_leverage,
        )
    }

    fn format_recent_events(&self) -> String {
        if self.recent_events.is_empty() {
            return "  No recent events".to_string();
        }

        self.recent_events
            .iter()
            .map(|e| {
                format!(
                    "  [{} {}] {} - {}",
                    e.timestamp.format("%H:%M:%S"),
                    e.risk_level,
                    e.rule_name,
                    e.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
