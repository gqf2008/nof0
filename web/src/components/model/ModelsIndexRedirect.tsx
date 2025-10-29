"use client";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useLatestEquityMap } from "@/lib/api/hooks/useModelSnapshots";

export default function ModelsIndexRedirect() {
  const { map, isLoading } = useLatestEquityMap();
  const navigate = useNavigate();
  useEffect(() => {
    if (isLoading) return;
    const ids = Object.keys(map || {});
    if (ids.length > 0) {
      navigate(`/models/${encodeURIComponent(ids[0])}`, { replace: true });
    }
  }, [isLoading, map, navigate]);
  return null;
}
