import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");

function run(cmd: string[]): number {
  const proc = Bun.spawnSync(cmd, {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  });
  return proc.exitCode ?? 1;
}

let code = run(["bun", "./src/main.ts"]);
if (code !== 0) process.exit(code);
code = run(["bun", "x", "vite", "build"]);
process.exit(code);
