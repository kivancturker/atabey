import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Fixed port: tauri.conf.json devUrl points here.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, host: "127.0.0.1" },
  build: { target: "es2023", outDir: "dist" },
  test: { environment: "jsdom", include: ["src/**/*.test.{ts,tsx}"] },
});
