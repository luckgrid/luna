import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  eachWorkspaceChild,
  readPackageJsonName,
  semverCoreParts,
  spawnExit,
  spawnText,
  spawnTextAsync,
  trimOuterWhitespace,
  uniqSorted,
} from "./utils";

// --------------------
// bun
// --------------------

const BUN_OUTDATED_PIPE_RE = /^\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|/;

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

export type WorkspaceVersionBump = { pkg: string; cwd: string; version: string };

/** Structured row from `bun outdated --recursive` (pipe-table format). */
export type BunOutdatedRow = {
  pkg: string;
  current: string;
  /** Highest version allowed by the manifest range (`Update` column in bun's table). */
  newest: string;
  latest: string;
  workspace: string;
};

/** Strip ` (dev)` / ` (peer)` / ` (optional)` suffixes that bun adds to the Package column. */
function stripDepKindSuffix(pkg: string): string {
  return pkg.replace(/\s*\((?:dev|peer|optional)\)\s*$/i, "").trim();
}

/** Resolve workspace label to package directory or manifest path for table config column. */
export function resolveWorkspacePackageJson(repoRoot: string, workspaceLabel: string): string {
  const rootPkg = join(repoRoot, "package.json");
  if (existsSync(rootPkg) && readPackageJsonName(rootPkg) === workspaceLabel) return rootPkg;
  for (const { dir } of eachWorkspaceChild(repoRoot)) {
    const pkgPath = join(dir, "package.json");
    if (!existsSync(pkgPath)) continue;
    if (readPackageJsonName(pkgPath) === workspaceLabel) return pkgPath;
  }
  return join(repoRoot, `workspace:${workspaceLabel}`);
}

function resolveWorkspaceDir(repoRoot: string, workspaceLabel: string): string | null {
  const manifest = resolveWorkspacePackageJson(repoRoot, workspaceLabel);
  if (manifest.includes("workspace:")) return null;
  return resolve(join(manifest, ".."));
}

function matchBunOutdatedPipeLine(line: string): BunOutdatedRow | null {
  const m = BUN_OUTDATED_PIPE_RE.exec(line);
  if (!m) return null;
  const pkgRaw = trimOuterWhitespace(m[1]);
  if (pkgRaw === "Package" || pkgRaw.startsWith("-")) return null;
  const current = trimOuterWhitespace(m[2]);
  if (!/^[\dv]/.test(current)) return null;
  return {
    pkg: stripDepKindSuffix(pkgRaw),
    current,
    newest: trimOuterWhitespace(m[3]),
    latest: trimOuterWhitespace(m[4]),
    workspace: trimOuterWhitespace(m[5]),
  };
}

/** Parse pipe-table rows from `bun outdated --recursive` into structured form. */
export function parseBunOutdatedRows(stdin: string): BunOutdatedRow[] {
  const rows: BunOutdatedRow[] = [];
  for (const line of stdin.split("\n")) {
    const row = matchBunOutdatedPipeLine(line);
    if (row) rows.push(row);
  }
  return rows;
}

