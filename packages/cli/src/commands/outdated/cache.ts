import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { readCliPackageVersion } from "../version";
import { listBunWorkspacePackageDirs } from "../../lib/bun";
import { listGoModuleRoots, listUvProjectRoots } from "../../lib/moon";
import type { ProtoPinsOutdatedReport } from "../../lib/proto";
import type { StoredOutdatedSnapshot } from "./types";

const CACHE_SCHEMA = 2;

type DiskPayload = {
  lunaSchema: number;
  cliVersion: string;
  writtenAt: string;
  fingerprint: string;
  snap: StoredOutdatedSnapshot;
};

function cacheJsonPath(repoRoot: string): string {
  return join(repoRoot, ".cache", "outdated-snapshot.json");
}

/** Paths that should invalidate a cached outdated snapshot when they change. */
export function collectOutdatedFingerprintPaths(repoRoot: string): string[] {
  const paths: string[] = [
    join(repoRoot, ".prototools"),
    join(repoRoot, "package.json"),
    join(repoRoot, "bun.lock"),
  ];
  for (const dir of listBunWorkspacePackageDirs(repoRoot)) {
    paths.push(join(dir, "package.json"));
  }
  for (const root of listUvProjectRoots(repoRoot)) {
    paths.push(join(root, "pyproject.toml"));
    const lock = join(root, "uv.lock");
    if (existsSync(lock)) paths.push(lock);
  }
  for (const root of listGoModuleRoots(repoRoot)) {
    paths.push(join(root, "go.mod"));
    const sum = join(root, "go.sum");
    if (existsSync(sum)) paths.push(sum);
  }
  return paths;
}

export function computeOutdatedFingerprint(repoRoot: string): string {
  const parts: string[] = [];
  for (const p of collectOutdatedFingerprintPaths(repoRoot)) {
    if (!existsSync(p)) {
      parts.push(`${p}:missing`);
      continue;
    }
    const st = statSync(p);
    parts.push(`${p}:${st.mtimeMs}:${st.size}`);
  }
  return createHash("sha256").update(parts.join("|"), "utf8").digest("hex");
}

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function isProtoReport(v: unknown): v is ProtoPinsOutdatedReport {
  if (!isPlainObject(v)) return false;
  for (const row of Object.values(v)) {
    if (!isPlainObject(row)) return false;
    if (typeof Reflect.get(row, "is_outdated") !== "boolean") return false;
    if (typeof Reflect.get(row, "current_version") !== "string") return false;
  }
  return true;
}

function isStoredSnapshot(v: unknown): v is StoredOutdatedSnapshot {
  if (!isPlainObject(v)) return false;
  if (!isProtoReport(v.protoReport)) return false;
  if (typeof v.bunOut !== "string") return false;
  if (!Array.isArray(v.uvProjects) || !Array.isArray(v.goModules)) return false;
  for (const u of v.uvProjects) {
    if (!isPlainObject(u)) return false;
    if (typeof u.root !== "string" || typeof u.dryRunOut !== "string") return false;
  }
  for (const g of v.goModules) {
    if (!isPlainObject(g)) return false;
    if (typeof g.root !== "string" || typeof g.goGetDryRunOut !== "string") return false;
    const probe = Reflect.get(g, "probe");
    if (probe !== undefined && probe !== "tool-list" && probe !== "get-dry-run") return false;
  }
  return true;
}

export function writeOutdatedCache(repoRoot: string, snap: StoredOutdatedSnapshot): void {
  const dir = join(repoRoot, ".cache");
  mkdirSync(dir, { recursive: true });
  const payload: DiskPayload = {
    lunaSchema: CACHE_SCHEMA,
    cliVersion: readCliPackageVersion(),
    writtenAt: new Date().toISOString(),
    fingerprint: computeOutdatedFingerprint(repoRoot),
    snap,
  };
  const finalPath = cacheJsonPath(repoRoot);
  const tmpPath = `${finalPath}.tmp`;
  writeFileSync(tmpPath, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
  renameSync(tmpPath, finalPath);
}

/** Max age before `luna update` refreshes a fingerprint-valid snapshot automatically. */
export const OUTDATED_SNAPSHOT_STALE_MS = 12 * 60 * 60 * 1000;

export type OutdatedCacheEntry = {
  snap: StoredOutdatedSnapshot;
  writtenAt: string;
};

export function isOutdatedSnapshotStale(
  writtenAt: string,
  maxAgeMs = OUTDATED_SNAPSHOT_STALE_MS,
): boolean {
  const t = Date.parse(writtenAt);
  if (Number.isNaN(t)) return true;
  return Date.now() - t >= maxAgeMs;
}
/** Returns cached snapshot only when file exists, schema matches, and fingerprint still matches disk. */
export function tryReadOutdatedCacheEntry(repoRoot: string): OutdatedCacheEntry | null {
  const finalPath = cacheJsonPath(repoRoot);
  if (!existsSync(finalPath)) return null;
  let raw: unknown;
  try {
    raw = JSON.parse(readFileSync(finalPath, "utf8"));
  } catch {
    return null;
  }
  if (!isPlainObject(raw)) return null;
  const o = raw;
  if (o.lunaSchema !== CACHE_SCHEMA) return null;
  if (typeof o.fingerprint !== "string") return null;
  if (typeof o.writtenAt !== "string") return null;
  if (o.fingerprint !== computeOutdatedFingerprint(repoRoot)) return null;
  if (!isStoredSnapshot(o.snap)) return null;
  return { snap: o.snap, writtenAt: o.writtenAt };
}

export function tryReadOutdatedCache(repoRoot: string): StoredOutdatedSnapshot | null {
  return tryReadOutdatedCacheEntry(repoRoot)?.snap ?? null;
}
