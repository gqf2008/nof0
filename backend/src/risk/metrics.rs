use std::collections::HashMap;

/// 风险指标
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    /// 账户余额
    pub account_balance: f64,

    /// 账户权益(余额 + 未实现盈亏)
    pub account_equity: f64,

    /// 今日盈亏
    pub daily_pnl: f64,

    /// 历史最高权益
    pub peak_equity: f64,

    /// 当前回撤比例
    pub current_drawdown: f64,

    /// 各品种持仓
    pub position_by_symbol: HashMap<String, PositionInfo>,

    /// 当前市场价格
    pub current_prices: HashMap<String, f64>,

    /// 今日已执行订单数
    pub daily_order_count: u32,

    /// 今日交易次数
    pub daily_trade_count: u32,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            account_balance: 0.0,
            account_equity: 0.0,
            daily_pnl: 0.0,
            peak_equity: 0.0,
            current_drawdown: 0.0,
            position_by_symbol: HashMap::new(),
            current_prices: HashMap::new(),
            daily_order_count: 0,
            daily_trade_count: 0,
        }
    }
}

impl RiskMetrics {
    /// 创建新的风险指标
    pub fn new(account_balance: f64) -> Self {
        Self {
            account_balance,
            account_equity: account_balance,
            peak_equity: account_balance,
            ..Default::default()
        }
    }

    /// 更新账户余额
    pub fn update_balance(&mut self, balance: f64, equity: f64) {
        self.account_balance = balance;
        self.account_equity = equity;

        // 更新历史最高权益
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        // 计算当前回撤
        if self.peak_equity > 0.0 {
            self.current_drawdown = (self.peak_equity - equity) / self.peak_equity;
        }
    }

    /// 更新持仓信息
    pub fn update_position(&mut self, symbol: String, position: PositionInfo) {
        if position.quantity == 0.0 {
            self.position_by_symbol.remove(&symbol);
        } else {
            self.position_by_symbol.insert(symbol, position);
        }
    }

    /// 更新市场价格
    pub fn update_price(&mut self, symbol: String, price: f64) {
        self.current_prices.insert(symbol, price);
    }

    /// 批量更新价格
    pub fn update_prices(&mut self, prices: HashMap<String, f64>) {
        self.current_prices.extend(prices);
    }

    /// 更新今日盈亏
    pub fn update_daily_pnl(&mut self, pnl: f64) {
        self.daily_pnl = pnl;
    }

    /// 增加订单计数
    pub fn increment_order_count(&mut self) {
        self.daily_order_count += 1;
    }

    /// 增加交易计数
    pub fn increment_trade_count(&mut self) {
        self.daily_trade_count += 1;
    }

    /// 重置每日统计(在新的一天开始时调用)
    pub fn reset_daily_stats(&mut self) {
        self.daily_pnl = 0.0;
        self.daily_order_count = 0;
        self.daily_trade_count = 0;
    }

    /// 计算总持仓价值
    pub fn total_position_value(&self) -> f64 {
        self.position_by_symbol
            .values()
            .map(|p| p.value_usd.abs())
            .sum()
    }

    /// 计算总未实现盈亏
    pub fn total_unrealized_pnl(&self) -> f64 {
        self.position_by_symbol
            .values()
            .map(|p| p.unrealized_pnl)
            .sum()
    }

    /// 计算杠杆倍数
    pub fn leverage(&self) -> f64 {
        if self.account_equity > 0.0 {
            self.total_position_value() / self.account_equity
        } else {
            0.0
        }
    }

    /// 计算保证金使用率
    pub fn margin_usage_ratio(&self) -> f64 {
        if self.account_balance > 0.0 {
            let used_margin = self.total_position_value();
            used_margin / self.account_balance
        } else {
            0.0
        }
    }

    /// 获取风险评分(0-100,分数越高风险越大)
    pub fn risk_score(&self) -> f64 {
        let mut score = 0.0;

        // 回撤因子(0-40分)
        score += self.current_drawdown * 200.0;

        // 杠杆因子(0-30分)
        let leverage = self.leverage();
        score += (leverage / 10.0) * 30.0;

        // 每日亏损因子(0-20分)
        if self.daily_pnl < 0.0 && self.account_balance > 0.0 {
            let loss_ratio = self.daily_pnl.abs() / self.account_balance;
            score += loss_ratio * 100.0;
        }

        // 保证金使用率因子(0-10分)
        score += self.margin_usage_ratio() * 10.0;

        score.min(100.0)
    }

    /// 获取风险等级描述
    pub fn risk_level_description(&self) -> &'static str {
        let score = self.risk_score();
        match score {
            s if s < 20.0 => "LOW - Safe to trade",
            s if s < 40.0 => "MEDIUM - Monitor closely",
            s if s < 60.0 => "HIGH - Reduce positions",
            s if s < 80.0 => "VERY HIGH - Stop trading recommended",
            _ => "CRITICAL - Immediate action required",
        }
    }
}

/// 持仓信息
#[derive(Debug, Clone)]
pub struct PositionInfo {
    /// 持仓数量(正数为多头,负数为空头)
    pub quantity: f64,

    /// 持仓价值(USD)
    pub value_usd: f64,

    /// 平均入场价格
    pub avg_entry_price: f64,

    /// 当前市场价格
    pub current_price: f64,

    /// 未实现盈亏
    pub unrealized_pnl: f64,

    /// 持仓成本
    pub cost_basis: f64,
}

impl PositionInfo {
    /// 创建新的持仓信息
    pub fn new(quantity: f64, avg_entry_price: f64, current_price: f64) -> Self {
        let value_usd = quantity.abs() * current_price;
        let cost_basis = quantity.abs() * avg_entry_price;
        let unrealized_pnl = if quantity > 0.0 {
            // 多头盈亏
            quantity * (current_price - avg_entry_price)
        } else {
            // 空头盈亏
            quantity.abs() * (avg_entry_price - current_price)
        };

        Self {
            quantity,
            value_usd,
            avg_entry_price,
            current_price,
            unrealized_pnl,
            cost_basis,
        }
    }

    /// 更新市场价格并重新计算盈亏
    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
        self.value_usd = self.quantity.abs() * price;

        self.unrealized_pnl = if self.quantity > 0.0 {
            self.quantity * (price - self.avg_entry_price)
        } else {
            self.quantity.abs() * (self.avg_entry_price - price)
        };
    }

    /// 是否为多头
    pub fn is_long(&self) -> bool {
        self.quantity > 0.0
    }

    /// 是否为空头
    pub fn is_short(&self) -> bool {
        self.quantity < 0.0
    }

    /// 盈亏比例
    pub fn pnl_ratio(&self) -> f64 {
        if self.cost_basis > 0.0 {
            self.unrealized_pnl / self.cost_basis
        } else {
            0.0
        }
    }
}
