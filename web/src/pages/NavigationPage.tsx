"use client";
import { Link } from "react-router-dom";
import { useState, useEffect } from "react";

interface ExchangeConfig {
  id: string;
  name: string;
  name_en: string;
  description: string;
  icon: string;
  color: string;
  enabled: boolean;
  status: string;
  route: string;
  api_endpoint: string;
  features: string[];
}

interface ExchangesConfigResponse {
  exchanges: ExchangeConfig[];
  settings: any;
}

export default function NavigationPage() {
  const [exchanges, setExchanges] = useState<ExchangeConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // 从后端 API 加载交易所配置
    fetch("/api/config/exchanges")
      .then((res) => res.json())
      .then((data: ExchangesConfigResponse) => {
        // 只显示已启用的交易所
        const enabledExchanges = data.exchanges.filter((e) => e.enabled);
        setExchanges(enabledExchanges);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load exchanges config:", err);
        setError("加载配置失败，请刷新页面重试");
        setLoading(false);
      });
  }, []);

  return (
    <main className="w-full min-h-[calc(100vh-var(--header-h))] flex items-center justify-center p-8">
      <div className="max-w-6xl w-full">
        {/* 标题 - Terminal 风格 */}
        <div className="text-center mb-12">
          <h1 className="terminal-header text-4xl mb-4" style={{ color: "var(--foreground)" }}>
            EXCHANGE NAVIGATION
          </h1>
          <p className="terminal-text text-base" style={{ color: "var(--foreground-muted)" }}>
            SELECT AN EXCHANGE TO ENTER REAL-TIME MONITORING
          </p>
        </div>

        {/* 交易所卡片网格 - Terminal 风格 */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {loading && (
            <div className="terminal-text text-xl col-span-2 text-center" style={{ color: "var(--foreground)" }}>
              LOADING...
            </div>
          )}
          {error && (
            <div className="terminal-text text-xl col-span-2 text-center" style={{ color: "var(--accent-error)" }}>
              {error.toUpperCase()}
            </div>
          )}
          {!loading && !error && exchanges.map((exchange) => (
            <Link
              key={exchange.id}
              to={exchange.route}
              className="block"
            >
              <div className="terminal-card p-8">
                {/* 图标和状态 */}
                <div className="flex items-center justify-between mb-6">
                  <div className="text-6xl">
                    {exchange.icon}
                  </div>
                  <div
                    className="terminal-button-small"
                    style={{
                      background: exchange.status === "运行中" 
                        ? "rgba(0, 170, 0, 0.1)" 
                        : "rgba(170, 170, 170, 0.1)",
                      color: exchange.status === "运行中" 
                        ? "var(--terminal-green)" 
                        : "var(--foreground-muted)",
                      borderColor: exchange.status === "运行中"
                        ? "var(--terminal-green)"
                        : "var(--border-subtle)",
                    }}
                  >
                    {exchange.status.toUpperCase()}
                  </div>
                </div>

                {/* 标题 - Terminal 风格 */}
                <h2
                  className="terminal-header text-2xl mb-3"
                  style={{ color: "var(--foreground)" }}
                >
                  {exchange.name.toUpperCase()}
                </h2>

                {/* 英文名称 */}
                <p className="terminal-text text-sm mb-2" style={{ color: "var(--foreground-muted)" }}>
                  {exchange.name_en.toUpperCase()}
                </p>

                {/* 描述 */}
                <p className="terminal-text text-sm mb-6" style={{ color: "var(--foreground-subtle)" }}>
                  {exchange.description}
                </p>

                {/* 进入按钮 - Terminal 风格 */}
                <div className="flex items-center justify-end">
                  <button
                    className="terminal-button"
                    style={{
                      borderColor: "var(--border)",
                      background: "var(--surface-elevated)",
                    }}
                  >
                    ENTER MONITORING →
                  </button>
                </div>
              </div>
            </Link>
          ))}
        </div>

        {/* 底部提示 - Terminal 风格 */}
        <div className="mt-12 text-center">
          <p className="terminal-text text-xs" style={{ color: "var(--foreground-subtle)" }}>
            ⚡ INDEPENDENT REAL-TIME MONITORING, POSITION MANAGEMENT AND TRADING FOR EACH EXCHANGE
          </p>
        </div>
      </div>
    </main>
  );
}
