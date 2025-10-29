"use client";
import { useSearchParams } from "react-router-dom";

export default function TabButton({
  name,
  tabKey,
  disabled = false,
}: {
  name: string;
  tabKey?: string;
  disabled?: boolean;
}) {
  // rely on theme CSS variables
  const [searchParams, setSearchParams] = useSearchParams();
  const tab = searchParams.get("tab") || "positions";
  const active = tabKey ? tab === tabKey : false;
  return (
    <button
      onClick={() => {
        if (disabled || !tabKey) return;
        const params = new URLSearchParams(searchParams.toString());
        if (tabKey === "positions") params.delete("tab");
        else params.set("tab", tabKey);
        const next = params.toString();
        if (next) setSearchParams(params, { replace: true });
        else setSearchParams({}, { replace: true });
      }}
      aria-disabled={disabled}
      className={`flex-1 px-3 py-1.5 text-xs border-r-2 last:border-r-0 ${disabled ? "cursor-not-allowed" : ""}`}
      style={{
        borderColor: "var(--panel-border)",
        background: disabled
          ? "transparent"
          : active
            ? "#000000"
            : "transparent",
        color: disabled
          ? "var(--muted-text)"
          : active
            ? "#ffffff"
            : "var(--btn-inactive-fg)",
        fontWeight: active ? 600 : 400,
      }}
      type="button"
    >
      {name}
    </button>
  );
}
