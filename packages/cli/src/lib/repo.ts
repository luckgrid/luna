import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";

/** Walk parents from `start` until `.prototools` + `package.json` exist (monorepo root). */
export function findRepoRoot(start = process.cwd()): string {
  let dir = resolve(start);
  for (;;) {
    if (existsSync(join(dir, ".prototools")) && existsSync(join(dir, "package.json"))) {
      return dir;
    }
    const parent = resolve(dir, "..");
    if (parent === dir) {
      throw new Error(
        "Could not find monorepo root (no .prototools + package.json in any parent of cwd). Run from inside the repo.",
      );
    }
    dir = parent;
  }
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

function uniqSorted(paths: string[]): string[] {
  return [...new Set(paths.map((p) => resolve(p)))].toSorted();
}

type MoonProjectRow = { root: string; language: string };

/** One `moon query projects` (all languages) per process; avoids duplicate moon spawns. */
let moonAllProjectsCache: MoonProjectRow[] | null | undefined;

/** For tests or long-lived runners that change cwd/repo between calls. */
export function resetMoonProjectsCache(): void {
  moonAllProjectsCache = undefined;
}

/**
 * All Moon projects with `config.language` — single `moon query projects` per repo per process.
 * Returns `null` if moon is missing, fails, or JSON is unusable.
 */
function queryMoonProjectsAll(repoRoot: string): MoonProjectRow[] | null {
  if (moonAllProjectsCache !== undefined) return moonAllProjectsCache;

  const r = Bun.spawnSync(["moon", "query", "projects"], {
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (r.exitCode !== 0) {
    moonAllProjectsCache = null;
    return null;
  }
  const text = new TextDecoder().decode(r.stdout).trim();
  if (!text.startsWith("{")) {
    moonAllProjectsCache = null;
    return null;
  }
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    moonAllProjectsCache = null;
    return null;
  }
  if (typeof data !== "object" || data === null || !("projects" in data)) {
    moonAllProjectsCache = null;
    return null;
  }
  const projects = Reflect.get(data, "projects");
  if (!Array.isArray(projects)) {
    moonAllProjectsCache = null;
    return null;
  }
  const rows: MoonProjectRow[] = [];
  for (const item of projects) {
    if (typeof item !== "object" || item === null) continue;
    const rootRaw = Reflect.get(item, "root");
    const root = typeof rootRaw === "string" ? rootRaw.trim() : "";
    if (!root) continue;
    const config = Reflect.get(item, "config");
    let language = "";
    if (typeof config === "object" && config !== null) {
      const lang = Reflect.get(config, "language");
      if (typeof lang === "string") language = lang.trim().toLowerCase();
    }
    rows.push({ root: resolve(root), language });
  }
  moonAllProjectsCache = rows;
  return moonAllProjectsCache;
}

/** Same globs as `.moon/workspace.yml` — walk `moon.yml` + `language:` when moon query is unavailable. */
function scanWorkspaceForLanguage(repoRoot: string, language: "python" | "go"): string[] {
  const langToken = language === "python" ? "python" : "go";
  const langRe = new RegExp(`^\\s*language:\\s*${langToken}\\s*$`, "m");
  const roots: string[] = [];
  for (const top of ["apps", "packages"] as const) {
    const base = join(repoRoot, top);
    if (!existsSync(base)) continue;
    for (const ent of readdirSync(base, { withFileTypes: true })) {
      if (!ent.isDirectory()) continue;
      const dir = join(base, ent.name);
      const moonPath = join(dir, "moon.yml");
      if (!existsSync(moonPath)) continue;
      const raw = readFileSync(moonPath, "utf8");
      if (!langRe.test(raw)) continue;
      if (language === "python" && existsSync(join(dir, "pyproject.toml")))
        roots.push(resolve(dir));
      if (language === "go" && existsSync(join(dir, "go.mod"))) roots.push(resolve(dir));
    }
  }
  return uniqSorted(roots);
}

function optionalExtraRoot(repoRoot: string, envName: string): string | null {
  const raw = process.env[envName]?.trim();
  if (!raw) return null;
  return isAbsolute(raw) ? raw : resolve(repoRoot, raw);
}

/**
 * Prefer a single `moon query projects` when it returns at least one project; otherwise scan
 * `apps/*` + `packages/*` for `moon.yml` + `language:` (covers missing moon, JSON changes, or empty query).
 */
function resolveProjectRoots(repoRoot: string, language: "python" | "go"): string[] {
  const scanned = scanWorkspaceForLanguage(repoRoot, language);
  const all = queryMoonProjectsAll(repoRoot);
  if (all !== null && all.length > 0) {
    const token = language === "python" ? "python" : "go";
    const fromMoon = all
      .filter((e) => {
        if (e.language !== token) return false;
        if (language === "python") return existsSync(join(e.root, "pyproject.toml"));
        return existsSync(join(e.root, "go.mod"));
      })
      .map((e) => e.root);
    if (fromMoon.length > 0) return uniqSorted(fromMoon);
  }
  return scanned;
}

/** All uv/pyproject roots, plus optional `UV_PROJECT_ROOT` (extra path outside the scan). */
export function listUvProjectRoots(repoRoot: string): string[] {
  let roots = [...resolveProjectRoots(repoRoot, "python")];
  const extra = optionalExtraRoot(repoRoot, "UV_PROJECT_ROOT");
  if (extra && existsSync(join(extra, "pyproject.toml"))) roots.push(resolve(extra));
  return uniqSorted(roots);
}

/** All go.mod roots, plus optional `GO_MODULE_ROOT` (extra path outside the scan). */
export function listGoModuleRoots(repoRoot: string): string[] {
  let roots = [...resolveProjectRoots(repoRoot, "go")];
  const extra = optionalExtraRoot(repoRoot, "GO_MODULE_ROOT");
  if (extra && existsSync(join(extra, "go.mod"))) roots.push(resolve(extra));
  return uniqSorted(roots);
}

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

/**
 * `apps/*` and `packages/*` dirs that contain `package.json`. Root `bun update --recursive` does not
 * rewrite semver ranges in these nested workspace manifests (Bun 1.3.x); `luna update` runs
 * `bun update --latest` per directory so each package.json stays in sync with the lockfile.
 */
export function listBunWorkspacePackageDirs(repoRoot: string): string[] {
  const dirs: string[] = [];
  for (const top of ["apps", "packages"] as const) {
    const base = join(repoRoot, top);
    if (!existsSync(base)) continue;
    for (const ent of readdirSync(base, { withFileTypes: true })) {
      if (!ent.isDirectory()) continue;
      const dir = join(base, ent.name);
      if (existsSync(join(dir, "package.json"))) dirs.push(resolve(dir));
    }
  }
  return uniqSorted(dirs);
}

export function formatProjectDirLabel(repoRoot: string, dir: string): string {
  try {
    const r = relative(repoRoot, resolve(dir)).replace(/\\/g, "/");
    return r && !r.startsWith("..") ? r : resolve(dir);
  } catch {
    return dir;
  }
}
