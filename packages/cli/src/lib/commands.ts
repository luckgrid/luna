/**
 * CLI infrastructure: process spawning, repo root, terminal UI, outdated cache/summary.
 * (Distinct from `src/commands/*` subcommand handlers.)
 */
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { readCliPackageVersion } from "../commands/version";
import {
  bunWorkspaceOutdatedFromOutput,
  listBunWorkspacePackageDirs,
  parseBunOutdatedTableRows,
} from "./bun";
import type { GoOutdatedProbe } from "./go";
import {
  goModuleOutdatedHasChanges,
  parseGoGetDryRunOutdatedRows,
  parseGoListModuleUpgradeRows,
} from "./go";
import { listGoModuleRoots, listUvProjectRoots } from "./moon";
import { parseUvDryRunTableRows } from "./py";
import type { ProtoPinsOutdatedReport } from "./proto";
import { protoPinsAnyOutdated } from "./proto";
import { uvLockHasUpgradesFromOutput } from "./py";
import {
  die,
  envFlagEnabled,
  formatProjectDirLabel,
  readOptionalIntEnvMin,
  shortenMiddle,
  shortenVersionCell,
  spawnExit,
  spawnText,
  spawnTextAsync,
  terminalAnsiStderr,
  terminalAnsiStdout,
  terminalHyperlinksSupported,
} from "./utils";

export { die, spawnExit, spawnText, spawnTextAsync };

// --------------------
// process (CLI wrappers)
// --------------------

export function requireCmd(name: string): void {
  const r = Bun.spawnSync(["/bin/sh", "-c", `command -v "${name.replace(/"/g, '\\"')}"`], {
    stdout: "ignore",
    stderr: "ignore",
  });
  if (r.exitCode !== 0) die(`missing required command: ${name}`);
}

export function runOrExit(code: number, step: string): void {
  if (code !== 0) {
    console.error(`error: ${step} (exit ${code})`);
    process.exit(code);
  }
}

// --------------------
// repo root
// --------------------

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

// --------------------
// terminal (ANSI + outdated tables)
// --------------------

