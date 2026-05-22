/**
 * `luna outdated` support: disk snapshot cache (`.cache/outdated-snapshot.json`),
 * TTY live gather status, and pass/fail summary helpers.
 */
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { readCliPackageVersion } from "../commands/version";
import {
  formatProjectDirLabel,
  listBunWorkspacePackageDirs,
  listGoModuleRoots,
  listUvProjectRoots,
} from "./repo";
import {
  bunWorkspaceOutdatedFromOutput,
  goGetDryRunHasModuleChanges,
  protoPinsAnyOutdated,
  type ProtoPinsOutdatedReport,
  uvLockHasUpgradesFromOutput,
} from "./toolchains";
import { terminalAnsiStderr } from "./utils";

// --------------------
// Snapshot cache
// --------------------

const CACHE_SCHEMA = 1;

/** On-disk shape (mirrors the gather snapshot in `commands/outdated.ts`). */
export type StoredOutdatedSnapshot = {
  protoReport: ProtoPinsOutdatedReport;
  bunOut: string;
  uvProjects: { root: string; dryRunOut: string }[];
  goModules: { root: string; goGetDryRunOut: string }[];
};

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

/** Returns cached snapshot only when file exists, schema matches, and fingerprint still matches disk. */
export function tryReadOutdatedCache(repoRoot: string): StoredOutdatedSnapshot | null {
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
  if (o.fingerprint !== computeOutdatedFingerprint(repoRoot)) return null;
  if (!isStoredSnapshot(o.snap)) return null;
  return o.snap;
}

// --------------------
// Summary (pass/fail + tier lines)
// --------------------

export type OutdatedSummaryState = {
  failed: number;
  stProto: number;
  stBun: number;
  stUv: number;
  stGo: number;
};

export function computeOutdatedSummaryState(snap: StoredOutdatedSnapshot): OutdatedSummaryState {
  let stProto = 0;
  let stBun = 0;
  let stUv = 0;
  let stGo = 0;

  if (protoPinsAnyOutdated(snap.protoReport)) stProto = 1;
  if (bunWorkspaceOutdatedFromOutput(snap.bunOut)) stBun = 1;
  const uvBad = snap.uvProjects.filter((p) => uvLockHasUpgradesFromOutput(p.dryRunOut));
  if (uvBad.length > 0) stUv = 1;
  const goBad = snap.goModules.filter((g) => goGetDryRunHasModuleChanges(g.goGetDryRunOut));
  if (goBad.length > 0) stGo = 1;

  const failed = stProto | stBun | stUv | stGo;
  return { failed, stProto, stBun, stUv, stGo };
}

/** One line per tier (same wording as `printOutdatedCheckSummary` full mode). */
export function outdatedTierMessages(
  repoRoot: string,
  snap: StoredOutdatedSnapshot,
): { ok: boolean; text: string }[] {
  const { stProto, stBun, stUv, stGo } = computeOutdatedSummaryState(snap);
  const uvBad = snap.uvProjects.filter((p) => uvLockHasUpgradesFromOutput(p.dryRunOut));
  const goBad = snap.goModules.filter((g) => goGetDryRunHasModuleChanges(g.goGetDryRunOut));

  const lines: { ok: boolean; text: string }[] = [];
  lines.push(
    stProto
      ? { ok: false, text: "proto — outdated pin(s) in .prototools" }
      : { ok: true, text: "proto — OK (.prototools)" },
  );
  lines.push(
    stBun
      ? { ok: false, text: "Bun — outdated direct dependencies in workspaces" }
      : { ok: true, text: "Bun — OK (workspaces)" },
  );
  if (stUv) {
    lines.push({
      ok: false,
      text: `Python / uv — lockfile(s) can update: ${uvBad.map((p) => formatProjectDirLabel(repoRoot, p.root)).join(", ")} (see luna update)`,
    });
  } else if (snap.uvProjects.length === 0) {
    lines.push({ ok: true, text: "Python / uv — OK (no uv projects discovered)" });
  } else {
    lines.push({ ok: true, text: `Python / uv — OK (${snap.uvProjects.length} project(s))` });
  }
  if (stGo) {
    lines.push({
      ok: false,
      text: `Go — go.mod can advance: ${goBad.map((g) => formatProjectDirLabel(repoRoot, g.root)).join(", ")} (see luna update)`,
    });
  } else if (snap.goModules.length === 0) {
    lines.push({ ok: true, text: "Go — OK (no go.mod projects discovered)" });
  } else {
    lines.push({ ok: true, text: `Go — OK (${snap.goModules.length} module(s))` });
  }
  return lines;
}

