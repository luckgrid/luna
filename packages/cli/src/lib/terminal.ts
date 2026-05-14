/** Terminal output: ANSI styling, status lines, and box-drawn outdated dependency tables. */
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import type { ProtoPinsOutdatedReport } from "./toolchains";
import {
  readOptionalIntEnvMin,
  shortenMiddle,
  shortenVersionCell,
  terminalAnsiStderr,
  terminalAnsiStdout,
  terminalHyperlinksSupported,
  trimOuterWhitespace,
} from "./utils";

// --------------------
// ANSI (stderr-centric for status/errors)
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

export function die(msg: string): never {
  console.error(`error: ${msg}`);
  process.exit(1);
}

// --------------------
// Outdated dependency tables (bun / uv)
// --------------------

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

function readPackageName(pkgPath: string): string | undefined {
  try {
    const raw = readFileSync(pkgPath, "utf8");
    const j: unknown = JSON.parse(raw);
    if (typeof j === "object" && j !== null && "name" in j) {
      const rec: Record<string, unknown> = j;
      if (typeof rec.name === "string") return rec.name;
    }
  } catch {
    /* ignore */
  }
  return undefined;
}

function resolveWorkspaceManifest(repo: string, workspaceLabel: string): string {
  for (const top of ["apps", "packages"] as const) {
    const base = join(repo, top);
    if (!existsSync(base)) continue;
    for (const ent of readdirSync(base, { withFileTypes: true })) {
      if (!ent.isDirectory()) continue;
      const pkgPath = join(base, ent.name, "package.json");
      if (!existsSync(pkgPath)) continue;
      const name = readPackageName(pkgPath);
      if (name === workspaceLabel) return pkgPath;
    }
  }
  return join(repo, `workspace:${workspaceLabel}`);
}

function parseBunOutdated(text: string, repo: string): string[][] {
  const rows: string[][] = [];
  for (const line of text.split("\n")) {
    const m = /^\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|/.exec(line);
    if (!m) continue;
    const pkg = trimOuterWhitespace(m[1]);
    if (pkg === "Package" || pkg.startsWith("-")) continue;
    const current = trimOuterWhitespace(m[2]);
    const newest = trimOuterWhitespace(m[3]);
    const latest = trimOuterWhitespace(m[4]);
    const ws = trimOuterWhitespace(m[5]);
    if (!/^[\dv]/.test(current)) continue;
    rows.push([pkg, current, newest, latest, resolveWorkspaceManifest(repo, ws)]);
  }
  return rows;
}

const uvUpdateRe = /^Update ([^ ]+) v(.+) -> v(.+)$/;

/** Parse `go get -n -u all` stdout/stderr for table rows (Dependency … Config). */
function parseGoGetDryRunOutdatedRows(text: string, goModAbs: string): string[][] {
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

function parseUvDryRun(text: string, pyproject: string): string[][] {
  const rows: string[][] = [];
  for (const line of text.split("\n")) {
    const m = uvUpdateRe.exec(line.trim());
    if (!m) continue;
    const name = m[1];
    const current = `v${m[2]}`;
    const newest = `v${m[3]}`;
    rows.push([name, current, newest, newest, pyproject]);
  }
  return rows;
}

export function printOutdatedTable(
  kind: OutdatedTableKind,
  stdin: string,
  ctx: { repoRoot: string; pyprojectPath?: string; goModPath?: string },
): void {
  const { repoRoot, pyprojectPath, goModPath } = ctx;
  if (kind === "bun") {
    const rows = parseBunOutdated(stdin, repoRoot);
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
    const rows = parseUvDryRun(stdin, py);
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
    const rows = parseGoGetDryRunOutdatedRows(stdin, gm);
    if (rows.length === 0) {
      renderBox([["(no changes from go get -n -u all)", "—", "—", "—", gm]], "go", repoRoot);
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
