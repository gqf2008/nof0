use axum::http::Uri;
use rand::Rng;
use serde_json::{json, Value};
use std::collections::HashMap;

/// 根据 exchange_id 生成模拟数据
pub fn generate_mock_data(path: &str, exchange_id: &str) -> Option<Value> {
    match path {
        "account-totals" => Some(generate_account_totals(exchange_id)),
        "model-accounts" => Some(generate_model_accounts(exchange_id)),
        "positions" => Some(generate_positions(exchange_id)),
        "trades" => Some(generate_trades(exchange_id)),
        "crypto-prices" => Some(generate_crypto_prices(exchange_id)),
        "analytics" => Some(generate_analytics(exchange_id)),
        "leaderboard" => Some(generate_leaderboard(exchange_id)),
        "conversations" => Some(generate_conversations(exchange_id)),
        "since-inception-values" => Some(generate_since_inception(exchange_id)),
        "exchange-account" | "account" => Some(generate_exchange_account(exchange_id)),
        "models" => Some(generate_models_list(exchange_id)),
        _ => None,
    }
}

fn generate_account_totals(exchange_id: &str) -> Value {
    let mut rng = rand::thread_rng();
    let now = chrono::Utc::now().timestamp();

    // 返回该交易所下所有AI模型的历史账户数据（用于图表）
    let models = get_models_for_exchange(exchange_id);
    let mut account_totals = vec![];

    // 生成过去120分钟的历史数据（每分钟一个点）
    for model in &models {
        let base_value = model.base_capital;

        // 根据风险等级设置波动率和趋势
        let (volatility, trend_bias) = match model.risk_level.as_str() {
            "VERY_LOW" => (0.0005, 0.0001),  // 0.05%波动，微弱上涨趋势
            "LOW" => (0.001, 0.0002),        // 0.1%波动
            "MEDIUM" => (0.002, 0.0001),     // 0.2%波动
            "HIGH" => (0.003, -0.0001),      // 0.3%波动，微弱下跌
            "VERY_HIGH" => (0.005, -0.0002), // 0.5%波动，较弱下跌
            _ => (0.002, 0.0001),
        };

        let mut current_value = base_value;
        let mut cumulative_return = 0.0;

        for i in 0..120 {
            let minutes_ago = 119 - i;
            let timestamp = now - (minutes_ago * 60);

            // 随机游走 + 小趋势
            let random_change = rng.gen_range(-volatility..volatility);
            let change_pct = random_change + trend_bias;
            current_value *= 1.0 + change_pct;

            cumulative_return = (current_value - base_value) / base_value;
            let unrealized_pnl = current_value * rng.gen_range(-0.005..0.005); // 小幅未实现盈亏
            let dollar_equity = current_value + unrealized_pnl;

            account_totals.push(json!({
                "model_id": &model.id,
                "model_name": &model.name,
                "strategy": &model.strategy,
                "risk_level": &model.risk_level,
                "exchange_id": exchange_id,
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
                        rng.gen_range(0.8..2.5)
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

    json!({
        "accountTotals": account_totals
    })
}

// 生成当前模型账户摘要（用于账户卡片显示）
fn generate_model_accounts(exchange_id: &str) -> Value {
    let mut rng = rand::thread_rng();
    let now = chrono::Utc::now().timestamp();
    let models = get_models_for_exchange(exchange_id);
    let mut accounts = vec![];

    for model in models {
        let base_value = model.base_capital;

        // 根据风险等级生成盈亏幅度
        let pnl_pct = match model.risk_level.as_str() {
            "VERY_LOW" => rng.gen_range(-0.02..0.04),
            "LOW" => rng.gen_range(-0.03..0.06),
            "MEDIUM" => rng.gen_range(-0.05..0.10),
            "HIGH" => rng.gen_range(-0.08..0.15),
            "VERY_HIGH" => rng.gen_range(-0.12..0.25),
            _ => rng.gen_range(-0.05..0.10),
        };

        let realized_pnl = base_value * pnl_pct;
        let total_unrealized_pnl = base_value * rng.gen_range(-0.02..0.03);
        let dollar_equity = base_value + realized_pnl + total_unrealized_pnl;
        let return_pct = (realized_pnl / base_value) * 100.0;

        // 根据风险等级生成胜率和交易次数
        let (win_rate, trade_count) = match model.risk_level.as_str() {
            "VERY_LOW" => (rng.gen_range(0.55..0.70), rng.gen_range(20..50)),
            "LOW" => (rng.gen_range(0.52..0.65), rng.gen_range(30..80)),
            "MEDIUM" => (rng.gen_range(0.48..0.62), rng.gen_range(50..150)),
            "HIGH" => (rng.gen_range(0.45..0.60), rng.gen_range(80..250)),
            "VERY_HIGH" => (rng.gen_range(0.42..0.58), rng.gen_range(100..500)),
            _ => (rng.gen_range(0.45..0.60), rng.gen_range(50..200)),
        };

        accounts.push(json!({
            "model_id": model.id,
            "model_name": model.name,
            "strategy": model.strategy,
            "risk_level": model.risk_level,
            "exchange_id": exchange_id,
            "timestamp": now,
            "account_value": base_value,
            "dollar_equity": dollar_equity,
            "equity": dollar_equity,
            "realized_pnl": realized_pnl,
            "unrealized_pnl": total_unrealized_pnl,
            "total_unrealized_pnl": total_unrealized_pnl,
            "return_pct": return_pct,
            "cum_pnl_pct": return_pct,
            "sharpe_ratio": if return_pct > 0.0 { rng.gen_range(0.8..2.5) } else { rng.gen_range(-0.5..0.8) },
            "win_rate": win_rate,
            "total_trades": trade_count,
            "winning_trades": (trade_count as f64 * win_rate) as i32,
            "losing_trades": trade_count - (trade_count as f64 * win_rate) as i32,
            "positions": {}
        }));
    }

    json!({
        "accounts": accounts
    })
}

fn generate_positions(exchange_id: &str) -> Value {
    let symbols = get_symbols_for_exchange(exchange_id);
    let mut rng = rand::thread_rng();
    let now = chrono::Utc::now().timestamp();

    let mut positions = HashMap::new();

    // 根据交易所类型设置不同的持仓特征
    let (num_positions, leverage_range, risk_profile) = match exchange_id {
        "crypto" => (rng.gen_range(2..=4), (5, 20), "aggressive"),
        "ctp" => (rng.gen_range(3..=5), (1, 10), "conservative"),
        "binance" => (rng.gen_range(3..=6), (10, 25), "very_aggressive"),
        "bybit" => (rng.gen_range(2..=5), (5, 15), "moderate"),
        "kraken" => (rng.gen_range(1..=3), (1, 5), "very_conservative"),
        _ => (rng.gen_range(2..=4), (1, 10), "moderate"),
    };

    let num_positions = num_positions.min(symbols.len());

    for i in 0..num_positions {
        let symbol = &symbols[i];
        let entry_price = get_base_price(symbol);

        // 根据风险配置设置价格波动
        let price_volatility = match risk_profile {
            "very_aggressive" => rng.gen_range(0.90..1.10),
            "aggressive" => rng.gen_range(0.93..1.07),
            "moderate" => rng.gen_range(0.95..1.05),
            "conservative" => rng.gen_range(0.97..1.03),
            _ => rng.gen_range(0.98..1.02),
        };

        let current_price = entry_price * price_volatility;

        // 根据交易所设置持仓大小
        let quantity = match exchange_id {
            "crypto" => rng.gen_range(0.5..5.0),
            "ctp" => rng.gen_range(10.0..100.0), // 期货合约数量较大
            "binance" => rng.gen_range(1.0..20.0),
            "bybit" => rng.gen_range(0.3..8.0),
            "kraken" => rng.gen_range(0.1..3.0),
            _ => rng.gen_range(1.0..10.0),
        };

        // 多空方向概率根据市场类型调整
        let long_probability = match exchange_id {
            "crypto" => 0.6,   // 加密货币偏多
            "ctp" => 0.5,      // 期货平衡
            "binance" => 0.65, // 币安偏多
            "bybit" => 0.55,
            "kraken" => 0.5,
            _ => 0.5,
        };

        let side = if rng.gen_bool(long_probability) {
            1.0
        } else {
            -1.0
        };
        let signed_qty = quantity * side;
        let unrealized_pnl = (current_price - entry_price) * quantity * side;

        let leverage = rng.gen_range(leverage_range.0..=leverage_range.1);
        let margin = (entry_price * quantity) / leverage as f64;

        // 生成平仓计划
        let (profit_target, stop_loss) = if side > 0.0 {
            // 做多
            (
                current_price * rng.gen_range(1.05..1.15),
                current_price * rng.gen_range(0.85..0.95),
            )
        } else {
            // 做空
            (
                current_price * rng.gen_range(0.85..0.95),
                current_price * rng.gen_range(1.05..1.15),
            )
        };

        positions.insert(
            symbol.clone(),
            json!({
                "symbol": symbol,
                "entry_price": entry_price,
                "current_price": current_price,
                "quantity": signed_qty,
                "unrealized_pnl": unrealized_pnl,
                "leverage": leverage,
                "margin": margin,
                "entry_time": now - rng.gen_range(3600..86400),
                "confidence": rng.gen_range(0.55..0.85),
                "side": if side > 0.0 { "LONG" } else { "SHORT" },
                "entry_oid": rng.gen_range(100000000..999999999),
                "risk_usd": margin * 0.02, // 2% 风险
                "exit_plan": {
                    "profit_target": profit_target,
                    "stop_loss": stop_loss,
                    "invalidation_condition": format!("如果价格突破 {:.2} 则止损", stop_loss)
                },
                "closed_pnl": rng.gen_range(-50.0..100.0),
                "commission": quantity * current_price * 0.0004,
                "liquidation_price": if side > 0.0 {
                    entry_price * (1.0 - 1.0 / leverage as f64)
                } else {
                    entry_price * (1.0 + 1.0 / leverage as f64)
                },
            }),
        );
    }

    json!({
        "accountTotals": [{
            "model_id": exchange_id,
            "exchange_id": exchange_id,
            "timestamp": now,
            "realized_pnl": rng.gen_range(-500.0..1000.0),
            "total_unrealized_pnl": positions.values()
                .filter_map(|p| p.get("unrealized_pnl").and_then(|v| v.as_f64()))
                .sum::<f64>(),
            "positions": positions
        }]
    })
}

fn generate_trades(exchange_id: &str) -> Value {
    let symbols = get_symbols_for_exchange(exchange_id);
    let mut rng = rand::thread_rng();
    let mut trades = vec![];
    let now = chrono::Utc::now().timestamp();

    // 根据交易所设置不同的交易特征
    let (num_trades, win_rate, avg_holding_mins, trade_size_range) = match exchange_id {
        "crypto" => (
            rng.gen_range(15..30),
            0.58,        // 58% 胜率
            (120, 1440), // 2-24小时
            (0.5, 5.0),
        ),
        "ctp" => (
            rng.gen_range(20..40),
            0.52,      // 52% 胜率
            (30, 240), // 30分钟-4小时
            (10.0, 100.0),
        ),
        "binance" => (
            rng.gen_range(25..50),
            0.62,      // 62% 胜率 - 更激进
            (15, 180), // 15分钟-3小时
            (1.0, 20.0),
        ),
        "bybit" => (
            rng.gen_range(18..35),
            0.55,
            (60, 360), // 1-6小时
            (0.3, 8.0),
        ),
        "kraken" => (
            rng.gen_range(10..20),
            0.60,       // 更保守,胜率更高
            (180, 720), // 3-12小时
            (0.1, 3.0),
        ),
        _ => (rng.gen_range(15..25), 0.55, (60, 300), (1.0, 10.0)),
    };

    let mut total_pnl = 0.0;
    let mut winning_trades = 0;

    for i in 0..num_trades {
        let symbol = &symbols[i % symbols.len()];
        let base_price = get_base_price(symbol);

        // 是否盈利 (根据胜率)
        let is_winner = rng.gen_bool(win_rate);

        let quantity = rng.gen_range(trade_size_range.0..trade_size_range.1);
        let side_long = rng.gen_bool(0.5);

        // 生成价格和盈亏
        let (entry_price, exit_price, pnl) = if is_winner {
            winning_trades += 1;
            let profit_pct = rng.gen_range(0.01..0.08); // 1-8% 利润
            if side_long {
                let entry = base_price * rng.gen_range(0.95..1.0);
                let exit = entry * (1.0 + profit_pct);
                let pnl = (exit - entry) * quantity;
                (entry, exit, pnl)
            } else {
                let entry = base_price * rng.gen_range(1.0..1.05);
                let exit = entry * (1.0 - profit_pct);
                let pnl = (entry - exit) * quantity;
                (entry, exit, pnl)
            }
        } else {
            let loss_pct = rng.gen_range(0.01..0.05); // 1-5% 亏损
            if side_long {
                let entry = base_price * rng.gen_range(0.95..1.0);
                let exit = entry * (1.0 - loss_pct);
                let pnl = (exit - entry) * quantity;
                (entry, exit, pnl)
            } else {
                let entry = base_price * rng.gen_range(1.0..1.05);
                let exit = entry * (1.0 + loss_pct);
                let pnl = (entry - exit) * quantity;
                (entry, exit, pnl)
            }
        };

        total_pnl += pnl;

        let holding_time = rng.gen_range(avg_holding_mins.0..avg_holding_mins.1) * 60;
        let exit_time = now - rng.gen_range(3600..86400 * 7); // 过去7天内
        let entry_time = exit_time - holding_time;

        let fee_rate = match exchange_id {
            "crypto" => 0.0004,
            "ctp" => 0.0002, // 期货手续费较低
            "binance" => 0.0005,
            "bybit" => 0.0006,
            "kraken" => 0.0003,
            _ => 0.0004,
        };

        let fee = quantity * entry_price * fee_rate + quantity * exit_price * fee_rate;

        trades.push(json!({
            "trade_id": format!("{}_{:06}", exchange_id, i),
            "doc_id": format!("{}_trade_{}", exchange_id, i),
            "model_id": exchange_id,
            "exchange_id": exchange_id,
            "symbol": symbol,
            "side": if side_long { "LONG" } else { "SHORT" },
            "entry_price": entry_price,
            "exit_price": exit_price,
            "quantity": quantity,
            "gross_pnl": pnl,
            "net_pnl": pnl - fee,
            "fees": fee,
            "entry_time": entry_time,
            "exit_time": exit_time,
            "holding_period_mins": holding_time / 60,
            "entry_oid": rng.gen_range(100000000..999999999),
            "exit_oid": rng.gen_range(100000000..999999999),
            "leverage": rng.gen_range(1..20),
            "exit_reason": if is_winner { "TAKE_PROFIT" } else { "STOP_LOSS" },
            "slippage": rng.gen_range(0.0..0.001),
            "status": "CLOSED",
        }));
    }

    json!({
        "trades": trades,
        "summary": {
            "total_trades": num_trades,
            "winning_trades": winning_trades,
            "losing_trades": num_trades - winning_trades,
            "actual_win_rate": winning_trades as f64 / num_trades as f64,
            "total_pnl": total_pnl,
            "exchange_id": exchange_id
        }
    })
}

fn generate_crypto_prices(exchange_id: &str) -> Value {
    let symbols = get_symbols_for_exchange(exchange_id);
    let mut prices = HashMap::new();

    for symbol in symbols {
        let base_price = get_base_price(&symbol);
        let mut rng = rand::thread_rng();
        prices.insert(
            symbol.clone(),
            json!({
                "symbol": symbol,
                "price": base_price * rng.gen_range(0.98..1.02),
                "change_24h": rng.gen_range(-5.0..5.0),
                "volume_24h": rng.gen_range(1000000.0..10000000.0),
                "timestamp": chrono::Utc::now().timestamp()
            }),
        );
    }

    json!({
        "prices": prices
    })
}

fn generate_analytics(exchange_id: &str) -> Value {
    let mut rng = rand::thread_rng();

    json!({
        "analytics": [{
            "id": exchange_id,
            "exchange_id": exchange_id,
            "total_trades": rng.gen_range(50..200),
            "win_rate": rng.gen_range(0.45..0.65),
            "avg_pnl": rng.gen_range(-100.0..200.0),
            "total_pnl": rng.gen_range(-5000.0..10000.0),
            "sharpe_ratio": rng.gen_range(0.5..2.5),
            "max_drawdown": rng.gen_range(-0.2..-0.05),
            "avg_holding_period_mins": rng.gen_range(30.0..1440.0),
            "total_fees_paid": rng.gen_range(100.0..1000.0),
            "updated_at": chrono::Utc::now().timestamp()
        }]
    })
}

fn generate_leaderboard(exchange_id: &str) -> Value {
    let mut rng = rand::thread_rng();
    let mut leaderboard = vec![];

    for i in 1..=10 {
        leaderboard.push(json!({
            "rank": i,
            "model_id": format!("{}_{}", exchange_id, i),
            "total_pnl": rng.gen_range(1000.0..50000.0) * (11 - i) as f64 / 10.0,
            "win_rate": rng.gen_range(0.4..0.7),
            "total_trades": rng.gen_range(50..300),
            "sharpe_ratio": rng.gen_range(0.5..3.0),
            "exchange_id": exchange_id
        }));
    }

    json!({
        "leaderboard": leaderboard
    })
}

fn generate_conversations(exchange_id: &str) -> Value {
    let mut rng = rand::thread_rng();
    let mut conversations = vec![];
    let symbols = get_symbols_for_exchange(exchange_id);
    let now = chrono::Utc::now().timestamp();
    let models = get_models_for_exchange(exchange_id);

    // 每个AI模型生成若干对话
    let conversations_per_model = match exchange_id {
        "crypto" => rng.gen_range(3..5),
        "ctp" => rng.gen_range(5..8),
        "binance" => rng.gen_range(4..7),
        "bybit" => rng.gen_range(3..6),
        "kraken" => rng.gen_range(2..4),
        _ => rng.gen_range(3..5),
    };

    for model in &models {
        // 根据模型的风险等级设置信号分布
        let signal_distribution = match model.risk_level.as_str() {
            "VERY_LOW" => (0.25, 0.20, 0.55), // 很保守，多观望
            "LOW" => (0.30, 0.25, 0.45),
            "MEDIUM" => (0.35, 0.30, 0.35),
            "HIGH" => (0.40, 0.35, 0.25),
            "VERY_HIGH" => (0.45, 0.40, 0.15), // 很激进，少观望
            _ => (0.33, 0.33, 0.34),
        };

        for i in 0..conversations_per_model {
            let symbol = &symbols[i % symbols.len()];
            let current_price = get_base_price(symbol);

            // 根据概率分布确定信号
            let rand_val = rng.gen_range(0.0..1.0);
            let signal = if rand_val < signal_distribution.0 {
                "LONG"
            } else if rand_val < signal_distribution.0 + signal_distribution.1 {
                "SHORT"
            } else {
                "HOLD"
            };

            let confidence = match signal {
                "LONG" | "SHORT" => rng.gen_range(0.60..0.85),
                _ => rng.gen_range(0.50..0.70),
            };

            let timestamp = now - rng.gen_range(60..7200); // 过去2小时内

            // 生成AI分析内容
            let analysis = match signal {
                "LONG" => format!(
                    "{}技术指标显示上涨趋势。RSI: {:.1}, MACD: 多头排列, 支撑位: {:.2}",
                    symbol,
                    rng.gen_range(45.0..65.0),
                    current_price * 0.95
                ),
                "SHORT" => format!(
                    "{}出现看跌信号。RSI: {:.1}, MACD: 空头排列, 阻力位: {:.2}",
                    symbol,
                    rng.gen_range(55.0..75.0),
                    current_price * 1.05
                ),
                _ => format!(
                    "{}当前处于震荡区间,建议观望。波动率: {:.1}%",
                    symbol,
                    rng.gen_range(1.0..5.0)
                ),
            };

            let reasoning = format!(
                "{} 模型分析: {}当前{}机会。策略: {} | 风险等级: {}",
                model.name,
                symbol,
                if signal == "LONG" {
                    "存在做多"
                } else if signal == "SHORT" {
                    "存在做空"
                } else {
                    "暂无明确"
                },
                model.strategy,
                model.risk_level
            );

            // 生成消息历史
            let messages = vec![
                json!({
                    "role": "system",
                    "content": format!("你是{}交易所的{}，专注于{}", exchange_id, model.name, model.strategy),
                    "timestamp": timestamp - 30
                }),
                json!({
                    "role": "user",
                    "content": format!("分析{}的当前市场状况", symbol),
                    "timestamp": timestamp - 20
                }),
                json!({
                    "role": "assistant",
                    "content": analysis,
                    "timestamp": timestamp - 10
                }),
                json!({
                    "role": "assistant",
                    "content": reasoning,
                    "timestamp": timestamp
                }),
            ];

            conversations.push(json!({
                "conversation_id": format!("{}_{}", model.id, timestamp),
                "doc_id": format!("{}_conv_{}_{}", exchange_id, model.id, timestamp),
                "model_id": model.id,
                "model_name": model.name,
                "ai_model": model.name,
                "exchange_id": exchange_id,
                "symbol": symbol,
                "timestamp": timestamp,
                "signal": signal,
                "confidence": confidence,
                "strategy": model.strategy,
                "risk_level": model.risk_level,
                "messages": messages,
                "technical_indicators": {
                    "rsi": rng.gen_range(30.0..70.0),
                    "macd": rng.gen_range(-100.0..100.0),
                    "volume_ratio": rng.gen_range(0.5..2.0),
                    "trend": if signal == "LONG" { "BULLISH" } else if signal == "SHORT" { "BEARISH" } else { "NEUTRAL" }
                },
                "recommended_action": {
                    "action": signal,
                    "entry_price": if signal != "HOLD" { Some(current_price) } else { None },
                    "target_price": if signal == "LONG" {
                        Some(current_price * rng.gen_range(1.03..1.08))
                    } else if signal == "SHORT" {
                        Some(current_price * rng.gen_range(0.92..0.97))
                    } else {
                        None
                    },
                    "stop_loss": if signal == "LONG" {
                        Some(current_price * rng.gen_range(0.95..0.98))
                    } else if signal == "SHORT" {
                        Some(current_price * rng.gen_range(1.02..1.05))
                    } else {
                        None
                    },
                    "position_size": if signal != "HOLD" { rng.gen_range(0.05..0.15) } else { 0.0 },
                },
                "risk_assessment": {
                    "risk_level": match confidence {
                        c if c > 0.75 => "LOW",
                        c if c > 0.65 => "MEDIUM",
                        _ => "HIGH"
                    },
                    "volatility": rng.gen_range(0.01..0.05),
                    "market_condition": match signal {
                        "LONG" => "UPTREND",
                        "SHORT" => "DOWNTREND",
                        _ => "RANGING"
                    }
                }
            }));
        }
    }

    let total_conversations = conversations.len();

    json!({
        "conversations": conversations,
        "metadata": {
            "total_conversations": total_conversations,
            "exchange_id": exchange_id,
            "num_models": models.len(),
            "time_range": "过去2小时",
            "models": models.iter().map(|m| m.name.clone()).collect::<Vec<_>>()
        }
    })
}

fn generate_since_inception(exchange_id: &str) -> Value {
    let now = chrono::Utc::now().timestamp();

    json!({
        "sinceInceptionValues": [{
            "id": format!("{}_inception", exchange_id),
            "model_id": exchange_id,
            "exchange_id": exchange_id,
            "nav_since_inception": 10000,
            "inception_date": now - 7200,  // 2小时前开始
            "num_invocations": 120
        }]
    })
}

fn get_symbols_for_exchange(exchange_id: &str) -> Vec<String> {
    match exchange_id {
        "crypto" | "binance" | "bybit" | "kraken" => {
            vec![
                "BTC".to_string(),
                "ETH".to_string(),
                "SOL".to_string(),
                "AVAX".to_string(),
            ]
        }
        "ctp" => {
            vec![
                "IF2504".to_string(),
                "IH2504".to_string(),
                "IC2504".to_string(),
                "IM2504".to_string(),
            ]
        }
        _ => vec![
            "ASSET1".to_string(),
            "ASSET2".to_string(),
            "ASSET3".to_string(),
        ],
    }
}

fn get_base_price(symbol: &str) -> f64 {
    match symbol {
        "BTC" => 106000.0,
        "ETH" => 3800.0,
        "SOL" => 230.0,
        "AVAX" => 45.0,
        "IF2504" => 4200.0, // 沪深300股指期货
        "IH2504" => 3100.0, // 上证50股指期货
        "IC2504" => 5800.0, // 中证500股指期货
        "IM2504" => 4500.0, // 中证1000股指期货
        _ => 100.0,
    }
}

/// 获取每个交易所的AI模型列表
fn get_models_for_exchange(exchange_id: &str) -> Vec<ModelInfo> {
    match exchange_id {
        "crypto" => vec![
            ModelInfo {
                id: "crypto_gpt4".to_string(),
                name: "GPT-4 趋势追踪".to_string(),
                strategy: "趋势跟踪".to_string(),
                description: "使用GPT-4分析市场趋势，捕捉中长期方向".to_string(),
                risk_level: "MEDIUM".to_string(),
                base_capital: 30000.0,
            },
            ModelInfo {
                id: "crypto_claude".to_string(),
                name: "Claude 动量交易".to_string(),
                strategy: "动量交易".to_string(),
                description: "基于Claude分析价格动量，快速进出".to_string(),
                risk_level: "HIGH".to_string(),
                base_capital: 25000.0,
            },
            ModelInfo {
                id: "crypto_gemini".to_string(),
                name: "Gemini 套利".to_string(),
                strategy: "套利策略".to_string(),
                description: "利用Gemini识别跨市场套利机会".to_string(),
                risk_level: "LOW".to_string(),
                base_capital: 45000.0,
            },
        ],
        "ctp" => vec![
            ModelInfo {
                id: "ctp_quant_alpha".to_string(),
                name: "量化Alpha".to_string(),
                strategy: "统计套利".to_string(),
                description: "基于统计模型的期货套利系统".to_string(),
                risk_level: "LOW".to_string(),
                base_capital: 150000.0,
            },
            ModelInfo {
                id: "ctp_arbitrage_engine".to_string(),
                name: "套利引擎".to_string(),
                strategy: "跨期套利".to_string(),
                description: "识别不同到期合约间的价差机会".to_string(),
                risk_level: "VERY_LOW".to_string(),
                base_capital: 200000.0,
            },
            ModelInfo {
                id: "ctp_risk_control".to_string(),
                name: "风控系统".to_string(),
                strategy: "风险中性".to_string(),
                description: "基于Delta中性的对冲策略".to_string(),
                risk_level: "VERY_LOW".to_string(),
                base_capital: 150000.0,
            },
        ],
        "binance" => vec![
            ModelInfo {
                id: "binance_alpha".to_string(),
                name: "Alpha Strategy".to_string(),
                strategy: "高频动量".to_string(),
                description: "高频交易动量捕捉系统".to_string(),
                risk_level: "VERY_HIGH".to_string(),
                base_capital: 20000.0,
            },
            ModelInfo {
                id: "binance_beta".to_string(),
                name: "Beta Engine".to_string(),
                strategy: "波段交易".to_string(),
                description: "中期波段交易引擎".to_string(),
                risk_level: "HIGH".to_string(),
                base_capital: 30000.0,
            },
            ModelInfo {
                id: "binance_gamma".to_string(),
                name: "Gamma AI".to_string(),
                strategy: "趋势跟踪".to_string(),
                description: "基于AI的趋势识别系统".to_string(),
                risk_level: "MEDIUM".to_string(),
                base_capital: 30000.0,
            },
        ],
        "bybit" => vec![
            ModelInfo {
                id: "bybit_deepseek".to_string(),
                name: "DeepSeek V3".to_string(),
                strategy: "深度学习".to_string(),
                description: "使用DeepSeek的深度学习交易模型".to_string(),
                risk_level: "MEDIUM".to_string(),
                base_capital: 35000.0,
            },
            ModelInfo {
                id: "bybit_qwen".to_string(),
                name: "Qwen Max".to_string(),
                strategy: "智能分析".to_string(),
                description: "Qwen大模型驱动的市场分析".to_string(),
                risk_level: "MEDIUM".to_string(),
                base_capital: 40000.0,
            },
            ModelInfo {
                id: "bybit_mistral".to_string(),
                name: "Mistral Trader".to_string(),
                strategy: "波段交易".to_string(),
                description: "基于Mistral的波段交易系统".to_string(),
                risk_level: "HIGH".to_string(),
                base_capital: 45000.0,
            },
        ],
        "kraken" => vec![
            ModelInfo {
                id: "kraken_conservative".to_string(),
                name: "Conservative AI".to_string(),
                strategy: "价值投资".to_string(),
                description: "保守的长期价值投资策略".to_string(),
                risk_level: "VERY_LOW".to_string(),
                base_capital: 30000.0,
            },
            ModelInfo {
                id: "kraken_risk_aware".to_string(),
                name: "Risk Aware".to_string(),
                strategy: "风险管理".to_string(),
                description: "注重风险控制的稳健交易".to_string(),
                risk_level: "LOW".to_string(),
                base_capital: 35000.0,
            },
            ModelInfo {
                id: "kraken_value".to_string(),
                name: "Value Finder".to_string(),
                strategy: "价值发现".to_string(),
                description: "寻找被低估的交易机会".to_string(),
                risk_level: "LOW".to_string(),
                base_capital: 25000.0,
            },
        ],
        _ => vec![ModelInfo {
            id: format!("{}_model1", exchange_id),
            name: "Default Model 1".to_string(),
            strategy: "混合策略".to_string(),
            description: "默认交易模型".to_string(),
            risk_level: "MEDIUM".to_string(),
            base_capital: 50000.0,
        }],
    }
}

#[derive(Clone)]
struct ModelInfo {
    id: String,
    name: String,
    strategy: String,
    description: String,
    risk_level: String,
    base_capital: f64,
}

/// 生成交易所总账户信息（汇总所有AI模型）
fn generate_exchange_account(exchange_id: &str) -> Value {
    let models = get_models_for_exchange(exchange_id);
    let mut rng = rand::thread_rng();
    let now = chrono::Utc::now().timestamp();

    // 计算总资金
    let total_initial_capital: f64 = models.iter().map(|m| m.base_capital).sum();

    // 生成每个模型的当前状态
    let mut model_accounts = vec![];
    let mut total_equity = 0.0;
    let mut total_realized_pnl = 0.0;
    let mut total_unrealized_pnl = 0.0;
    let mut total_positions_count = 0;
    let mut total_trades_today = 0;

    for model in &models {
        let pnl_pct = match model.risk_level.as_str() {
            "VERY_LOW" => rng.gen_range(-0.02..0.04),
            "LOW" => rng.gen_range(-0.03..0.06),
            "MEDIUM" => rng.gen_range(-0.05..0.10),
            "HIGH" => rng.gen_range(-0.08..0.15),
            "VERY_HIGH" => rng.gen_range(-0.12..0.20),
            _ => rng.gen_range(-0.05..0.10),
        };

        let realized_pnl = model.base_capital * pnl_pct;
        let unrealized_pnl = rng.gen_range(-500.0..500.0);
        let equity = model.base_capital + realized_pnl + unrealized_pnl;

        let positions_count = match model.risk_level.as_str() {
            "VERY_LOW" => rng.gen_range(1..3),
            "LOW" => rng.gen_range(1..3),
            "MEDIUM" => rng.gen_range(2..4),
            "HIGH" => rng.gen_range(3..5),
            "VERY_HIGH" => rng.gen_range(3..6),
            _ => rng.gen_range(2..4),
        };

        let trades_today = match model.risk_level.as_str() {
            "VERY_LOW" => rng.gen_range(2..8),
            "LOW" => rng.gen_range(5..15),
            "MEDIUM" => rng.gen_range(10..20),
            "HIGH" => rng.gen_range(15..30),
            "VERY_HIGH" => rng.gen_range(20..50),
            _ => rng.gen_range(5..20),
        };

        total_equity += equity;
        total_realized_pnl += realized_pnl;
        total_unrealized_pnl += unrealized_pnl;
        total_positions_count += positions_count;
        total_trades_today += trades_today;

        model_accounts.push(json!({
            "model_id": model.id,
            "model_name": model.name,
            "strategy": model.strategy,
            "risk_level": model.risk_level,
            "initial_capital": model.base_capital,
            "current_equity": equity,
            "realized_pnl": realized_pnl,
            "unrealized_pnl": unrealized_pnl,
            "pnl_pct": pnl_pct * 100.0,
            "positions_count": positions_count,
            "trades_today": trades_today,
            "status": "ACTIVE",
            "last_trade_time": now - rng.gen_range(60..3600),
        }));
    }

    json!({
        "exchange_id": exchange_id,
        "exchange_name": match exchange_id {
            "crypto" => "Crypto Exchange",
            "ctp" => "CTP Futures",
            "binance" => "Binance",
            "bybit" => "Bybit",
            "kraken" => "Kraken",
            _ => exchange_id
        },
        "timestamp": now,
        "account_summary": {
            "total_initial_capital": total_initial_capital,
            "total_equity": total_equity,
            "total_realized_pnl": total_realized_pnl,
            "total_unrealized_pnl": total_unrealized_pnl,
            "total_pnl": total_realized_pnl + total_unrealized_pnl,
            "total_pnl_pct": ((total_equity - total_initial_capital) / total_initial_capital) * 100.0,
            "total_positions": total_positions_count,
            "total_trades_today": total_trades_today,
            "num_active_models": models.len(),
            "available_balance": total_equity * 0.3, // 假设70%已使用
            "margin_used": total_equity * 0.7,
            "margin_ratio": 0.7,
        },
        "models": model_accounts,
        "risk_metrics": {
            "max_drawdown": rng.gen_range(-0.15..-0.02),
            "sharpe_ratio": rng.gen_range(0.8..2.5),
            "win_rate": rng.gen_range(0.48..0.65),
            "avg_win_loss_ratio": rng.gen_range(1.2..2.5),
            "volatility": rng.gen_range(0.01..0.05),
        }
    })
}

/// 生成交易所的AI模型列表
fn generate_models_list(exchange_id: &str) -> Value {
    let models = get_models_for_exchange(exchange_id);
    let now = chrono::Utc::now().timestamp();

    let model_list: Vec<Value> = models
        .iter()
        .map(|model| {
            json!({
                "model_id": model.id,
                "model_name": model.name,
                "exchange_id": exchange_id,
                "strategy": model.strategy,
                "description": model.description,
                "risk_level": model.risk_level,
                "status": "ACTIVE",
                "created_at": now - 86400 * 30, // 30天前创建
                "last_updated": now - rand::thread_rng().gen_range(60..3600),
            })
        })
        .collect();

    json!({
        "exchange_id": exchange_id,
        "models": model_list,
        "total_models": models.len(),
    })
}
