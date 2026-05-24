import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import {
  envFlagEnabled,
  nonEmptyLines,
  semverCoreParts,
  spawnSyncCaptured,
  spawnText,
  spawnTextAsync,
} from "./utils";

// --------------------
// Go modules
// --------------------

export type GoOutdatedProbe = "tool-list" | "get-dry-run";

/** `tool` directive paths from `go.mod` (e.g. `github.com/gohugoio/hugo`). */
export function readGoModToolPaths(moduleRoot: string): string[] {
  const path = join(moduleRoot, "go.mod");
  if (!existsSync(path)) return [];
  const raw = readFileSync(path, "utf8");
  const out: string[] = [];
  const seen = new Set<string>();
  for (const line of raw.split(/\r?\n/)) {
    const m = /^\s*tool\s+(\S+)/.exec(line);
    if (!m) continue;
    const p = m[1];
    if (seen.has(p)) continue;
    seen.add(p);
    out.push(p);
  }
  return out;
}

/** Last path segment of a Go module path (e.g. `github.com/gohugoio/hugo` → `hugo`). */
export function goToolBinaryName(toolModulePath: string): string {
  const t = toolModulePath.trim();
  const i = t.lastIndexOf("/");
  return i >= 0 ? t.slice(i + 1) : t;
}

/** True when the module root contains at least one local Go package (`go list ./...`). */
export function goModuleHasLocalPackages(moduleRoot: string): boolean {
  const { exitCode, stdout } = spawnSyncCaptured(["go", "list", "./..."], { cwd: moduleRoot });
  if (exitCode !== 0) return false;
  return nonEmptyLines(stdout).length > 0;
}

/**
 * Hugo-style modules: `tool` lines in go.mod and no local packages (e.g. `apps/web`).
 * Modules with both code and tools use the full-graph path.
 */
export function isGoModuleToolOnly(moduleRoot: string): boolean {
  if (readGoModToolPaths(moduleRoot).length === 0) return false;
  return !goModuleHasLocalPackages(moduleRoot);
}

export type VerifyGoModuleResult = { ok: true } | { ok: false; reason: string };

/**
 * Smoke-test after `go get` / `go mod tidy`: run `go build ./...` when the module lists packages,
 * then run each declared `go tool` (prefers `version`, falls back to `-h`).
 */
export function verifyGoModuleAfterDependencyUpdate(moduleRoot: string): VerifyGoModuleResult {
  const list = spawnSyncCaptured(["go", "list", "./..."], { cwd: moduleRoot });
  if (list.exitCode !== 0) {
    const msg = list.stderr.trim();
    return { ok: false, reason: msg || "go list ./... failed" };
  }
  const pkgLines = nonEmptyLines(list.stdout);
  if (pkgLines.length > 0) {
    const build = spawnSyncCaptured(["go", "build", "./..."], { cwd: moduleRoot });
    if (build.exitCode !== 0) {
      const msg = build.stderr.trim() || build.stdout.trim() || "go build ./... failed";
      return { ok: false, reason: msg };
    }
  }
  for (const path of readGoModToolPaths(moduleRoot)) {
    const name = goToolBinaryName(path);
    const ver = spawnSyncCaptured(["go", "tool", name, "version"], { cwd: moduleRoot });
    if (ver.exitCode === 0) continue;
    const help = spawnSyncCaptured(["go", "tool", name, "-h"], { cwd: moduleRoot });
    if (help.exitCode === 0) continue;
    const msg =
      ver.stderr.trim() || help.stderr.trim() || `go tool ${name} did not accept version or -h`;
    return { ok: false, reason: msg };
  }
  return { ok: true };
}

/** When set, tool-only modules use `go get -n -u all` instead of `go list -m -u` on tools. */
export function goFullGraphOutdatedEnabled(): boolean {
  return envFlagEnabled("LUNA_GO_FULL_GRAPH");
}

