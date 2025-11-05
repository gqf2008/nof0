/**
 * CTP 柜台选择页面
 * 
 * 显示所有可用的期货柜台，用户可以选择连接
 */

"use client";
import { useState, useEffect } from "react";
import { Link } from "react-router-dom";

interface BrokerConfig {
  name: string;
  broker_id: string;
  td_address: string;
  md_address: string;
  description: string;
  recommended?: boolean;
}

interface CtpExchangeConfig {
  id: string;
  name: string;
  brokers: BrokerConfig[];
}

export default function CtpBrokerSelectionPage() {
  const [brokers, setBrokers] = useState<BrokerConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // 从后端 API 加载 CTP 柜台配置
    fetch("/api/config/exchanges")
      .then((res) => res.json())
      .then((data) => {
        // 找到 CTP 交易所配置
        const ctpConfig = data.exchanges.find((e: any) => e.id === "ctp");
        if (ctpConfig && ctpConfig.brokers) {
          setBrokers(ctpConfig.brokers);
        } else {
          setError("未找到 CTP 柜台配置");
        }
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load CTP brokers:", err);
        setError("加载柜台配置失败");
        setLoading(false);
      });
  }, []);

  // 按类别分组
  const groupedBrokers = {
    openctp: brokers.filter(b => b.name.includes("openctp")),
    simnow: brokers.filter(b => b.name.includes("simnow")),
    futures: brokers.filter(b => !b.name.includes("openctp") && !b.name.includes("simnow")),
  };

  return (
    <main className="w-full min-h-[calc(100vh-var(--header-h))] p-8">
      <div className="max-w-7xl mx-auto">
        {/* 返回按钮 */}
        <Link
          to="/"
          className="inline-flex items-center terminal-text mb-8 hover:text-accent transition-colors"
        >
          <span className="mr-2">←</span>
          <span>BACK TO EXCHANGES</span>
        </Link>

        {/* 标题 */}
        <div className="mb-12">
          <h1 className="terminal-header text-4xl mb-4">
            CTP BROKER SELECTION
          </h1>
          <p className="terminal-text text-base opacity-70">
            SELECT A FUTURES BROKER TO CONNECT
          </p>
        </div>

        {/* 加载状态 */}
        {loading && (
          <div className="terminal-text text-xl text-center">
            LOADING BROKERS...
          </div>
        )}

        {/* 错误状态 */}
        {error && (
          <div className="terminal-text text-xl text-center text-accent-error">
            {error.toUpperCase()}
          </div>
        )}

        {/* 柜台列表 */}
        {!loading && !error && (
          <div className="space-y-12">
            {/* OpenCTP 柜台 */}
            {groupedBrokers.openctp.length > 0 && (
              <section>
                <h2 className="terminal-text text-2xl mb-6 text-accent">
                  ▸ OPENCTP PLATFORM
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {groupedBrokers.openctp.map((broker, index) => (
                    <BrokerCard key={index} broker={broker} />
                  ))}
                </div>
              </section>
            )}

            {/* SimNow 柜台 */}
            {groupedBrokers.simnow.length > 0 && (
              <section>
                <h2 className="terminal-text text-2xl mb-6 text-accent">
                  ▸ SIMNOW PLATFORM
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {groupedBrokers.simnow.map((broker, index) => (
                    <BrokerCard key={index} broker={broker} />
                  ))}
                </div>
              </section>
            )}

            {/* 期货公司柜台 */}
            {groupedBrokers.futures.length > 0 && (
              <section>
                <h2 className="terminal-text text-2xl mb-6 text-accent">
                  ▸ FUTURES BROKERS
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {groupedBrokers.futures.map((broker, index) => (
                    <BrokerCard key={index} broker={broker} />
                  ))}
                </div>
              </section>
            )}
          </div>
        )}

        {/* 统计信息 */}
        {!loading && !error && (
          <div className="mt-12 p-6 terminal-card">
            <div className="terminal-text text-sm opacity-70">
              <div>TOTAL BROKERS: {brokers.length}</div>
              <div className="mt-2">
                OPENCTP: {groupedBrokers.openctp.length} | 
                SIMNOW: {groupedBrokers.simnow.length} | 
                OTHERS: {groupedBrokers.futures.length}
              </div>
            </div>
          </div>
        )}
      </div>
    </main>
  );
}

// 柜台卡片组件
function BrokerCard({ broker }: { broker: BrokerConfig }) {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <Link
      to={`/ctp/monitor?broker=${encodeURIComponent(broker.name)}`}
      className="block"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div className={`terminal-card p-6 h-full transition-all ${isHovered ? 'border-accent' : ''}`}>
        {/* 柜台名称 */}
        <div className="flex items-start justify-between mb-4">
          <h3 className="terminal-text text-lg font-bold flex-1">
            {broker.name}
          </h3>
          {broker.recommended && (
            <span className="terminal-badge bg-accent text-background text-xs px-2 py-1 ml-2">
              推荐
            </span>
          )}
        </div>

        {/* 描述 */}
        <p className="terminal-text text-sm opacity-70 mb-4">
          {broker.description}
        </p>

        {/* 柜台信息 */}
        <div className="terminal-text text-xs opacity-50 space-y-1">
          <div className="flex items-center">
            <span className="w-20">BROKER:</span>
            <span className="font-mono">{broker.broker_id}</span>
          </div>
          <div className="flex items-start">
            <span className="w-20 flex-shrink-0">TD:</span>
            <span className="font-mono break-all">{broker.td_address}</span>
          </div>
          <div className="flex items-start">
            <span className="w-20 flex-shrink-0">MD:</span>
            <span className="font-mono break-all">{broker.md_address}</span>
          </div>
        </div>

        {/* 连接按钮提示 */}
        <div className="mt-4 pt-4 border-t border-border">
          <div className={`terminal-text text-sm transition-colors ${isHovered ? 'text-accent' : 'opacity-50'}`}>
            {isHovered ? '→ CLICK TO CONNECT' : 'HOVER TO CONNECT'}
          </div>
        </div>
      </div>
    </Link>
  );
}
