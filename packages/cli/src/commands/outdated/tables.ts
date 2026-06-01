import { existsSync } from "node:fs";
import { basename, dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { parseBunOutdatedTableRows } from "../../lib/bun";
import type { GoOutdatedProbe } from "../../lib/go";
import { parseGoGetDryRunOutdatedRows, parseGoListModuleUpgradeRows } from "../../lib/go";
import { parseUvDryRunTableRows } from "../../lib/py";
import type { ProtoPinsOutdatedReport } from "../../lib/proto";
import {
  readOptionalIntEnvMin,
  shortenMiddle,
  shortenVersionCell,
  terminalAnsiStdout,
  terminalHyperlinksSupported,
} from "../../lib/utils";

export type OutdatedTableKind = "bun" | "uv" | "go";

const HEADERS = ["Dependency", "Current", "Newest", "Latest", "Config"] as const;
const PROTO_TABLE_HEADERS = ["Tool", "Current", "Newest", "Latest", "Config"] as const;

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
