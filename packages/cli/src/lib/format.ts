/** Box-drawn tables for outdated dependency reports. */
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";

const HEADERS = ["Dependency", "Current", "Newest", "Latest", "Config"] as const;

export type OutdatedTableKind = "bun" | "uv" | "go";

function versionMaxForTable(table: OutdatedTableKind): number {
  const raw = process.env.OUTDATED_VERSION_MAX;
  if (raw !== undefined && raw !== "") {
    const n = Number.parseInt(raw, 10);
    if (Number.isFinite(n) && n >= 10) return n;
  }
  return table === "go" ? 18 : 30;
}

function shortenMiddle(s: string, maxLen: number): string {
  if (s.length <= maxLen) return s;
  const el = "…";
  const inner = maxLen - el.length;
  const left = Math.ceil(inner * 0.55);
  const right = inner - left;
  return s.slice(0, left) + el + s.slice(-right);
}

function shortenVersionCell(s: string, max: number): string {
  if (s.length <= max) return s;
  if (!/^v[\d._-]|^[\d]/.test(s)) return s;
  return shortenMiddle(s, max);
}

function friendlyConfigLabel(absPath: string, repoRoot: string): string {
  if (absPath.startsWith("workspace:")) return absPath;
  try {
    const rel = relative(repoRoot, absPath).replace(/\\/g, "/");
    const base = basename(absPath);
    if (rel.startsWith("..") || rel === "") return base;
    if (base === "package.json") {
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

function useTerminalFileLinks(): boolean {
  return (
    !process.env.NO_COLOR &&
    !process.env.OUTDATED_NO_TERMINAL_LINKS &&
    process.stdout.isTTY &&
    process.env.TERM !== "dumb"
  );
}

function configFileHref(absPath: string): string | null {
  if (!useTerminalFileLinks()) return null;
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

function buildTableModel(rows: string[][], table: OutdatedTableKind, repoRoot: string): TableModel {
  const vmax = versionMaxForTable(table);
  const goDepMax = 34;
  const configHrefs: (string | null)[] = [];
  const display = rows.map((r) => {
    const c = [...r];
    const dep = c[0] ?? "";
    if (table === "go" && dep.length > goDepMax && !dep.startsWith("(")) {
      c[0] = shortenMiddle(dep, goDepMax);
    }
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
  return !process.env.NO_COLOR && process.stdout.isTTY && process.env.TERM !== "dumb";
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

function trimCell(s: string): string {
  return s.replace(/\s+$/g, "").replace(/^\s+/g, "");
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
    const pkg = trimCell(m[1]);
    if (pkg === "Package" || pkg.startsWith("-")) continue;
    const current = trimCell(m[2]);
    const newest = trimCell(m[3]);
    const latest = trimCell(m[4]);
    const ws = trimCell(m[5]);
    if (!/^[\dv]/.test(current)) continue;
    rows.push([pkg, current, newest, latest, resolveWorkspaceManifest(repo, ws)]);
  }
  return rows;
}

const uvUpdateRe = /^Update ([^ ]+) v(.+) -> v(.+)$/;

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

const goListUpdateRe = /^(\S+)\s+(v\S+)\s+\[([^\]]+)\]/;

function parseGoList(text: string, goModPath: string): string[][] {
  const rows: string[][] = [];
  for (const line of text.split("\n")) {
    const t = line.trim();
    if (!t || t.startsWith("go:")) continue;
    const m = goListUpdateRe.exec(t);
    if (!m) continue;
    const path = m[1];
    const current = m[2];
    const avail = m[3].trim();
    rows.push([path, current, avail, avail, goModPath]);
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
    const rows = parseGoList(stdin, gm);
    if (rows.length === 0) {
      renderBox(
        [["(no module lines with a newer [version] from go list -u -m all)", "—", "—", "—", gm]],
        "go",
        repoRoot,
      );
    } else {
      renderBox(rows, "go", repoRoot);
    }
  }
}
