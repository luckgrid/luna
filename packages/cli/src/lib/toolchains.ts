import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { die } from "./term";
import { spawnExit, spawnText } from "./process";

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
    const r = Bun.spawnSync(["bun", "add", `${pkg}@latest`], {
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

// --------------------
// go
// --------------------

function goMust0(code: number, step: string): void {
  if (code !== 0) {
    console.error(`error: ${step} failed (exit ${code})`);
    process.exit(code);
  }
}

const goModRequireLineRe = /^\s*([^/\s][^\s]+)\s+(v[0-9].*)/;

/** Module paths from go.mod require blocks (direct + indirect), one per line. */
export function goModRequiredModulePathsFromFile(gomodPath: string): string[] {
  let raw: string;
  try {
    raw = readFileSync(gomodPath, "utf8");
  } catch {
    return [];
  }
  const paths = new Set<string>();
  for (const line of raw.split("\n")) {
    const m = goModRequireLineRe.exec(line);
    if (m) paths.add(m[1]);
  }
  return [...paths].toSorted();
}

/** Subset of `go list -u -m all` lines for modules listed in go.mod (with `[newer]`). */
export function goFilterGoOutLinesModfileUpdates(gomodPath: string, fullList: string): string {
  const allowed = new Set(goModRequiredModulePathsFromFile(gomodPath));
  const out: string[] = [];
  for (const line of fullList.split("\n")) {
    const t = line.trim();
    if (!t || t.startsWith("go:")) continue;
    if (!t.includes("[")) continue;
    const path = t.split(/\s+/)[0];
    if (path && allowed.has(path)) out.push(t);
  }
  return out.join("\n");
}

export function goModUListHasTableRows(gomodPath: string, fullList: string): boolean {
  return goFilterGoOutLinesModfileUpdates(gomodPath, fullList).trim().length > 0;
}

export function captureGoListUModule(goRoot: string): string {
  return spawnText(["go", "list", "-u", "-m", "all"], { cwd: goRoot });
}

export function goEnvGomod(goRoot: string): string {
  const r = Bun.spawnSync(["go", "env", "GOMOD"], {
    cwd: goRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  const p = new TextDecoder().decode(r.stdout).trim();
  if (!p || !existsSync(p)) return "";
  return p;
}

export function goModHasUpgrades(goRoot: string): boolean {
  const gomod = goEnvGomod(goRoot);
  if (!gomod) return false;
  const required = goModRequiredModulePathsFromFile(gomod);
  if (required.length === 0) return false;
  const reqSet = new Set(required);
  const listOut = spawnText(
    ["go", "list", "-u", "-m", "-f", "{{if .Update}}{{println .Path}}{{end}}", "all"],
    { cwd: goRoot },
  );
  const updates = new Set(
    listOut
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean),
  );
  for (const p of reqSet) {
    if (updates.has(p)) return true;
  }
  return false;
}

export function goApplyModfileModuleUpdates(goRoot: string): void {
  const gomod = goEnvGomod(goRoot);
  if (!gomod) return;

  const listPkgs = spawnText(
    ["go", "list", "-u", "-m", "-f", "{{if .Update}}{{.Path}} {{.Update.Version}}{{end}}", "all"],
    { cwd: goRoot },
  );
  const reqSet = new Set(goModRequiredModulePathsFromFile(gomod));
  const pkgs: string[] = [];
  for (const line of listPkgs.split("\n")) {
    const t = line.trim();
    if (!t) continue;
    const [path, ver] = t.split(/\s+/);
    if (path && ver && reqSet.has(path)) pkgs.push(`${path}@${ver}`);
  }
  if (pkgs.length > 0) {
    goMust0(spawnExit(["go", "get", ...pkgs], { cwd: goRoot }), "go get (modfile modules)");
  }
  goMust0(spawnExit(["go", "get", "-u", "./..."], { cwd: goRoot }), "go get -u ./...");
  goMust0(
    spawnExit(["go", "get", "-u", "github.com/a-h/templ/cmd/templ"], { cwd: goRoot }),
    "go get -u templ",
  );
  goMust0(spawnExit(["go", "mod", "tidy"], { cwd: goRoot }), "go mod tidy");
}
