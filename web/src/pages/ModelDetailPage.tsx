import { useMemo } from "react";
import { useParams } from "react-router-dom";
import ModelSelectorBar from "@/components/model/ModelSelectorBar";
import ModelStatsSummary from "@/components/model/ModelStatsSummary";
import ModelOpenPositions from "@/components/model/ModelOpenPositions";
import ModelRecentTradesTable from "@/components/model/ModelRecentTradesTable";
import ModelAnalyticsDetails from "@/components/model/ModelAnalyticsDetails";

export default function ModelDetailPage() {
  const params = useParams<"id">();
  const activeId = useMemo(() => {
    const raw = params.id || "";
    return decodeURIComponent(raw);
  }, [params.id]);

  if (!activeId) {
    return (
      <main className="min-h-screen w-full px-3 py-3 sm:px-4 sm:py-4 lg:px-8 lg:py-6">
        <div className="mx-auto w-full max-w-7xl space-y-3">
          <ModelSelectorBar />
          <div
            className="rounded-md border p-3 text-xs"
            style={{
              background: "var(--panel-bg)",
              borderColor: "var(--panel-border)",
              color: "var(--muted-text)",
            }}
          >
            未找到模型，请从列表中选择。
          </div>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen w-full px-3 py-3 sm:px-4 sm:py-4 lg:px-8 lg:py-6">
      <div className="mx-auto w-full max-w-7xl space-y-3">
        <ModelSelectorBar activeId={activeId} />
        <ModelStatsSummary modelId={activeId} />
        <ModelAnalyticsDetails modelId={activeId} />
        <div className="space-y-3">
          <div
            className="rounded-md border p-3"
            style={{
              background: "var(--panel-bg)",
              borderColor: "var(--panel-border)",
            }}
          >
            <ModelOpenPositions modelId={activeId} />
          </div>
          <div
            className="rounded-md border p-3"
            style={{
              background: "var(--panel-bg)",
              borderColor: "var(--panel-border)",
            }}
          >
            <ModelRecentTradesTable modelId={activeId} />
          </div>
        </div>
      </div>
    </main>
  );
}
