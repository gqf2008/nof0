"use client";
import { useMemo } from "react";
import { useSearchParams } from "react-router-dom";

const SIDES = ["ALL", "LONG", "SHORT"] as const;

export default function PositionsFilter({
  models,
  symbols,
}: {
  models: string[];
  symbols: string[];
}) {
  // Use CSS variables in styles instead of theme branching
  const [search, setSearch] = useSearchParams();

  const model = search.get("model") || "ALL";
  const symbol = search.get("symbol") || "ALL";
  const side = search.get("side") || "ALL";

  const modelOptions = useMemo(() => ["ALL", ...models], [models]);
  const symbolOptions = useMemo(() => ["ALL", ...symbols], [symbols]);

  function setQuery(next: Record<string, string>) {
    const params = new URLSearchParams(search.toString());
    for (const [k, v] of Object.entries(next)) {
      if (!v || v === "ALL") params.delete(k);
      else params.set(k, v);
    }
    const nextSearch = params.toString();
    if (nextSearch) setSearch(params, { replace: true });
    else setSearch({}, { replace: true });
  }

  return (
    <div
      className={`pb-3 flex flex-wrap items-center gap-2 text-xs`}
      style={{ color: "var(--muted-text)" }}
    >
      <Select
        label="模型"
        value={model}
        options={modelOptions}
        onChange={(v) => setQuery({ model: v })}
      />
      <Select
        label="币种"
        value={symbol}
        options={symbolOptions}
        onChange={(v) => setQuery({ symbol: v })}
      />
      <Select
        label="方向"
        value={side}
        options={SIDES as unknown as string[]}
        onChange={(v) => setQuery({ side: v })}
      />
    </div>
  );
}

function Select({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  onChange: (v: string) => void;
}) {
  return (
    <label className="flex items-center gap-1">
      <span style={{ color: "var(--muted-text)" }}>{label}</span>
      <select
        className={`px-2 py-1 text-xs`}
        style={{
          border: "1px solid var(--panel-border)",
          background: "var(--panel-bg)",
          color: "var(--foreground)",
        }}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {options.map((opt) => (
          <option key={opt} value={opt}>
            {opt}
          </option>
        ))}
      </select>
    </label>
  );
}
