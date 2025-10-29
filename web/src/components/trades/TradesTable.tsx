"use client";
import { useMemo } from "react";
import { useTrades } from "@/lib/api/hooks/useTrades";
import { useSearchParams } from "react-router-dom";
import { fmtUSD } from "@/lib/utils/formatters";
import ErrorBanner from "@/components/ui/ErrorBanner";
import { SkeletonRow } from "@/components/ui/Skeleton";
import { getModelName, getModelColor } from "@/lib/model/meta";
import { ModelLogoChip } from "@/components/shared/ModelLogo";
import type { TradeRow } from "@/lib/api/hooks/useTrades";

export default function TradesTable() {
  const { trades, isLoading, isError } = useTrades();
  const [search, setSearch] = useSearchParams();

  const qModel = (search.get("model") || "ALL").toLowerCase();

  const all = useMemo(() => {
    const arr = [...trades];
    arr.sort(
      (a, b) =>
        Number(b.exit_time || b.entry_time) -
        Number(a.exit_time || a.entry_time),
    );
    // show last 100 by default to match screenshot
    return arr.slice(0, 100);
  }, [trades]);

  const rows = useMemo(() => {
    return all.filter((t) =>
      qModel === "all" ? true : (t.model_id || "").toLowerCase() === qModel,
    );
  }, [all, qModel]);

  const models = useMemo(() => {
    const ids = Array.from(new Set(trades.map((t) => t.model_id))).filter(
      Boolean,
    ) as string[];
    return ids.sort((a, b) => a.localeCompare(b));
  }, [trades]);

  return (
    <div
      className={`mt-3 rounded-md border terminal-text text-[13px] sm:text-xs leading-relaxed`}
      style={{
        background: "var(--panel-bg)",
        borderColor: "var(--panel-border)",
      }}
    >
      {/* Header: filter + count */}
      <div
        className="flex items-center justify-between px-3 py-2 border-b"
        style={{ borderColor: "var(--panel-border)" }}
      >
        <div
          className="flex items-center gap-2 text-sm ui-sans"
          style={{ color: "var(--foreground)" }}
        >
          <span
            className="font-semibold"
            style={{ color: "var(--foreground)" }}
          >
            筛选：
          </span>
          <select
            className="rounded px-2 py-1 text-xs"
            style={{
              background: "var(--panel-bg)",
              border: "1px solid var(--panel-border)",
              color: "var(--foreground)",
            }}
            value={search.get("model") || "ALL"}
            onChange={(e) => setQuery("model", e.target.value)}
          >
            <option value="ALL">全部模型</option>
            {models.map((m) => (
              <option key={m} value={m}>
                {getModelName(m)}
              </option>
            ))}
          </select>
        </div>
        <div
          className="text-xs font-semibold tabular-nums ui-sans"
          style={{ color: "var(--muted-text)" }}
        >
          展示最近 100 笔成交
        </div>
      </div>

      <ErrorBanner
        message={isError ? "成交记录数据源暂时不可用，请稍后重试。" : undefined}
      />

      {/* List */}
      <div
        className="divide-y"
        style={{
          borderColor:
            "color-mix(in oklab, var(--panel-border) 50%, transparent)",
        }}
      >
        {isLoading ? (
          <div className="p-3">
            <SkeletonRow cols={6} />
            <SkeletonRow cols={6} />
            <SkeletonRow cols={6} />
          </div>
        ) : rows.length ? (
          rows.map((t) => <TradeItem key={t.id} t={t} />)
        ) : (
          <div className="p-3 text-xs" style={{ color: "var(--muted-text)" }}>
            暂无数据
          </div>
        )}
      </div>
    </div>
  );

  function setQuery(k: string, v: string) {
    const params = new URLSearchParams(search.toString());
    if (v === "ALL") params.delete(k);
    else params.set(k, v);
    const next = params.toString();
    if (next) setSearch(params, { replace: true });
    else setSearch({}, { replace: true });
  }
}

