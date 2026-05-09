import { watch } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { main } from "../src/main";

const dir = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(dir, "..");

async function runSsg(): Promise<void> {
  await main({ root });
}

let debounce: ReturnType<typeof setTimeout> | undefined;
function scheduleSsg(): void {
  clearTimeout(debounce);
  debounce = setTimeout(() => {
    void runSsg().catch((e) => console.error("[site]", e));
  }, 150);
}

await runSsg().catch((e) => {
  console.error(e);
  process.exit(1);
});

watch(path.join(root, "src"), { recursive: true }, () => scheduleSsg());

const vite = Bun.spawn(["bun", "x", "vite", "build", "--watch"], {
  cwd: root,
  stdout: "inherit",
  stderr: "inherit",
});

const port = Number(process.env.WEB_PORT ?? "3000");
console.log(`serving ${path.join(root, "dist")} on http://localhost:${port}`);

function resolveDistPath(pathname: string): string {
  let p = pathname;
  if (p === "/") return "index.html";
  p = p.replace(/^\//, "");
  if (p.endsWith("/")) return `${p}index.html`;
  if (!p.includes(".")) return `${p}/index.html`;
  return p;
}

Bun.serve({
  port,
  async fetch(req) {
    const url = new URL(req.url);
    const rel = resolveDistPath(url.pathname);
    const filePath = path.join(root, "dist", rel);
    const file = Bun.file(filePath);
    if (await file.exists()) {
      return new Response(file);
    }

    const fallback = Bun.file(path.join(root, "dist", `${url.pathname.replace(/^\//, "")}.html`));
    if (await fallback.exists()) {
      return new Response(fallback);
    }

    return new Response("Not found", { status: 404 });
  },
});

function cleanup(): void {
  if (typeof vite.kill === "function") vite.kill();
}

process.on("SIGINT", () => {
  cleanup();
  process.exit(0);
});

process.on("SIGTERM", () => {
  cleanup();
  process.exit(0);
});
