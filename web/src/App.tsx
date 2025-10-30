import { useRoutes, Navigate } from "react-router-dom";
import { useEffect, useState } from "react";
import Header from "@/components/layout/Header";
import PageActivityProvider from "@/components/providers/PageActivityProvider";
import SWRProvider from "@/components/providers/SWRProvider";
import ThemeProvider from "@/components/theme/ThemeProvider";
import HomePage from "@/pages/HomePage";
import LeaderboardPage from "@/pages/LeaderboardPage";
import ModelDetailPage from "@/pages/ModelDetailPage";
import ModelsIndexPage from "@/pages/ModelsIndexPage";
import CryptoExchangePage from "@/pages/CryptoExchangePage";
import CtpMonitorPage from "@/pages/CtpMonitorPage";
import GenericExchangePage from "@/pages/GenericExchangePage";

interface ExchangeConfig {
  id: string;
  name: string;
  route: string;
  enabled: boolean;
}

interface ExchangesConfigResponse {
  exchanges: ExchangeConfig[];
}

// 交易所ID到组件的映射
const EXCHANGE_COMPONENTS: Record<string, React.ComponentType> = {
  crypto: CryptoExchangePage,
  ctp: CtpMonitorPage,
};

function AppRoutes() {
  const [exchangeRoutes, setExchangeRoutes] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // 从后端获取交易所配置
    fetch("/api/config/exchanges")
      .then((res) => res.json())
      .then((data: ExchangesConfigResponse) => {
        // 为启用的交易所生成路由配置
        const routes = data.exchanges
          .filter((e) => e.enabled)
          .map((exchange) => {
            const Component = EXCHANGE_COMPONENTS[exchange.id] || GenericExchangePage;
            return {
              path: exchange.route,
              element: <Component />,
            };
          });
        setExchangeRoutes(routes);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load exchange routes:", err);
        setLoading(false);
      });
  }, []);

  const routes = useRoutes([
    { path: "/", element: <HomePage /> },
    ...exchangeRoutes,
    { path: "/leaderboard", element: <LeaderboardPage /> },
    { path: "/models", element: <ModelsIndexPage /> },
    { path: "/models/:id", element: <ModelDetailPage /> },
    { path: "*", element: <Navigate to="/" replace /> },
  ]);

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 flex items-center justify-center">
        <div className="text-white text-xl">加载中...</div>
      </div>
    );
  }

  return routes;
}

export default function App() {
  return (
    <>
      <ThemeProvider />
      <PageActivityProvider />
      <SWRProvider>
        <div className="min-h-screen">
          <Header />
          <AppRoutes />
        </div>
      </SWRProvider>
    </>
  );
}