/**
 * `go get -n -u all` — dry-run of what `go get -u all` would change (MVS-aligned).
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

/** `go list -m -u` lines with `[v…]` mean a newer version exists on the proxy. */
export function goListModuleUpgradesHasChanges(out: string): boolean {
  return /^\S+\s+\S+\s+\[/m.test(out);
}

export function captureGoListToolUpgrades(moduleRoot: string, toolPaths: string[]): string {
  if (toolPaths.length === 0) return "";
  return spawnText(["go", "list", "-m", "-u", ...toolPaths], { cwd: moduleRoot });
}

export async function captureGoListToolUpgradesAsync(
  moduleRoot: string,
  toolPaths: string[],
): Promise<string> {
  if (toolPaths.length === 0) return "";
  return spawnTextAsync(["go", "list", "-m", "-u", ...toolPaths], { cwd: moduleRoot });
}

export function goModuleOutdatedHasChanges(
  out: string,
  probe: GoOutdatedProbe | undefined,
): boolean {
  if (probe === "tool-list") return goListModuleUpgradesHasChanges(out);
  return goGetDryRunHasModuleChanges(out);
}

function isPatchOnlyBump(from: string, to: string): boolean {
  const [fMaj, fMin] = semverCoreParts(from);
  const [tMaj, tMin, tPat] = semverCoreParts(to);
  const [, , fPat] = semverCoreParts(from);
  return fMaj === tMaj && fMin === tMin && tPat > fPat;
}

/**
 * True when `luna update` would change this module (respects `--major` and tool-only patch policy).
 */
export function goModuleHasActionableUpdates(
  out: string,
  probe: GoOutdatedProbe | undefined,
  major: boolean,
): boolean {
  if (!goModuleOutdatedHasChanges(out, probe)) return false;
  if (major || probe !== "tool-list") return true;
  for (const line of out.split("\n")) {
    const m = /^(\S+)\s+(\S+)\s+\[(\S+)\]/.exec(line.trim());
    if (!m) continue;
    const current = m[2];
    const newest = m[3];
    if (current && newest && isPatchOnlyBump(current, newest)) return true;
  }
  return false;
}

/** Parse `go list -m -u` lines with `[v…]` upgrade hints (tool-only modules). */
export function parseGoListModuleUpgradeRows(text: string, goModAbs: string): string[][] {
  const rows: string[][] = [];
  for (const line of text.split("\n")) {
    const t = line.trim();
    if (!t) continue;
    const m = /^(\S+)\s+(\S+)\s+\[(\S+)\]/.exec(t);
    if (!m) continue;
    const pkg = m[1];
    const current = m[2];
    const newest = m[3];
    if (pkg && current && newest) rows.push([pkg, current, newest, newest, goModAbs]);
  }
  return rows;
}

/** Parse `go get -n -u all` stdout/stderr for table rows (Dependency … Config). */
export function parseGoGetDryRunOutdatedRows(text: string, goModAbs: string): string[][] {
  const rows: string[][] = [];
  for (const m of text.matchAll(/^go: upgraded (.+?) (.+?) => (.+)$/gm)) {
    const p = m[1];
    const from = m[2];
    const to = m[3];
    if (p && from && to) rows.push([p, from, to, to, goModAbs]);
  }
  for (const m of text.matchAll(/^go: downgraded (.+?) (.+?) => (.+)$/gm)) {
    const p = m[1];
    const from = m[2];
    const to = m[3];
    if (p && from && to) rows.push([p, from, to, to, goModAbs]);
  }
  for (const m of text.matchAll(/^go: added (.+?) (.+)$/gm)) {
    const p = m[1];
    const ver = m[2];
    if (p && ver) rows.push([p, "—", ver, ver, goModAbs]);
  }
  return rows;
}

export async function captureGoModuleOutdatedAsync(
  moduleRoot: string,
): Promise<{ root: string; goGetDryRunOut: string; probe: GoOutdatedProbe }> {
  const useToolList = isGoModuleToolOnly(moduleRoot) && !goFullGraphOutdatedEnabled();
  if (useToolList) {
    const tools = readGoModToolPaths(moduleRoot);
    const goGetDryRunOut = await captureGoListToolUpgradesAsync(moduleRoot, tools);
    return { root: moduleRoot, goGetDryRunOut, probe: "tool-list" };
  }
  const goGetDryRunOut = await captureGoGetNDryRunUAllAsync(moduleRoot);
  return { root: moduleRoot, goGetDryRunOut, probe: "get-dry-run" };
}
