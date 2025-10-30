import { Suspense } from "react";
import PriceTicker from "@/components/layout/PriceTicker";
import AccountValueChart from "@/components/chart/AccountValueChart";
import RightTabs from "@/components/tabs/RightTabs";
import TabButton from "@/components/tabs/TabButton";
import ModelAccountsPanel from "@/components/model/ModelAccountsPanel";
import { ExchangeProvider } from "@/contexts/ExchangeContext";

export default function CryptoExchangePage() {
  return (
    <ExchangeProvider exchangeId="crypto">
      <main className="w-full terminal-scan flex flex-col h-[calc(100vh-var(--header-h))]">
      <PriceTicker />
      <section className="grid grid-cols-1 lg:grid-cols-3 overflow-hidden h-[calc(100vh-var(--header-h)-var(--ticker-h))]">
        <div className="lg:col-span-2 h-full flex flex-col border-r-2 pr-2" style={{ borderColor: "var(--panel-border)" }}>
          <AccountValueChart />
          {/* AI模型账户面板 */}
          <div className="overflow-y-auto px-3 pb-4">
            <Suspense fallback={<div className="text-xs text-zinc-500">加载AI模型账户...</div>}>
              <ModelAccountsPanel />
            </Suspense>
          </div>
        </div>
        <div className="lg:col-span-1 h-full overflow-hidden flex flex-col">
          <Suspense fallback={<div className="text-xs text-zinc-500">加载标签…</div>}>
            <div className="flex items-stretch justify-between border-r-2 border-b-2 overflow-hidden" style={{ borderColor: "var(--panel-border)" }}>
              <div className="flex items-stretch flex-1">
                <TabButton name="持仓" tabKey="positions" />
                <TabButton name="模型对话" tabKey="chat" />
                <TabButton name="成交" tabKey="trades" />
                <TabButton name="分析" disabled />
                <TabButton name="README.md" tabKey="readme" />
              </div>
            </div>
          </Suspense>
          <div className="flex-1 overflow-y-auto px-3">
            <Suspense fallback={<div className="text-xs text-zinc-500">加载数据…</div>}>
              <RightTabs />
            </Suspense>
          </div>
        </div>
      </section>
    </main>
    </ExchangeProvider>
  );
}
