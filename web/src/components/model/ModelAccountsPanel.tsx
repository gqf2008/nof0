"use client";
import { useModelAccounts } from "@/lib/api/hooks/useModelAccounts";
import type { ModelAccount } from "@/lib/api/hooks/useModelAccounts";

export default function ModelAccountsPanel() {
  const { accounts, isLoading, isError } = useModelAccounts();

  if (isLoading) {
    return (
      <div className="text-xs text-zinc-500 p-4">加载AI模型账户数据...</div>
    );
  }

  if (isError) {
    return (
      <div className="text-xs text-red-500 p-4">
        加载AI模型账户数据失败
      </div>
    );
  }

  if (accounts.length === 0) {
    return (
      <div className="text-xs text-zinc-500 p-4">暂无AI模型账户数据</div>
    );
  }

  const getRiskLevelColor = (level: string) => {
    switch (level) {
      case "VERY_LOW":
        return "text-green-400";
      case "LOW":
        return "text-green-300";
      case "MEDIUM":
        return "text-yellow-400";
      case "HIGH":
        return "text-orange-400";
      case "VERY_HIGH":
        return "text-red-400";
      default:
        return "text-zinc-400";
    }
  };

  const getRiskLevelText = (level: string) => {
    switch (level) {
      case "VERY_LOW":
        return "极低风险";
      case "LOW":
        return "低风险";
      case "MEDIUM":
        return "中等风险";
      case "HIGH":
        return "高风险";
      case "VERY_HIGH":
        return "极高风险";
      default:
        return level;
    }
  };

  return (
    <div className="border-t-2 pt-4 mt-4" style={{ borderColor: "var(--panel-border)" }}>
      <h3 className="text-sm font-bold mb-3 text-zinc-300 px-2">
        AI 模型账户 ({accounts.length})
      </h3>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        {accounts.map((account) => {
          const pnlPositive = account.realized_pnl >= 0;
          const returnPositive = account.return_pct >= 0;

          return (
            <div
              key={account.model_id}
              className="border rounded p-3 bg-zinc-900/30"
              style={{ borderColor: "var(--panel-border)" }}
            >
              {/* 模型名称和策略 */}
              <div className="mb-2">
                <div className="text-sm font-bold text-zinc-200 mb-1">
                  {account.model_name}
                </div>
                <div className="text-xs text-zinc-400">
                  策略: {account.strategy}
                </div>
                <div className={`text-xs ${getRiskLevelColor(account.risk_level)}`}>
                  {getRiskLevelText(account.risk_level)}
                </div>
              </div>

              {/* 账户价值 */}
              <div className="mb-2">
                <div className="text-xs text-zinc-500">账户净值</div>
                <div className="text-base font-mono font-bold text-zinc-100">
                  ${account.dollar_equity.toLocaleString("en-US", {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 2,
                  })}
                </div>
              </div>

              {/* 盈亏信息 */}
              <div className="grid grid-cols-2 gap-2 text-xs mb-2">
                <div>
                  <div className="text-zinc-500">已实现盈亏</div>
                  <div
                    className={`font-mono ${pnlPositive ? "text-green-400" : "text-red-400"}`}
                  >
                    {pnlPositive ? "+" : ""}
                    ${account.realized_pnl.toLocaleString("en-US", {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 2,
                    })}
                  </div>
                </div>
                <div>
                  <div className="text-zinc-500">未实现盈亏</div>
                  <div
                    className={`font-mono ${account.unrealized_pnl >= 0 ? "text-green-400" : "text-red-400"}`}
                  >
                    {account.unrealized_pnl >= 0 ? "+" : ""}
                    ${account.unrealized_pnl.toLocaleString("en-US", {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 2,
                    })}
                  </div>
                </div>
              </div>

              {/* 收益率和夏普比率 */}
              <div className="grid grid-cols-2 gap-2 text-xs mb-2">
                <div>
                  <div className="text-zinc-500">收益率</div>
                  <div
                    className={`font-mono font-bold ${returnPositive ? "text-green-400" : "text-red-400"}`}
                  >
                    {returnPositive ? "+" : ""}
                    {account.return_pct.toFixed(2)}%
                  </div>
                </div>
                <div>
                  <div className="text-zinc-500">夏普比率</div>
                  <div className="font-mono text-zinc-300">
                    {account.sharpe_ratio.toFixed(2)}
                  </div>
                </div>
              </div>

              {/* 交易统计 */}
              <div className="grid grid-cols-3 gap-2 text-xs border-t pt-2" style={{ borderColor: "var(--panel-border)" }}>
                <div>
                  <div className="text-zinc-500">总交易</div>
                  <div className="font-mono text-zinc-300">
                    {account.total_trades}
                  </div>
                </div>
                <div>
                  <div className="text-zinc-500">胜率</div>
                  <div className="font-mono text-zinc-300">
                    {(account.win_rate * 100).toFixed(1)}%
                  </div>
                </div>
                <div>
                  <div className="text-zinc-500">胜/负</div>
                  <div className="font-mono text-zinc-300">
                    <span className="text-green-400">{account.winning_trades}</span>
                    /
                    <span className="text-red-400">{account.losing_trades}</span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
