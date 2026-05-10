import dsConfig from "@luna/ds/vite.config";
import { solidStart } from "@solidjs/start/config";
import { nitroV2Plugin } from "@solidjs/vite-plugin-nitro-2";
import { defineConfig, mergeConfig } from "vite";

/** Python FastAPI + Pydantic AI (`apps/api`), default `moon run api:dev` port 8080 — not Nitro itself. */
const apiBaseUrl = process.env.API_BASE_URL || "http://localhost:8080";

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
      /** Fail fast if `APP_PORT` is taken (e.g. stray Node on 3000) instead of binding another port quietly. */
      strictPort: true,
    },
    // Client bundle: same URL as Nitro `public` (FastAPI on 8080).
    define: {
      "import.meta.env.NITRO_PUBLIC_API_BASE_URL": JSON.stringify(apiBaseUrl),
    },
  }),
);
