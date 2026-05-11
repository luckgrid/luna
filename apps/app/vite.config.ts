import dsConfig from "@luna/ds/vite.config";
import { solidStart } from "@solidjs/start/config";
import { nitroV2Plugin } from "@solidjs/vite-plugin-nitro-2";
import { defineConfig, mergeConfig } from "vite";

/** Python FastAPI (`apps/api`); default `API_PORT` is 8000 (typical Uvicorn) — not Nitro. */
const apiBaseUrl = process.env.API_BASE_URL || "http://localhost:8000";

export default defineConfig(
  mergeConfig(dsConfig, {
    plugins: [
      solidStart(),
      // Nitro is SolidStart’s server/runtime; runtimeConfig still points at the separate Python API.
      nitroV2Plugin({
        runtimeConfig: {
          apiBaseUrl,
          public: { apiBaseUrl },
        },
      }),
    ],
    server: {
      port: parseInt(process.env.APP_PORT || "3000", 10),
      /** Fail fast if `APP_PORT` is taken instead of binding another port quietly. */
      strictPort: true,
    },
    // Client bundle: Nitro `public`; browser calls FastAPI on API_PORT (default 8000).
    define: {
      "import.meta.env.NITRO_PUBLIC_API_BASE_URL": JSON.stringify(apiBaseUrl),
    },
  }),
);