// --------------------
// Live TTY status while gathering
// --------------------

export type OutdatedTierId = "proto" | "bun" | "uv" | "go";

const SPIN = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"] as const;

/** Live single-line status on stderr while probes run (TTY only). */
export function isOutdatedLiveStatusEnabled(): boolean {
  if (process.env.LUNA_OUTDATED_NO_LIVE === "1") return false;
  if (process.env.CI === "true" || process.env.CI === "1") return false;
  return process.stderr.isTTY && process.stderr.writable;
}

type TierCell = "pending" | "running" | { ok: boolean; ms: number };

const TIER_ORDER: OutdatedTierId[] = ["proto", "bun", "uv", "go"];

function sectionErr(title: string): void {
  const bold = terminalAnsiStderr() ? "\x1b[1m" : "";
  const reset = terminalAnsiStderr() ? "\x1b[0m" : "";
  process.stderr.write(`\n${bold}== ${title} ==${reset}\n`);
}

export class OutdatedLiveStatus {
  private readonly cells: Record<OutdatedTierId, TierCell> = {
    proto: "pending",
    bun: "pending",
    uv: "pending",
    go: "pending",
  };
  private spinIdx = 0;
  private timer: ReturnType<typeof setInterval> | null = null;

  begin(): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    process.stderr.write("\n[luna] scanning toolchains (parallel)…\n");
    for (const t of TIER_ORDER) this.cells[t] = "running";
    this.timer = setInterval(() => this.draw(), 110);
    this.draw();
  }

  markDone(tier: OutdatedTierId, ok: boolean, ms: number): void {
    this.cells[tier] = { ok, ms };
    if (isOutdatedLiveStatusEnabled()) this.draw();
  }

  private draw(): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    const spin = SPIN[this.spinIdx % SPIN.length] ?? "⠋";
    this.spinIdx += 1;
    const parts = TIER_ORDER.map((id) => {
      const c = this.cells[id];
      if (c === "pending") return `○ ${id}`;
      if (c === "running") return `${spin} ${id}`;
      const mark = c.ok ? "✓" : "✗";
      return `${mark} ${id} (${c.ms}ms)`;
    });
    const line = `[luna] outdated  ${parts.join("  │  ")}`;
    const w = Math.max(20, process.stderr.columns ?? 80);
    const padded = line.length >= w ? `${line.slice(0, w - 4)} …` : line.padEnd(w, " ");
    process.stderr.write(`\r\x1b[2K${padded}`);
  }

  /** Final ✓/✗ lines on stderr (after spinner cleared). Safe without {@link begin}. */
  printSummaryBlock(
    lines: { ok: boolean; text: string }[],
    printLine: (ok: boolean, text: string) => void,
  ): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    sectionErr("check results");
    for (const { ok, text } of lines) printLine(ok, text);
    process.stderr.write("\n");
  }

  /** Stop spinner, then {@link printSummaryBlock}. */
  finishAfterGather(
    lines: { ok: boolean; text: string }[],
    printLine: (ok: boolean, text: string) => void,
  ): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    process.stderr.write("\r\x1b[2K");
    this.printSummaryBlock(lines, printLine);
  }
}
