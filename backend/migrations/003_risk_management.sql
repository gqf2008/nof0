-- 风险事件表
CREATE TABLE IF NOT EXISTS risk_events (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type VARCHAR(50) NOT NULL,
    rule_name VARCHAR(100) NOT NULL,
    risk_level VARCHAR(20) NOT NULL,
    description TEXT NOT NULL,
    order_symbol VARCHAR(50),
    order_side VARCHAR(10),
    order_quantity DOUBLE PRECISION,
    order_price DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引优化查询性能
CREATE INDEX idx_risk_events_timestamp ON risk_events(timestamp DESC);
CREATE INDEX idx_risk_events_event_type ON risk_events(event_type);
CREATE INDEX idx_risk_events_risk_level ON risk_events(risk_level);
CREATE INDEX idx_risk_events_rule_name ON risk_events(rule_name);

-- 风险指标快照表(用于历史分析)
CREATE TABLE IF NOT EXISTS risk_metrics_snapshots (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    account_balance DOUBLE PRECISION NOT NULL,
    account_equity DOUBLE PRECISION NOT NULL,
    daily_pnl DOUBLE PRECISION NOT NULL,
    peak_equity DOUBLE PRECISION NOT NULL,
    current_drawdown DOUBLE PRECISION NOT NULL,
    total_position_value DOUBLE PRECISION NOT NULL,
    leverage DOUBLE PRECISION NOT NULL,
    margin_usage_ratio DOUBLE PRECISION NOT NULL,
    risk_score DOUBLE PRECISION NOT NULL,
    daily_order_count INTEGER NOT NULL,
    daily_trade_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_risk_metrics_timestamp ON risk_metrics_snapshots(timestamp DESC);

-- 注释
COMMENT ON TABLE risk_events IS '风险事件记录表';
COMMENT ON TABLE risk_metrics_snapshots IS '风险指标快照表';

COMMENT ON COLUMN risk_events.event_type IS '事件类型: ORDER_REJECTED, RISK_WARNING, STOP_LOSS_TRIGGERED, etc.';
COMMENT ON COLUMN risk_events.risk_level IS '风险级别: LOW, MEDIUM, HIGH, CRITICAL';
COMMENT ON COLUMN risk_events.rule_name IS '触发的规则名称';

COMMENT ON COLUMN risk_metrics_snapshots.risk_score IS '风险评分(0-100)';
COMMENT ON COLUMN risk_metrics_snapshots.current_drawdown IS '当前回撤比例(0-1)';
COMMENT ON COLUMN risk_metrics_snapshots.leverage IS '杠杆倍数';
