import dsConfig from "@luna/ds/vite.config";
import { solidStart } from "@solidjs/start/config";
import { defineConfig, mergeConfig } from "vite";

export default defineConfig(
  mergeConfig(dsConfig, {
    plugins: [solidStart()],
    server: {
      port: parseInt(process.env.WEB_PORT || "3000", 10),
    },
    nitro: {
      runtimeConfig: {
        apiBaseUrl: process.env.API_BASE_URL || "http://localhost:8080",
      },
      public: {
        apiBaseUrl: process.env.API_BASE_URL || "http://localhost:8080",
      },
    },
  }),
);
