import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5173,
    open: false,
    proxy: {
      "/api": {
        target: process.env.VITE_DEV_PROXY_TARGET || "http://127.0.0.1:8788",
        changeOrigin: true,
      },
    },
  },
});
