import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    strictPort: true,
    host: host || false,
    port: 1420,
    watch: {
      ignored: [
        "**/src-tauri/**",
        "**/plugins/**",
        "**/node_modules/**",
      ],
    },
    fs: {
      strict: true,
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari14",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
