/**
 * 统一 API 演示页面
 * 
 * 展示统一交易 API 的使用方法
 */

'use client';

import { useState } from 'react';
import { useExchanges, useExchange, useMarketTicker, useAccount, usePositions } from '@/hooks/useUnifiedApi';

export default function UnifiedApiDemo() {
  const { exchanges, loading: exchangesLoading } = useExchanges();
  const [selectedExchange, setSelectedExchange] = useState('ctp');

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 p-8">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* 页头 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            🚀 统一交易 API 演示
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            统一的交易接口，支持多交易所无缝切换
          </p>
        </div>

        {/* 交易所选择 */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
            📊 选择交易所
          </h2>
          {exchangesLoading ? (
            <div className="text-gray-500">加载中...</div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {exchanges.map((exchange) => (
                <button
                  key={exchange.id}
                  onClick={() => setSelectedExchange(exchange.id)}
                  className={`p-4 rounded-lg border-2 transition-all ${
                    selectedExchange === exchange.id
                      ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                      : 'border-gray-200 dark:border-gray-700 hover:border-blue-300'
                  }`}
                >
                  <div className="text-left">
                    <div className="font-semibold text-gray-900 dark:text-white">
                      {exchange.name}
                    </div>
                    <div className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                      {exchange.exchange_type} · {exchange.status}
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* 交易所详情 */}
        {selectedExchange && (
          <>
            <ExchangeStatus exchangeId={selectedExchange} />
            <MarketDataPanel exchangeId={selectedExchange} />
            <AccountPanel exchangeId={selectedExchange} />
          </>
        )}
      </div>
    </div>
  );
}

// 交易所状态组件
function ExchangeStatus({ exchangeId }: { exchangeId: string }) {
  const { status, loading, error, connect, disconnect } = useExchange(exchangeId);

  if (loading && !status) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <div className="text-gray-500">加载状态中...</div>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
          🔌 连接状态
        </h2>
        <button
          onClick={status?.connected ? disconnect : connect}
          disabled={loading}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            status?.connected
              ? 'bg-red-500 hover:bg-red-600 text-white'
              : 'bg-green-500 hover:bg-green-600 text-white'
          } disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          {loading ? '处理中...' : status?.connected ? '断开连接' : '连接'}
        </button>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400">
          {error}
        </div>
      )}

      {status && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <StatusCard
            label="连接状态"
            value={status.connected ? '已连接' : '未连接'}
            color={status.connected ? 'green' : 'red'}
          />
          <StatusCard label="运行模式" value={status.mode} color="blue" />
          <StatusCard
            label="行情连接"
            value={status.connections.market_data.connected ? '正常' : '断开'}
            color={status.connections.market_data.connected ? 'green' : 'gray'}
          />
          <StatusCard
            label="交易连接"
            value={status.connections.trading.connected ? '正常' : '断开'}
            color={status.connections.trading.connected ? 'green' : 'gray'}
          />
        </div>
      )}

      {status?.broker_info && (
        <div className="mt-4 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            柜台: {status.broker_info.broker_name} ({status.broker_info.broker_id})
          </div>
        </div>
      )}
    </div>
  );
}

// 状态卡片
function StatusCard({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color: 'green' | 'red' | 'blue' | 'gray';
}) {
  const colorClasses = {
    green: 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400',
    red: 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400',
    blue: 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400',
    gray: 'bg-gray-50 dark:bg-gray-700/50 text-gray-700 dark:text-gray-400',
  };

  return (
    <div className={`p-3 rounded-lg ${colorClasses[color]}`}>
      <div className="text-xs opacity-75 mb-1">{label}</div>
      <div className="font-semibold">{value}</div>
    </div>
  );
}