function TradeItem({ t }: { t: TradeRow }) {
  const sideColor = t.side === "long" ? "#16a34a" : "#ef4444"; // green-600 / red-500
  const modelColor = getModelColor(t.model_id || "");
  const symbol = (t.symbol || "").toUpperCase();
  const qty = t.quantity;
  const absQty = Math.abs(qty ?? 0);
  const entry = t.entry_price;
  const exit = t.exit_price;
  const notionalIn = absQty * (entry ?? 0);
  const notionalOut = absQty * (exit ?? 0);
  const hold = humanHold(t.entry_time, t.exit_time);
  const when = humanTime(t.exit_time || t.entry_time);

  return (
    <div className="px-3 py-3">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div
            className="mb-1 terminal-text text-[13px] sm:text-xs leading-relaxed"
            style={{ color: "var(--foreground)" }}
          >
            <span className="mr-1 align-middle">
              <ModelLogoChip modelId={t.model_id} size="sm" />
            </span>
            <b style={{ color: modelColor }}>{getModelName(t.model_id)}</b>
            <span> 完成了一笔 </span>
            <b style={{ color: sideColor }}>{sideZh(t.side)}</b>
            <span> 交易，标的 </span>
            <span className="inline-flex items-center gap-1 font-semibold">
              <CoinIcon symbol={symbol} />
              <span>{symbol}!</span>
            </span>
          </div>
        </div>
        <div
          className="text-xs whitespace-nowrap tabular-nums"
          style={{ color: "var(--muted-text)" }}
        >
          {when}
        </div>
      </div>

      <div
        className="mt-1 grid grid-cols-1 gap-0.5 text-[13px] sm:text-xs leading-relaxed sm:grid-cols-2"
        style={{ color: "var(--foreground)" }}
      >
        <div>
          价格：{fmtPrice(entry)} → {fmtPrice(exit)}
        </div>
        <div>
          数量：<span className="tabular-nums">{fmtNumber(qty, 2)}</span>
        </div>
        <div>
          名义金额：{fmtUSD(notionalIn)} → {fmtUSD(notionalOut)}
        </div>
        <div>持有时长：{hold}</div>
      </div>

      <div className="mt-2 flex items-baseline gap-2">
        <span
          className="ui-sans text-[12px] sm:text-sm"
          style={{ color: "var(--muted-text)" }}
        >
          净盈亏：
        </span>
        <span
          className="terminal-text tabular-nums text-[13px] sm:text-sm font-semibold"
          style={{ color: pnlColor(t.realized_net_pnl) }}
        >
          {fmtUSD(t.realized_net_pnl)}
        </span>
      </div>
    </div>
  );
}

function CoinIcon({ symbol }: { symbol: string }) {
  const src = coinSrc(symbol);
  if (!src) return <span className="inline-block text-[12px]">{symbol}</span>;
  return (
    <span className="logo-chip logo-chip-md overflow-hidden">
      {/* use img to keep it simple; public/ path is already safe */}
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img src={src} alt={symbol} width={16} height={16} />
    </span>
  );
}

function coinSrc(symbol: string): string | undefined {
  const k = symbol.toUpperCase();
  switch (k) {
    case "BTC":
      return "/coins/btc.svg";
    case "ETH":
      return "/coins/eth.svg";
    case "SOL":
      return "/coins/sol.svg";
    case "BNB":
      return "/coins/bnb.svg";
    case "DOGE":
      return "/coins/doge.svg";
    case "XRP":
      return "/coins/xrp.svg";
    default:
      return undefined;
  }
}

function pnlColor(n?: number | null) {
  if (n == null || Number.isNaN(n)) return "var(--muted-text)";
  return n > 0 ? "#22c55e" : n < 0 ? "#ef4444" : "var(--muted-text)";
}

function humanTime(sec?: number) {
  if (!sec) return "--";
  const d = new Date(sec > 1e12 ? sec : sec * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function humanHold(entry?: number, exit?: number) {
  if (!entry) return "—";
  const a = entry > 1e12 ? entry : entry * 1000;
  const b = exit ? (exit > 1e12 ? exit : exit * 1000) : Date.now();
  const ms = Math.max(0, b - a);
  const m = Math.floor(ms / 60000);
  const h = Math.floor(m / 60);
  const mm = m % 60;
  return h ? `${h}小时${mm}分` : `${mm}分`;
}

function fmtPrice(n?: number | null) {
  if (n == null || Number.isNaN(n)) return "--";
  const abs = Math.abs(n);
  const digits = abs >= 1000 ? 1 : abs >= 100 ? 2 : abs >= 1 ? 4 : 5;
  return `$${n.toFixed(digits)}`;
}

function fmtNumber(n?: number | null, digits = 2) {
  if (n == null || Number.isNaN(n)) return "--";
  const sign = n < 0 ? "-" : "";
  const v = Math.abs(n).toLocaleString(undefined, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
  return `${sign}${v}`;
}

function sideZh(s?: string) {
  return s === "long" ? "做多" : s === "short" ? "做空" : String(s ?? "—");
}
