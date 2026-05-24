import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { die, spawnExit, spawnText, spawnTextAsync } from "./utils";

// --------------------
// proto (pins in .prototools)
// --------------------

/**
 * Strip per-tool runtime hints that Bun (and other proto shims) inject so
 * `proto outdated` resolves pin sources from `.prototools` instead of treating
 * the running tool as "version-locked by env" (which silently drops `config_source`
 * and prevents `--update` from rewriting the pin).
 *
 * Keeps `PROTO_VERSION` (used by proto for self-reflection).
 */
export function protoScrubbedEnv(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env };
  for (const k of Object.keys(env)) {
    if (k === "PROTO_VERSION") continue;
    if (/^PROTO_[A-Z0-9]+_VERSION$/.test(k)) delete env[k];
  }
  return { ...env, ...extra };
}

/** One toolchain row from \`proto outdated --json\`. */
export type ProtoPinOutdatedEntry = {
  is_latest: boolean;
  is_outdated: boolean;
  config_source: string;
  config_version: string;
  current_version: string;
  newest_version: string;
  latest_version: string;
};

export type ProtoPinsOutdatedReport = Record<string, ProtoPinOutdatedEntry>;

/**
 * When `proto outdated --json` runs as a subprocess of Bun, the `bun` tool row often has
 * `config_source: null` (self-runtime / no separate pin file in JSON). Treat null like unknown.
 */
function isProtoPinOutdatedEntryJson(v: unknown): v is Omit<
  ProtoPinOutdatedEntry,
  "config_source"
> & {
  config_source: string | null;
} {
  if (typeof v !== "object" || v === null) return false;
  const bool = (k: string) => typeof Reflect.get(v, k) === "boolean";
  const str = (k: string) => typeof Reflect.get(v, k) === "string";
  const cs = Reflect.get(v, "config_source");
  const configSourceOk = typeof cs === "string" || cs === null || cs === undefined;
  return (
    bool("is_latest") &&
    bool("is_outdated") &&
    configSourceOk &&
    str("config_version") &&
    str("current_version") &&
    str("newest_version") &&
    str("latest_version")
  );
}

function isProtoPinsOutdatedReportJson(
  v: unknown,
): v is Record<
  string,
  Omit<ProtoPinOutdatedEntry, "config_source"> & { config_source: string | null }
> {
  if (typeof v !== "object" || v === null) return false;
  for (const val of Object.values(v)) {
    if (!isProtoPinOutdatedEntryJson(val)) return false;
  }
  return true;
}

export function protoPinsAnyOutdated(report: ProtoPinsOutdatedReport): boolean {
  return Object.values(report).some((x) => x.is_outdated);
}

/** Parse stdout of `proto outdated --json` (combined stdout+stderr also ok). */
export function parseProtoPinsOutdatedJson(text: string): ProtoPinsOutdatedReport {
  const trimmed = text.trim();
  if (!trimmed) die("proto outdated --json returned empty output (is proto in PATH?)");
  let data: unknown;
  try {
    data = JSON.parse(trimmed);
  } catch {
    die("proto outdated --json returned invalid JSON");
  }
  if (!isProtoPinsOutdatedReportJson(data)) die("proto outdated --json: unexpected shape");
  const out: ProtoPinsOutdatedReport = {};
  for (const [tool, row] of Object.entries(data)) {
    out[tool] = {
      ...row,
      config_source: typeof row.config_source === "string" ? row.config_source : "",
    };
  }
  return out;
}

/** `proto outdated --json`, parsed — run from repo root so .prototools resolves. */
export function captureProtoPinsOutdatedJson(repoRoot: string): ProtoPinsOutdatedReport {
  const text = spawnText(["proto", "outdated", "--json"], {
    cwd: repoRoot,
    env: protoScrubbedEnv(),
  });
  return parseProtoPinsOutdatedJson(text);
}

/** Async `proto outdated --json` for parallel outdated gathering. */
export async function captureProtoPinsOutdatedJsonAsync(
  repoRoot: string,
): Promise<ProtoPinsOutdatedReport> {
  const text = await spawnTextAsync(["proto", "outdated", "--json"], {
    cwd: repoRoot,
    env: protoScrubbedEnv(),
  });
  return parseProtoPinsOutdatedJson(text);
}

export function protoOutdatedUpdateArgs(major: boolean): readonly string[] {
  return major
    ? ["proto", "outdated", "--update", "--latest", "-y"]
    : ["proto", "outdated", "--update", "-y"];
}

const PROTO_TOOL_PIN_RE = /^(bun|moon|proto|python|go)\s*=\s*"([^"]+)"\s*$/;

/** Run proto from `.proto/logs` so install failure logs stay out of the repo root; resolve repo `.prototools` via parent search. */
export function protoRunOpts(repoRoot: string): {
  cwd: string;
  env: NodeJS.ProcessEnv;
} {
  const logsDir = join(repoRoot, ".proto/logs");
  mkdirSync(logsDir, { recursive: true });
  return {
    cwd: logsDir,
    env: protoScrubbedEnv({ PROTO_CONFIG_MODE: "upwards" }),
  };
}

/** Pinned tools from the root (implicit) table of `.prototools` — ignores `[table]` sections. */
export function readProtoPinnedTools(repoRoot: string): { name: string; version: string }[] {
  const path = join(repoRoot, ".prototools");
  const raw = readFileSync(path, "utf8");
  const implicit = /^\[/m.test(raw) ? (raw.split(/^\[/m)[0] ?? raw) : raw;
  const seen = new Set<string>();
  const out: { name: string; version: string }[] = [];
  for (const line of implicit.split(/\r?\n/)) {
    const m = PROTO_TOOL_PIN_RE.exec(line.trim());
    if (!m) continue;
    const name = m[1];
    const version = m[2];
    if (seen.has(name)) continue;
    seen.add(name);
    out.push({ name, version });
  }
  return out;
}

/** One proto tool id from `.prototools`; runs under {@link protoRunOpts} (logs in `.proto/logs/`). */
export function protoInstallPinnedTool(repoRoot: string, tool: string): number {
  const opts = protoRunOpts(repoRoot);
  if (tool === "python") {
    let code = spawnExit(["proto", "install", "python", "-y"], opts);
    if (code !== 0) {
      console.error(
        "[luna] python: pre-built install failed; retrying with `proto install python --build` (slower, no standalone tarball yet)",
      );
      code = spawnExit(["proto", "install", "python", "--build", "-y"], opts);
    }
    return code;
  }
  return spawnExit(["proto", "install", tool, "-y"], opts);
}

/**
 * Install each pin in `.prototools` with proto, with python pre-built fallbacks Luna needs in practice.
 */
export function installAllProtoPinnedTools(repoRoot: string): number {
  const tools = readProtoPinnedTools(repoRoot);

  for (const { name } of tools) {
    const code = protoInstallPinnedTool(repoRoot, name);
    if (code !== 0) return code;
  }

  return 0;
}
