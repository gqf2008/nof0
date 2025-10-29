import { Navigate, Route, Routes } from "react-router-dom";
import Header from "@/components/layout/Header";
import PageActivityProvider from "@/components/providers/PageActivityProvider";
import SWRProvider from "@/components/providers/SWRProvider";
import ThemeProvider from "@/components/theme/ThemeProvider";
import HomePage from "@/pages/HomePage";
import LeaderboardPage from "@/pages/LeaderboardPage";
import ModelDetailPage from "@/pages/ModelDetailPage";
import ModelsIndexPage from "@/pages/ModelsIndexPage";

export default function App() {
  return (
    <>
      <ThemeProvider />
      <PageActivityProvider />
      <SWRProvider>
        <div className="min-h-screen">
          <Header />
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/leaderboard" element={<LeaderboardPage />} />
            <Route path="/models" element={<ModelsIndexPage />} />
            <Route path="/models/:id" element={<ModelDetailPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </div>
      </SWRProvider>
    </>
  );
}