// 行情数据面板
function MarketDataPanel({ exchangeId }: { exchangeId: string }) {
  const { ticker, loading, error } = useMarketTicker(exchangeId, 'rb2501', 2000);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
      <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        📈 实时行情 (rb2501)
      </h2>

      {loading && !ticker && (
        <div className="text-gray-500">加载行情中...</div>
      )}

      {error && (
        <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400">
          {error}
        </div>
      )}

      {ticker && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <PriceCard label="最新价" value={ticker.last_price.toFixed(2)} />
          <PriceCard label="买价" value={ticker.bid_price.toFixed(2)} />
          <PriceCard label="卖价" value={ticker.ask_price.toFixed(2)} />
          <PriceCard
            label="涨跌幅"
            value={`${ticker.change_percent_24h.toFixed(2)}%`}
            color={ticker.change_percent_24h >= 0 ? 'green' : 'red'}
          />
          <PriceCard label="最高" value={ticker.high_24h.toFixed(2)} />
          <PriceCard label="最低" value={ticker.low_24h.toFixed(2)} />
          <PriceCard label="成交量" value={ticker.volume_24h.toFixed(0)} />
          <PriceCard
            label="涨跌"
            value={ticker.change_24h.toFixed(2)}
            color={ticker.change_24h >= 0 ? 'green' : 'red'}
          />
        </div>
      )}
    </div>
  );
}

// 价格卡片
function PriceCard({
  label,
  value,
  color = 'default',
}: {
  label: string;
  value: string;
  color?: 'default' | 'green' | 'red';
}) {
  const colorClass =
    color === 'green'
      ? 'text-green-600 dark:text-green-400'
      : color === 'red'
        ? 'text-red-600 dark:text-red-400'
        : 'text-gray-900 dark:text-white';

  return (
    <div className="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
      <div className="text-xs text-gray-600 dark:text-gray-400 mb-1">{label}</div>
      <div className={`text-lg font-semibold ${colorClass}`}>{value}</div>
    </div>
  );
}

// 账户面板
function AccountPanel({ exchangeId }: { exchangeId: string }) {
  const { account, loading, error } = useAccount(exchangeId, 5000);
  const { positions } = usePositions(exchangeId, 3000);

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      {/* 账户信息 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          💰 账户信息
        </h2>

        {loading && !account && (
          <div className="text-gray-500">加载账户中...</div>
        )}

        {error && (
          <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400">
            {error}
          </div>
        )}

        {account && (
          <div className="space-y-3">
            <div className="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700">
              <span className="text-gray-600 dark:text-gray-400">账户余额</span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ¥{account.balance.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700">
              <span className="text-gray-600 dark:text-gray-400">可用资金</span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ¥{account.available.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700">
              <span className="text-gray-600 dark:text-gray-400">冻结资金</span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ¥{account.frozen.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700">
              <span className="text-gray-600 dark:text-gray-400">权益</span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ¥{account.equity.toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between items-center py-2">
              <span className="text-gray-600 dark:text-gray-400">盈亏</span>
              <span
                className={`font-semibold ${
                  account.profit_loss >= 0
                    ? 'text-green-600 dark:text-green-400'
                    : 'text-red-600 dark:text-red-400'
                }`}
              >
                {account.profit_loss >= 0 ? '+' : ''}¥{account.profit_loss.toLocaleString()}
              </span>
            </div>
          </div>
        )}
      </div>

      {/* 持仓信息 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          📊 持仓列表
        </h2>

        {positions.length === 0 ? (
          <div className="text-gray-500 text-center py-8">暂无持仓</div>
        ) : (
          <div className="space-y-3">
            {positions.map((position, index) => (
              <div
                key={index}
                className="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg"
              >
                <div className="flex justify-between items-center mb-2">
                  <span className="font-semibold text-gray-900 dark:text-white">
                    {position.symbol}
                  </span>
                  <span
                    className={`text-xs px-2 py-1 rounded ${
                      position.direction === 'long'
                        ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                        : 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                    }`}
                  >
                    {position.direction === 'long' ? '多' : '空'}
                  </span>
                </div>
                <div className="text-sm text-gray-600 dark:text-gray-400 space-y-1">
                  <div>持仓: {position.volume} 手</div>
                  <div>均价: ¥{position.avg_price.toFixed(2)}</div>
                  <div>
                    盈亏:{' '}
                    <span
                      className={
                        position.profit_loss >= 0
                          ? 'text-green-600 dark:text-green-400'
                          : 'text-red-600 dark:text-red-400'
                      }
                    >
                      {position.profit_loss >= 0 ? '+' : ''}¥{position.profit_loss.toFixed(2)}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
