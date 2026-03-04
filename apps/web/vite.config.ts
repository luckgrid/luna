import dsConfig from "@luna/ds/vite.config";
import { solidStart } from "@solidjs/start/config";
import { nitro } from "nitro/vite";
import type { PluginOption } from "vite";
import { defineConfig, mergeConfig } from "vite";

export default defineConfig(
  mergeConfig(dsConfig, {
    plugins: [solidStart(), nitro() as unknown as PluginOption],
  }),
);
