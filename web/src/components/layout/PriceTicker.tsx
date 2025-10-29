"use client";
import { useMemo } from "react";
import { useCryptoPrices } from "@/lib/api/hooks/useCryptoPrices";
import { useAccountTotals } from "@/lib/api/hooks/useAccountTotals";
import { fmtUSD } from "@/lib/utils/formatters";
import { getModelName } from "@/lib/model/meta";
import { ModelLogoChip } from "@/components/shared/ModelLogo";

const ORDER = ["BTC", "ETH", "SOL", "BNB", "DOGE", "XRP"] as const;

export default function PriceTicker() {
  const { prices } = useCryptoPrices();
  const { data } = useAccountTotals();

  const list = useMemo(() => {
    const vals = Object.values(prices);
    return ORDER.map((s) => vals.find((v) => v.symbol === s)).filter(
      Boolean,
    ) as typeof vals;
  }, [prices]);

  const { highest, lowest } = useMemo(() => {
    if (!data?.accountTotals?.length) return { highest: null, lowest: null };
    
    const rows = data.accountTotals;
    let highest = rows[0];
    let lowest = rows[0];
    
    for (const row of rows) {
      const highestValue = highest.account_value ?? 0;
      const lowestValue = lowest.account_value ?? 0;
      const currentValue = row.account_value ?? 0;
      
      if (currentValue > highestValue) {
        highest = row;
      }
      if (currentValue < lowestValue) {
        lowest = row;
      }
    }
    
    return { highest, lowest };
  }, [data]);

  return (
    <div
      className={`w-full h-[var(--ticker-h)] border-b-2`}
      style={{
        background: "var(--panel-bg)",
        borderColor: "var(--panel-border)",
      }}
    >
      <div className="h-full overflow-x-auto overflow-y-hidden px-3 flex items-center justify-between">
        <div className="flex items-center">
          {list.map((p, idx) => (
            <>
              <div
                key={p.symbol}
                className="ticker-item inline-flex items-center gap-2 px-3 py-2"
              >
                {/* coin icon */}
                <img
                  src={`/coins/${String(p.symbol || "").toLowerCase()}.svg`}
                  alt={p.symbol}
                  width={18}
                  height={18}
                  style={{ display: "block" }}
                />
                <div className="flex flex-col justify-center">
                  <span className="text-[11px] font-semibold leading-tight" style={{ color: "var(--muted-text)" }}>
                    {p.symbol}
                  </span>
                  <span className="text-[12px] font-semibold leading-tight" style={{ color: "var(--foreground)" }}>
                    {fmtUSD(p.price)}
                  </span>
                </div>
              </div>
              {idx < list.length - 1 && (
                <span style={{ color: "#d0d0d0", margin: "0 4px" }}>|</span>
              )}
            </>
          ))}
        </div>
        
        {/* 最高和最低收益率 */}
        {(highest || lowest) && (
          <div className="flex items-center gap-4 px-3 whitespace-nowrap">
            {highest && (
              <div className="flex items-center gap-2">
                <span className="text-[11px] font-semibold uppercase" style={{ color: "var(--muted-text)" }}>
                  HIGHEST:
                </span>
                <ModelLogoChip modelId={highest.model_id} size="sm" />
                <span className="text-[11px] font-semibold uppercase" style={{ color: "var(--foreground)" }}>
                  {getModelName(highest.model_id)}
                </span>
                <span className="text-[11px] font-semibold" style={{ color: "var(--foreground)" }}>
                  {fmtUSD(highest.account_value ?? 0)}
                </span>
                <span 
                  className="text-[11px] font-semibold" 
                  style={{ 
                    color: (highest.return_pct ?? 0) >= 0 ? "#16a34a" : "#dc2626"
                  }}
                >
                  {(highest.return_pct ?? 0) >= 0 ? "+" : ""}{((highest.return_pct ?? 0) * 100).toFixed(2)}%
                </span>
              </div>
            )}
            <span style={{ color: "#d0d0d0" }}>|</span>
            {lowest && (
              <div className="flex items-center gap-2">
                <span className="text-[11px] font-semibold uppercase" style={{ color: "var(--muted-text)" }}>
                  LOWEST:
                </span>
                <ModelLogoChip modelId={lowest.model_id} size="sm" />
                <span className="text-[11px] font-semibold uppercase" style={{ color: "var(--foreground)" }}>
                  {getModelName(lowest.model_id)}
                </span>
                <span className="text-[11px] font-semibold" style={{ color: "var(--foreground)" }}>
                  {fmtUSD(lowest.account_value ?? 0)}
                </span>
                <span 
                  className="text-[11px] font-semibold" 
                  style={{ 
                    color: (lowest.return_pct ?? 0) >= 0 ? "#16a34a" : "#dc2626"
                  }}
                >
                  {(lowest.return_pct ?? 0) >= 0 ? "+" : ""}{((lowest.return_pct ?? 0) * 100).toFixed(2)}%
                </span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
