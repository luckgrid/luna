import { existsSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { die } from "./terminal";
import { spawnExit, spawnText, spawnTextAsync } from "./process";
import { semverCoreParts, trimOuterWhitespace } from "./utils";

// --------------------
// proto (pins in .prototools)
// --------------------

/**
 * Strip per-tool runtime hints that Bun (and other proto shims) inject so
 * `proto outdated` resolves pin sources from `.prototools` instead of treating
 * the running tool as "version-locked by env" (which silently drops `config_source`
 * and prevents `--update` from rewriting the pin).
 *
 * Keeps `PROTO_VERSION` (used by proto for self-reflection).
 */
export function protoScrubbedEnv(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env };
  for (const k of Object.keys(env)) {
    if (k === "PROTO_VERSION") continue;
    if (/^PROTO_[A-Z0-9]+_VERSION$/.test(k)) delete env[k];
  }
  return { ...env, ...extra };
}

/** One toolchain row from \`proto outdated --json\`. */
export type ProtoPinOutdatedEntry = {
  is_latest: boolean;
  is_outdated: boolean;
  config_source: string;
  config_version: string;
  current_version: string;
  newest_version: string;
  latest_version: string;
};

export type ProtoPinsOutdatedReport = Record<string, ProtoPinOutdatedEntry>;

/**
 * When `proto outdated --json` runs as a subprocess of Bun, the `bun` tool row often has
 * `config_source: null` (self-runtime / no separate pin file in JSON). Treat null like unknown.
 */
function isProtoPinOutdatedEntryJson(v: unknown): v is Omit<
  ProtoPinOutdatedEntry,
  "config_source"
> & {
  config_source: string | null;
} {
  if (typeof v !== "object" || v === null) return false;
  const bool = (k: string) => typeof Reflect.get(v, k) === "boolean";
  const str = (k: string) => typeof Reflect.get(v, k) === "string";
  const cs = Reflect.get(v, "config_source");
  const configSourceOk = typeof cs === "string" || cs === null || cs === undefined;
  return (
    bool("is_latest") &&
    bool("is_outdated") &&
    configSourceOk &&
    str("config_version") &&
    str("current_version") &&
    str("newest_version") &&
    str("latest_version")
  );
}

function isProtoPinsOutdatedReportJson(
  v: unknown,
): v is Record<
  string,
  Omit<ProtoPinOutdatedEntry, "config_source"> & { config_source: string | null }
> {
  if (typeof v !== "object" || v === null) return false;
  for (const val of Object.values(v)) {
    if (!isProtoPinOutdatedEntryJson(val)) return false;
  }
  return true;
}

export function protoPinsAnyOutdated(report: ProtoPinsOutdatedReport): boolean {
  return Object.values(report).some((x) => x.is_outdated);
}

/** Parse stdout of `proto outdated --json` (combined stdout+stderr also ok). */
export function parseProtoPinsOutdatedJson(text: string): ProtoPinsOutdatedReport {
  const trimmed = text.trim();
  if (!trimmed) die("proto outdated --json returned empty output (is proto in PATH?)");
  let data: unknown;
  try {
    data = JSON.parse(trimmed);
  } catch {
    die("proto outdated --json returned invalid JSON");
  }
  if (!isProtoPinsOutdatedReportJson(data)) die("proto outdated --json: unexpected shape");
  const out: ProtoPinsOutdatedReport = {};
  for (const [tool, row] of Object.entries(data)) {
    out[tool] = {
      ...row,
      config_source: typeof row.config_source === "string" ? row.config_source : "",
    };
  }
  return out;
}

/** `proto outdated --json`, parsed — run from repo root so .prototools resolves. */
export function captureProtoPinsOutdatedJson(repoRoot: string): ProtoPinsOutdatedReport {
  const r = Bun.spawnSync(["proto", "outdated", "--json"], {
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
    env: protoScrubbedEnv(),
  });
  const text = new TextDecoder().decode(r.stdout) + new TextDecoder().decode(r.stderr);
  return parseProtoPinsOutdatedJson(text);
}

/** Async `proto outdated --json` for parallel outdated gathering. */
export async function captureProtoPinsOutdatedJsonAsync(
  repoRoot: string,
): Promise<ProtoPinsOutdatedReport> {
  const text = await spawnTextAsync(["proto", "outdated", "--json"], {
    cwd: repoRoot,
    env: protoScrubbedEnv(),
  });
  return parseProtoPinsOutdatedJson(text);
}

