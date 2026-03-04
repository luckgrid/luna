import { resolve } from "node:path";
import { solidStart } from "@solidjs/start/config";
import { nitro } from "nitro/vite";
import UnoCSS from "unocss/vite";
import type { PluginOption } from "vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [
    UnoCSS({
      configFile: resolve(__dirname, "../../packages/ds/uno.config.ts"),
    }),
    solidStart(),
    nitro() as unknown as PluginOption,
  ],
});
