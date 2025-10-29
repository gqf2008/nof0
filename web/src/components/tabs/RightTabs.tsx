"use client";
import { useSearchParams } from "react-router-dom";
import { PositionsPanel } from "@/components/tabs/PositionsPanel";
import TradesTable from "@/components/trades/TradesTable";
import AnalyticsPanel from "@/components/analytics/AnalyticsPanel";
import ReadmePanel from "@/components/tabs/ReadmePanel";
import ModelChatPanel from "@/components/chat/ModelChatPanel";

export default function RightTabs() {
  const [search] = useSearchParams();
  const tab = search.get("tab") || "positions";
  if (tab === "chat") return <ModelChatPanel />;
  if (tab === "trades") return <TradesTable />;
  if (tab === "analytics") return <AnalyticsPanel />;
  if (tab === "readme") return <ReadmePanel />;
  return <PositionsPanel />;
}
