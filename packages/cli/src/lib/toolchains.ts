import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { die } from "./term";
import { spawnText } from "./process";

// --------------------
// proto
// --------------------

export function protoHasOutdatedPins(): boolean {
  const r = Bun.spawnSync(["proto", "outdated", "--json"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const text = new TextDecoder().decode(r.stdout).trim();
  if (!text) die("proto outdated --json returned empty output (is proto in PATH?)");
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    die("proto outdated --json returned invalid JSON");
  }
  if (typeof data !== "object" || data === null) die("proto outdated --json: unexpected shape");
  const entries = Object.values(data);
  return entries.some((x) => {
    if (typeof x !== "object" || x === null) return false;
    if (!("is_outdated" in x)) return false;
    return Reflect.get(x, "is_outdated") === true;
  });
}

export function printProtoOutdated(): void {
  Bun.spawnSync(["proto", "outdated"], { stdout: "inherit", stderr: "inherit", stdin: "ignore" });
}

// --------------------
// bun
// --------------------

/**
 * True when captured `bun outdated --recursive` output contains outdated rows
 * (same heuristics as the legacy shell helpers).
 */
export function bunWorkspaceOutdatedFromOutput(out: string): boolean {
  const headerRe = /^(│\s+Package\s+│\s+Current\s+│|\|\s*Package\s*\|\s*Current\s*\|)/m;
  if (!headerRe.test(out)) return false;

  const rowRe = /^\|\s+[^|]+\|\s+[^|]+\|\s+[^|]+\|\s+[^|]+\|\s+[^|]+\|\s*$/gm;
  const rows = out.match(rowRe) ?? [];
  for (const line of rows) {
    if (/^\|\s+Package\s+\|/.test(line)) continue;
    return true;
  }
  return false;
}

export function captureBunOutdatedRecursive(repoRoot: string): string {
  return spawnText(["bun", "outdated", "--recursive"], { cwd: repoRoot });
}

export type PrereleaseBumpRow = { pkg: string; cwd: string };

function bunTrimCell(s: string): string {
  return s.replace(/\s+$/g, "").replace(/^\s+/g, "");
}

function resolveWorkspaceDir(repoRoot: string, workspaceLabel: string): string | null {
  for (const top of ["apps", "packages"] as const) {
    const base = join(repoRoot, top);
    if (!existsSync(base)) continue;
    for (const ent of readdirSync(base, { withFileTypes: true })) {
      if (!ent.isDirectory()) continue;
      const pkgPath = join(base, ent.name, "package.json");
      if (!existsSync(pkgPath)) continue;
      try {
        const raw = readFileSync(pkgPath, "utf8");
        const j: unknown = JSON.parse(raw);
        if (typeof j === "object" && j !== null && "name" in j) {
          const rec = j as Record<string, unknown>;
          if (typeof rec.name === "string" && rec.name === workspaceLabel)
            return join(base, ent.name);
        }
      } catch {
        /* ignore */
      }
    }
  }
  return null;
}

/** Rows where Current === Update but Latest differs — needs `bun add pkg@latest` per workspace. */
export function collectPrereleaseBumps(stdin: string, repoRoot: string): PrereleaseBumpRow[] {
  const rows: PrereleaseBumpRow[] = [];
  for (const line of stdin.split("\n")) {
    const m = /^\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|/.exec(line);
    if (!m) continue;
    const pkg = bunTrimCell(m[1]);
    if (pkg === "Package" || pkg.startsWith("-")) continue;
    const current = bunTrimCell(m[2]);
    const update = bunTrimCell(m[3]);
    const latest = bunTrimCell(m[4]);
    const ws = bunTrimCell(m[5]);
    if (current === update && latest !== current) {
      const cwd = resolveWorkspaceDir(repoRoot, ws);
      if (cwd) rows.push({ pkg, cwd });
    }
  }
  return rows;
}

export function runPrereleaseBumps(rows: PrereleaseBumpRow[]): number {
  for (const { pkg, cwd } of rows) {
    const r = Bun.spawnSync(["bun", "add", `${pkg}@latest`, "--ignore-scripts"], {
      cwd,
      stdin: "ignore",
      stdout: "inherit",
      stderr: "inherit",
    });
    if (r.exitCode !== 0) return r.exitCode ?? 1;
  }
  return 0;
}

// --------------------
// python / uv
// --------------------

/** Python / uv: `uv lock --upgrade --dry-run` lines that indicate upgrades. */
export function uvLockHasUpgradesFromOutput(out: string): boolean {
  return /^Update /m.test(out);
}

export function captureUvLockDryRun(uvProjectRoot: string): string {
  return spawnText(["uv", "lock", "--upgrade", "--dry-run"], { cwd: uvProjectRoot });
}
