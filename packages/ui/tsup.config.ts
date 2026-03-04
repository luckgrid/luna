import { readdirSync } from "node:fs";
import { extname, resolve } from "node:path";
import { defineConfig } from "tsup";

const modules = ["action", "display", "form", "link", "feedback", "utils"];

function buildEntries() {
  const entries: Record<string, string> = {};
  const seenNames = new Set<string>();

  for (const moduleName of modules) {
    const modulePath = resolve(__dirname, `src/${moduleName}`);
    for (const file of readdirSync(modulePath)) {
      const extension = extname(file);
      if (extension !== ".ts" && extension !== ".tsx") {
        continue;
      }
      if (moduleName === "utils" && file !== "cx.ts") {
        continue;
      }

      const baseName = file.slice(0, -extension.length);
      const entryName = moduleName === "utils" && baseName === "cx" ? "utils" : baseName;

      if (seenNames.has(entryName)) {
        throw new Error(`Duplicate UI entry name detected: ${entryName}`);
      }

      seenNames.add(entryName);
      entries[entryName] = resolve(modulePath, file);
    }
  }

  return entries;
}

export default defineConfig({
  entry: buildEntries(),
  format: ["esm"],
  dts: {
    compilerOptions: {
      composite: false,
    },
  },
  sourcemap: true,
  splitting: false,
  clean: true,
  external: ["solid-js"],
});
