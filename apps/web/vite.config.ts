import path from "node:path";
import { fileURLToPath } from "node:url";

import dsConfig from "@luna/ds/vite.config";
import { defineConfig, mergeConfig } from "vite";

const dir = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig(
  mergeConfig(
    dsConfig,
    defineConfig({
      root: dir,
      publicDir: false,
      build: {
        emptyOutDir: false,
        outDir: "dist",
        rollupOptions: {
          input: path.join(dir, "src/styles.css"),
          output: {
            assetFileNames: "styles[extname]",
          },
        },
      },
    }),
  ),
);