/** Parse `bun outdated --recursive` pipe table into outdated table rows. */
export function parseBunOutdatedTableRows(text: string, repoRoot: string): string[][] {
  return parseBunOutdatedRows(text).map((row) => [
    row.pkg,
    row.current,
    row.newest,
    row.latest,
    resolveWorkspacePackageJson(repoRoot, row.workspace),
  ]);
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

/**
 * True when `luna update` would change Bun workspace deps (respects `--major` policy).
 * Default mode targets the `Update` / Newest column, not registry Latest.
 */
export function bunWorkspaceHasActionableUpdates(out: string, major: boolean): boolean {
  if (!bunWorkspaceOutdatedFromOutput(out)) return false;
  if (major) return true;
  for (const row of parseBunOutdatedRows(out)) {
    if (row.current !== row.newest && !isRealMajorBump(row.current, row.newest)) return true;
    if (
      row.current === row.newest &&
      row.latest !== row.current &&
      !isRealMajorBump(row.current, row.latest)
    ) {
      return true;
    }
  }
  return false;
}

/** Rows where installed version is below the manifest-range newest (`Update` column). */
export function collectNewestBumps(stdin: string, repoRoot: string): WorkspaceVersionBump[] {
  const rows: WorkspaceVersionBump[] = [];
  for (const row of parseBunOutdatedRows(stdin)) {
    if (row.current === row.newest) continue;
    if (isRealMajorBump(row.current, row.newest)) continue;
    const cwd = resolveWorkspaceDir(repoRoot, row.workspace);
    if (cwd) rows.push({ pkg: row.pkg, cwd, version: row.newest });
  }
  return rows;
}

/** Rows where Current === Newest but Latest differs — `bun add pkg@latest` per workspace (`--major`). */
export function collectPrereleaseBumps(stdin: string, repoRoot: string): WorkspaceVersionBump[] {
  const rows: WorkspaceVersionBump[] = [];
  for (const row of parseBunOutdatedRows(stdin)) {
    if (row.current === row.newest && row.latest !== row.current) {
      const cwd = resolveWorkspaceDir(repoRoot, row.workspace);
      if (cwd) rows.push({ pkg: row.pkg, cwd, version: row.latest });
    }
  }
  return rows;
}

/**
 * Rows where the configured range can't reach registry Latest, but the jump is not a
 * "real major" bump (e.g. 0.x → 0.x+1). Widens package.json with `bun add pkg@latest`.
 */
export function collectInRangeMinorBumps(stdin: string, repoRoot: string): WorkspaceVersionBump[] {
  const rows: WorkspaceVersionBump[] = [];
  for (const row of parseBunOutdatedRows(stdin)) {
    if (row.current !== row.newest || row.latest === row.current) continue;
    if (isRealMajorBump(row.current, row.latest)) continue;
    const cwd = resolveWorkspaceDir(repoRoot, row.workspace);
    if (cwd) rows.push({ pkg: row.pkg, cwd, version: row.latest });
  }
  return rows;
}

/** When the root manifest overrides a dep, align the pin so workspace `bun add` can resolve. */
export function syncRootOverrideForPackage(repoRoot: string, pkg: string, version: string): void {
  const pkgPath = join(repoRoot, "package.json");
  if (!existsSync(pkgPath)) return;
  const parsed: unknown = JSON.parse(readFileSync(pkgPath, "utf8"));
  if (typeof parsed !== "object" || parsed === null) return;
  const overridesRaw = Reflect.get(parsed, "overrides");
  if (typeof overridesRaw !== "object" || overridesRaw === null || Array.isArray(overridesRaw)) {
    return;
  }
  if (!Reflect.has(overridesRaw, pkg)) return;
  const current = Reflect.get(overridesRaw, pkg);
  if (typeof current !== "string") return;
  if (current === version) return;
  Reflect.set(overridesRaw, pkg, version);
  writeFileSync(pkgPath, `${JSON.stringify(parsed, null, 2)}\n`, "utf8");
  console.log(`Synced root package.json overrides.${pkg} -> ${version}`);
}

export function runWorkspaceVersionBumps(repoRoot: string, rows: WorkspaceVersionBump[]): number {
  for (const { pkg, cwd, version } of rows) {
    syncRootOverrideForPackage(repoRoot, pkg, version);
    const code = spawnExit(["bun", "add", `${pkg}@${version}`, "--ignore-scripts"], { cwd });
    if (code !== 0) return code;
  }
  return 0;
}

/** Align root `package.json` `packageManager` with the `bun=` pin in `.prototools`. */
export function syncRootPackageManagerBun(repoRoot: string): void {
  const prototoolsPath = join(repoRoot, ".prototools");
  const raw = readFileSync(prototoolsPath, "utf8");
  const line = raw.split("\n").find((l) => /^\s*bun\s*=/.test(l));
  if (!line) {
    console.warn("warning: no bun= line in .prototools; skip packageManager sync");
    return;
  }
  const m = /^\s*bun\s*=\s*"([^"]+)"/.exec(line) ?? /^\s*bun\s*=\s*([^\s#]+)/.exec(line);
  const ver = m?.[1]?.trim();
  if (!ver) return;

  const pkgPath = join(repoRoot, "package.json");
  const pkg: Record<string, unknown> = JSON.parse(readFileSync(pkgPath, "utf8"));
  pkg.packageManager = `bun@${ver}`;
  writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
  console.log(`Synced root package.json packageManager -> bun@${ver}`);
}

/**
 * `apps/*` and `packages/*` dirs that contain `package.json`. Root `bun update --recursive` does not
 * rewrite semver ranges in these nested workspace manifests (Bun 1.3.x); `luna update` runs
 * per-workspace `bun add pkg@newest` so each package.json stays in sync with the lockfile.
 */
export function listBunWorkspacePackageDirs(repoRoot: string): string[] {
  const dirs: string[] = [];
  for (const { dir } of eachWorkspaceChild(repoRoot)) {
    if (existsSync(join(dir, "package.json"))) dirs.push(resolve(dir));
  }
  return uniqSorted(dirs);
}