function stderrColors(): { red: string; green: string; bold: string; dim: string; reset: string } {
  if (!terminalAnsiStderr()) {
    return { red: "", green: "", bold: "", dim: "", reset: "" };
  }
  return {
    red: "\x1b[0;31m",
    green: "\x1b[0;32m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    reset: "\x1b[0m",
  };
}

const C = stderrColors();

export function section(title: string): void {
  console.log(`\n${C.bold}== ${title} ==${C.reset}`);
}

export function strictOk(msg: string): void {
  console.error(`${C.green}✓${C.reset} ${msg}`);
}

export function strictNeed(msg: string): void {
  console.error(`${C.red}✗${C.reset} ${msg}`);
}

export function strictSummaryFailTitle(msg: string): void {
  console.error(`${C.bold}${C.red}${msg}${C.reset}`);
}

export function strictSummaryBullet(msg: string): void {
  console.error(`  ${C.red}•${C.reset} ${msg}`);
}

export function strictHint(msg: string): void {
  console.error(`${C.dim}${msg}${C.reset}`);
}

export function strictAllPassed(msg: string): void {
  console.log(`${C.green}${msg}${C.reset}`);
}

const HEADERS = ["Dependency", "Current", "Newest", "Latest", "Config"] as const;

export type OutdatedTableKind = "bun" | "uv" | "go";

function versionMaxForTable(): number {
  return readOptionalIntEnvMin("OUTDATED_VERSION_MAX", 30, 10);
}

function friendlyConfigLabel(absPath: string, repoRoot: string): string {
  if (absPath.startsWith("workspace:")) return absPath;
  try {
    const rel = relative(repoRoot, absPath).replace(/\\/g, "/");
    const base = basename(absPath);
    if (rel.startsWith("..") || rel === "") return base;
    if (base === "package.json" || base === "go.mod") {
      const parts = dirname(rel).split("/").filter(Boolean);
      const parent = parts[parts.length - 1];
      if (parent) return `${base} (${parent})`;
    }
    return base;
  } catch {
    return shortenMiddle(absPath.replace(/\\/g, "/"), 40);
  }
}

const OSC = "\x1b]";
const ST = "\x1b\\";

function wrapOsc8(href: string, visibleText: string): string {
  return `${OSC}8;;${href}${ST}${visibleText}${OSC}8;;${ST}`;
}

function configFileHref(absPath: string): string | null {
  if (!terminalHyperlinksSupported()) return null;
  const scheme = (process.env.OUTDATED_TERMINAL_LINK_SCHEME ?? "cursor").toLowerCase();
  if (scheme === "none" || scheme === "off") return null;
  if (!absPath || absPath === "—" || absPath.startsWith("workspace:")) return null;
  if (!existsSync(absPath)) return null;
  try {
    const fileHref = pathToFileURL(absPath).href;
    if (scheme === "file") return fileHref;
    if (scheme !== "vscode" && scheme !== "cursor") return null;
    const u = new URL(fileHref);
    return `${scheme}://file${u.pathname}`;
  } catch {
    return null;
  }
}

function formatConfigCell(plainLabel: string, colWidth: number, href: string | null): string {
  const t = plainLabel.length > colWidth ? shortenMiddle(plainLabel, colWidth) : plainLabel;
  if (!href) return t.padEnd(colWidth);
  const linked = wrapOsc8(href, t);
  const pad = colWidth - t.length;
  return pad > 0 ? `${linked}${" ".repeat(pad)}` : linked;
}

type TableModel = { display: string[][]; configHrefs: (string | null)[] };

function buildTableModel(
  rows: string[][],
  _table: OutdatedTableKind,
  repoRoot: string,
): TableModel {
  const vmax = versionMaxForTable();
  const configHrefs: (string | null)[] = [];
  const display = rows.map((r) => {
    const c = [...r];
    for (let i = 1; i <= 3; i++) {
      const v = c[i];
      if (v) c[i] = shortenVersionCell(v, vmax);
    }
    const absCfg = c[4];
    let href: string | null = null;
    if (absCfg) {
      href = configFileHref(absCfg);
      c[4] = friendlyConfigLabel(absCfg, repoRoot);
    }
    configHrefs.push(href);
    return c;
  });
  return { display, configHrefs };
}

function useRowStripes(): boolean {
  return terminalAnsiStdout();
}

const BG_ALT = "\x1b[48;5;236m";
const RESET = "\x1b[0m";

function padWidths(rows: string[][]): number[] {
  return HEADERS.map((h, i) => Math.max(h.length, ...rows.map((r) => (r[i] ?? "").length)));
}

function renderRow(widths: number[], cells: string[], configHref: string | null): string {
  const parts = cells.map((c, i) => {
    if (i === 4) return formatConfigCell(c, widths[i], configHref);
    return c.padEnd(widths[i]);
  });
  return `│${parts.join("  ")}│`;
}

function renderSep(widths: number[]): string {
  const inner = widths.reduce((a, b) => a + b, 0) + (widths.length - 1) * 2;
  return `│${"─".repeat(inner)}│`;
}

function renderBox(rows: string[][], table: OutdatedTableKind, repoRoot: string): void {
  if (rows.length === 0) {
    rows = [["(none)", "—", "—", "—", "—"]];
  }
  const { display, configHrefs } = buildTableModel(rows, table, repoRoot);
  const widths = padWidths(display);
  const stripe = useRowStripes();
  const headerLine = renderRow(widths, [...HEADERS], null);
  const sepLine = renderSep(widths);
  const dataLines = display.map((r, i) => renderRow(widths, r, configHrefs[i] ?? null));
  const innerWidth = headerLine.length - 2;
  const top = `╭${"─".repeat(innerWidth)}╮`;
  const bot = `╰${"─".repeat(innerWidth)}╯`;
  console.log(top);
  console.log(headerLine);
  console.log(sepLine);
  for (let i = 0; i < dataLines.length; i++) {
    const ln = dataLines[i];
    if (ln === undefined) continue;
    console.log(stripe && i % 2 === 1 ? `${BG_ALT}${ln}${RESET}` : ln);
  }
  console.log(bot);
}

export function printOutdatedTable(
  kind: OutdatedTableKind,
  stdin: string,
  ctx: {
    repoRoot: string;
    pyprojectPath?: string;
    goModPath?: string;
    goProbe?: GoOutdatedProbe;
  },
): void {
  const { repoRoot, pyprojectPath, goModPath, goProbe } = ctx;
  if (kind === "bun") {
    const rows = parseBunOutdatedTableRows(stdin, repoRoot);
    if (rows.length === 0) {
      renderBox(
        [["(no outdated table rows)", "—", "—", "—", join(repoRoot, "package.json")]],
        "bun",
        repoRoot,
      );
    } else {
      renderBox(rows, "bun", repoRoot);
    }
  } else if (kind === "uv") {
    const py = pyprojectPath ?? join(repoRoot, "pyproject.toml");
    const rows = parseUvDryRunTableRows(stdin, py);
    if (rows.length === 0) {
      renderBox(
        [["(no upgrades in uv lock --upgrade --dry-run)", "—", "—", "—", py]],
        "uv",
        repoRoot,
      );
    } else {
      renderBox(rows, "uv", repoRoot);
    }
  } else {
    const gm = goModPath ?? join(repoRoot, "go.mod");
    const rows =
      goProbe === "tool-list"
        ? parseGoListModuleUpgradeRows(stdin, gm)
        : parseGoGetDryRunOutdatedRows(stdin, gm);
    if (rows.length === 0) {
      const emptyMsg =
        goProbe === "tool-list"
          ? "(no tool upgrades from go list -m -u)"
          : "(no changes from go get -n -u all)";
      renderBox([[emptyMsg, "—", "—", "—", gm]], "go", repoRoot);
    } else {
      renderBox(rows, "go", repoRoot);
    }
  }
}

const PROTO_TABLE_HEADERS = ["Tool", "Current", "Newest", "Latest", "Config"] as const;

/** Box-drawn proto pin table from `proto outdated --json` (avoids a second `proto outdated` spawn). */
export function printProtoOutdatedTableFromReport(
  repoRoot: string,
  report: ProtoPinsOutdatedReport,
): void {
  const vmax = versionMaxForTable();
  const entries = Object.entries(report).toSorted(([a], [b]) => a.localeCompare(b));
  const rows: string[][] = entries.map(([tool, row]) => {
    const cfgAbs = row.config_source?.trim() ?? "";
    const cfgLabel = cfgAbs ? friendlyConfigLabel(cfgAbs, repoRoot) : "N/A";
    return [
      tool,
      shortenVersionCell(row.current_version, vmax),
      shortenVersionCell(row.newest_version, vmax),
      shortenVersionCell(row.latest_version, vmax),
      cfgLabel,
    ];
  });
  const configHrefs: (string | null)[] = entries.map(([, row]) => {
    const cfgAbs = row.config_source?.trim() ?? "";
    return cfgAbs ? configFileHref(cfgAbs) : null;
  });
  const display = rows;
  const widths = PROTO_TABLE_HEADERS.map((h, i) =>
    Math.max(h.length, ...display.map((r) => (r[i] ?? "").length)),
  );
  const stripe = useRowStripes();
  const headerLine = renderProtoRow(widths, [...PROTO_TABLE_HEADERS], null);
  const sepLine = renderProtoSep(widths);
  const dataLines = display.map((r, i) => renderProtoRow(widths, r, configHrefs[i] ?? null));
  const innerWidth = headerLine.length - 2;
  const top = `╭${"─".repeat(innerWidth)}╮`;
  const bot = `╰${"─".repeat(innerWidth)}╯`;
  console.log(top);
  console.log(headerLine);
  console.log(sepLine);
  for (let i = 0; i < dataLines.length; i++) {
    const ln = dataLines[i];
    if (ln === undefined) continue;
    console.log(stripe && i % 2 === 1 ? `${BG_ALT}${ln}${RESET}` : ln);
  }
  console.log(bot);
}

function renderProtoSep(widths: number[]): string {
  const inner = widths.reduce((a, b) => a + b, 0) + (widths.length - 1) * 2;
  return `│${"─".repeat(inner)}│`;
}

function renderProtoRow(widths: number[], cells: string[], configHref: string | null): string {
  const parts = cells.map((c, i) => {
    if (i === 4) return formatConfigCell(c, widths[i], configHref);
    return c.padEnd(widths[i]);
  });
  return `│${parts.join("  ")}│`;
}

// --------------------
// outdated cache
// --------------------

const CACHE_SCHEMA = 2;

/** On-disk shape (mirrors the gather snapshot in `commands/outdated.ts`). */
export type StoredOutdatedSnapshot = {
  protoReport: ProtoPinsOutdatedReport;
  bunOut: string;
  uvProjects: { root: string; dryRunOut: string }[];
  goModules: { root: string; goGetDryRunOut: string; probe?: GoOutdatedProbe }[];
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
// outdated summary + live TTY
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
  const goBad = snap.goModules.filter((g) => goModuleOutdatedHasChanges(g.goGetDryRunOut, g.probe));
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
  const goBad = snap.goModules.filter((g) => goModuleOutdatedHasChanges(g.goGetDryRunOut, g.probe));

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

export type OutdatedTierId = "proto" | "bun" | "uv" | "go";

const SPIN = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"] as const;

/** Live single-line status on stderr while probes run (TTY only). */
export function isOutdatedLiveStatusEnabled(): boolean {
  if (envFlagEnabled("LUNA_OUTDATED_NO_LIVE")) return false;
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

  printSummaryBlock(
    lines: { ok: boolean; text: string }[],
    printLine: (ok: boolean, text: string) => void,
  ): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    sectionErr("check results");
    for (const { ok, text } of lines) printLine(ok, text);
    process.stderr.write("\n");
  }

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
