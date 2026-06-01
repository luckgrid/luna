import { existsSync, readFileSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { eachWorkspaceChild, spawnSyncCaptured, uniqSorted } from "./utils";

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

  const { exitCode, stdout } = spawnSyncCaptured(["moon", "query", "projects"], { cwd: repoRoot });
  if (exitCode !== 0) {
    moonAllProjectsCache = null;
    return null;
  }
  const text = stdout.trim();
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
  for (const { dir } of eachWorkspaceChild(repoRoot)) {
    const moonPath = join(dir, "moon.yml");
    if (!existsSync(moonPath)) continue;
    const raw = readFileSync(moonPath, "utf8");
    if (!langRe.test(raw)) continue;
    if (language === "python" && existsSync(join(dir, "pyproject.toml"))) roots.push(resolve(dir));
    if (language === "go" && existsSync(join(dir, "go.mod"))) roots.push(resolve(dir));
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
