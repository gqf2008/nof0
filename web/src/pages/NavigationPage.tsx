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
        {/* 标题 */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold mb-4" style={{ color: "var(--brand-accent)" }}>
            交易所导航
          </h1>
          <p className="text-lg text-zinc-400">
            选择一个交易所进入实时监控界面
          </p>
        </div>

        {/* 交易所卡片网格 */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {loading && <div className="text-white text-xl col-span-2 text-center">加载中...</div>}
          {error && <div className="text-red-500 text-xl col-span-2 text-center">{error}</div>}
          {!loading && !error && exchanges.map((exchange) => (
            <Link
              key={exchange.id}
              to={exchange.route}
              className="group"
            >
              <div
                className="border-2 rounded-xl p-8 transition-all duration-300 hover:scale-105 hover:shadow-2xl"
                style={{
                  borderColor: "var(--panel-border)",
                  background: "var(--card-bg)",
                }}
              >
                {/* 图标 */}
                <div className="flex items-center justify-between mb-6">
                  <div
                    className="text-6xl"
                    style={{
                      filter: "drop-shadow(0 0 10px rgba(255, 255, 255, 0.3))",
                    }}
                  >
                    {exchange.icon}
                  </div>
                  <div
                    className="px-3 py-1 rounded-full text-xs font-semibold"
                    style={{
                      background: "rgba(0, 255, 0, 0.1)",
                      color: "#00ff00",
                      border: "1px solid rgba(0, 255, 0, 0.3)",
                    }}
                  >
                    {exchange.status}
                  </div>
                </div>

                {/* 标题 */}
                <h2
                  className="text-2xl font-bold mb-3 group-hover:text-opacity-80 transition-colors"
                  style={{ color: exchange.color }}
                >
                  {exchange.name}
                </h2>

                {/* 描述 */}
                <p className="text-zinc-400 mb-6">{exchange.description}</p>

                {/* 进入按钮 */}
                <div className="flex items-center justify-end text-sm font-semibold">
                  <span
                    className="group-hover:translate-x-2 transition-transform"
                    style={{ color: exchange.color }}
                  >
                    进入监控 →
                  </span>
                </div>
              </div>
            </Link>
          ))}
        </div>

        {/* 底部提示 */}
        <div className="mt-12 text-center">
          <p className="text-sm text-zinc-500">
            💡 提示: 每个交易所都有独立的实时监控、持仓管理和交易功能
          </p>
        </div>
      </div>
    </main>
  );
}
