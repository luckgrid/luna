import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** Read `@luna/cli` version from this package's `package.json` (no build step). */
export function readCliPackageVersion(): string {
  const here = dirname(fileURLToPath(import.meta.url)); // .../src/commands
  const pkgPath = join(here, "..", "..", "package.json");
  try {
    const j: unknown = JSON.parse(readFileSync(pkgPath, "utf8"));
    if (typeof j === "object" && j !== null && "version" in j) {
      const v = (j as { version?: unknown }).version;
      if (typeof v === "string") return v;
    }
  } catch {
    /* ignore */
  }
  return "0.0.0";
}
