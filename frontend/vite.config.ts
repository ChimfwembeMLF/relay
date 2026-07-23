import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  base: "/",
  resolve: {
    alias: {
      "@": path.resolve(root, "./src"),
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": "http://localhost:8080",
      "/systems": "http://localhost:8080",
      "/payments": "http://localhost:8080",
      "/wallets": "http://localhost:8080",
      "/transactions": "http://localhost:8080",
      "/invoices": "http://localhost:8080",
      "/reports": "http://localhost:8080",
      "/webhook-endpoints": "http://localhost:8080",
      "/auth": "http://localhost:8080",
      "/admin": "http://localhost:8080",
      "/swagger-ui": "http://localhost:8080",
      "/api-docs": "http://localhost:8080",
      "/docs": "http://localhost:8080",
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