export function protoOutdatedUpdateArgs(major: boolean): readonly string[] {
  return major
    ? ["proto", "outdated", "--update", "--latest", "-y"]
    : ["proto", "outdated", "--update", "-y"];
}

const PROTO_TOOL_PIN_RE = /^(bun|moon|proto|python|go)\s*=\s*"([^"]+)"\s*$/;

/** Run proto from `.proto/logs` so install failure logs stay out of the repo root; resolve repo `.prototools` via parent search. */
export function protoRunOpts(repoRoot: string): {
  cwd: string;
  env: NodeJS.ProcessEnv;
} {
  const logsDir = join(repoRoot, ".proto/logs");
  mkdirSync(logsDir, { recursive: true });
  return {
    cwd: logsDir,
    env: protoScrubbedEnv({ PROTO_CONFIG_MODE: "upwards" }),
  };
}

/** Pinned tools from the root (implicit) table of `.prototools` — ignores `[table]` sections. */
export function readProtoPinnedTools(repoRoot: string): { name: string; version: string }[] {
  const path = join(repoRoot, ".prototools");
  const raw = readFileSync(path, "utf8");
  const implicit = /^\[/m.test(raw) ? (raw.split(/^\[/m)[0] ?? raw) : raw;
  const seen = new Set<string>();
  const out: { name: string; version: string }[] = [];
  for (const line of implicit.split(/\r?\n/)) {
    const m = PROTO_TOOL_PIN_RE.exec(line.trim());
    if (!m) continue;
    const name = m[1];
    const version = m[2];
    if (seen.has(name)) continue;
    seen.add(name);
    out.push({ name, version });
  }
  return out;
}

/** One proto tool id from `.prototools`; runs under {@link protoRunOpts} (logs in `.proto/logs/`). */
export function protoInstallPinnedTool(repoRoot: string, tool: string): number {
  const opts = protoRunOpts(repoRoot);
  if (tool === "python") {
    let code = spawnExit(["proto", "install", "python", "-y"], opts);
    if (code !== 0) {
      console.error(
        "[luna] python: pre-built install failed; retrying with `proto install python --build` (slower, no standalone tarball yet)",
      );
      code = spawnExit(["proto", "install", "python", "--build", "-y"], opts);
    }
    return code;
  }
  return spawnExit(["proto", "install", tool, "-y"], opts);
}

/**
 * Install each pin in `.prototools` with proto, with python pre-built fallbacks Luna needs in practice.
 */
export function installAllProtoPinnedTools(repoRoot: string): number {
  const tools = readProtoPinnedTools(repoRoot);

  for (const { name } of tools) {
    const code = protoInstallPinnedTool(repoRoot, name);
    if (code !== 0) return code;
  }

  return 0;
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

export async function captureBunOutdatedRecursiveAsync(repoRoot: string): Promise<string> {
  return spawnTextAsync(["bun", "outdated", "--recursive"], { cwd: repoRoot });
}

export type PrereleaseBumpRow = { pkg: string; cwd: string };

/** Structured row from `bun outdated --recursive` (pipe-table format). */
export type BunOutdatedRow = {
  pkg: string;
  current: string;
  update: string;
  latest: string;
  workspace: string;
};

/** Strip ` (dev)` / ` (peer)` / ` (optional)` suffixes that bun adds to the Package column. */
function stripDepKindSuffix(pkg: string): string {
  return pkg.replace(/\s*\((?:dev|peer|optional)\)\s*$/i, "").trim();
}

function readPackageName(pkgJsonPath: string): string | null {
  try {
    const j: unknown = JSON.parse(readFileSync(pkgJsonPath, "utf8"));
    if (typeof j === "object" && j !== null && "name" in j) {
      const rec = j as Record<string, unknown>;
      if (typeof rec.name === "string") return rec.name;
    }
  } catch {
    /* ignore */
  }
  return null;
}

function resolveWorkspaceDir(repoRoot: string, workspaceLabel: string): string | null {
  // Root workspace (e.g. "luna") lives in the repo root, not under apps/* or packages/*.
  const rootPkg = join(repoRoot, "package.json");
  if (existsSync(rootPkg) && readPackageName(rootPkg) === workspaceLabel) return repoRoot;
  for (const top of ["apps", "packages"] as const) {
    const base = join(repoRoot, top);
    if (!existsSync(base)) continue;
    for (const ent of readdirSync(base, { withFileTypes: true })) {
      if (!ent.isDirectory()) continue;
      const pkgPath = join(base, ent.name, "package.json");
      if (!existsSync(pkgPath)) continue;
      if (readPackageName(pkgPath) === workspaceLabel) return join(base, ent.name);
    }
  }
  return null;
}

/** Parse pipe-table rows from `bun outdated --recursive` into structured form. */
export function parseBunOutdatedRows(stdin: string): BunOutdatedRow[] {
  const rows: BunOutdatedRow[] = [];
  for (const line of stdin.split("\n")) {
    const m = /^\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|/.exec(line);
    if (!m) continue;
    const pkgRaw = trimOuterWhitespace(m[1]);
    if (pkgRaw === "Package" || pkgRaw.startsWith("-")) continue;
    const current = trimOuterWhitespace(m[2]);
    if (!/^[\dv]/.test(current)) continue;
    rows.push({
      pkg: stripDepKindSuffix(pkgRaw),
      current,
      update: trimOuterWhitespace(m[3]),
      latest: trimOuterWhitespace(m[4]),
      workspace: trimOuterWhitespace(m[5]),
    });
  }
  return rows;
}

/**
 * "Real major" = leading non-zero digit changes. By convention `0.X → 0.Y` is
 * treated as a minor bump even though npm's `^` treats it as breaking, because
 * Luna's policy is "block only true major bumps without --major".
 */
export function isRealMajorBump(from: string, to: string): boolean {
  const [fMajor] = semverCoreParts(from);
  const [tMajor] = semverCoreParts(to);
  return fMajor !== tMajor && fMajor >= 1;
}

/** Rows where Current === Update but Latest differs — needs `bun add pkg@latest` per workspace. */
export function collectPrereleaseBumps(stdin: string, repoRoot: string): PrereleaseBumpRow[] {
  const rows: PrereleaseBumpRow[] = [];
  for (const row of parseBunOutdatedRows(stdin)) {
    if (row.current === row.update && row.latest !== row.current) {
      const cwd = resolveWorkspaceDir(repoRoot, row.workspace);
      if (cwd) rows.push({ pkg: row.pkg, cwd });
    }
  }
  return rows;
}

/**
 * Rows where the configured range can't reach the latest version, but the jump
 * is not a "real major" bump (leading non-zero digit unchanged). These need
 * `bun add pkg@latest` per workspace to widen the package.json range.
 */
export function collectInRangeMinorBumps(stdin: string, repoRoot: string): PrereleaseBumpRow[] {
  const rows: PrereleaseBumpRow[] = [];
  for (const row of parseBunOutdatedRows(stdin)) {
    if (row.current !== row.update || row.latest === row.current) continue;
    if (isRealMajorBump(row.current, row.latest)) continue;
    const cwd = resolveWorkspaceDir(repoRoot, row.workspace);
    if (cwd) rows.push({ pkg: row.pkg, cwd });
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

export async function captureUvLockDryRunAsync(uvProjectRoot: string): Promise<string> {
  return spawnTextAsync(["uv", "lock", "--upgrade", "--dry-run"], { cwd: uvProjectRoot });
}

// --------------------
// Go modules
// --------------------

/**
 * `go get -n -u all` — dry-run of what `go get -u all` would change (MVS-aligned).
 * This can be slow (often 10–30s+) on large graphs because the toolchain resolves the
 * upgrade like a real `go get`. Faster probes such as `go list -m -u all` only show
 * “newer versions exist on the proxy” and often disagree with this dry-run (many
 * `[v…]` lines can appear even when `go get -n -u all` prints no `go: upgraded` lines),
 * so we keep this command for an accurate match to `luna update`’s Go step.
 */
export function captureGoGetNDryRunUAll(moduleRoot: string): string {
  return spawnText(["go", "get", "-n", "-u", "all"], { cwd: moduleRoot });
}

export async function captureGoGetNDryRunUAllAsync(moduleRoot: string): Promise<string> {
  return spawnTextAsync(["go", "get", "-n", "-u", "all"], { cwd: moduleRoot });
}

export function goGetDryRunHasModuleChanges(out: string): boolean {
  return /(^go: upgraded |^go: downgraded |^go: added )/m.test(out);
}
