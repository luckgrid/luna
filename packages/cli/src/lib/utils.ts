import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";

// --------------------
// process
// --------------------

export type SpawnOpts = {
  cwd?: string;
  env?: NodeJS.ProcessEnv;
};

export type SpawnExitOpts = SpawnOpts & {
  stdin?: "ignore" | "inherit";
};

export type SpawnCaptureResult = {
  exitCode: number;
  stdout: string;
  stderr: string;
};

const textDecoder = new TextDecoder();

function decode(bytes: Uint8Array | ArrayBufferLike): string {
  return textDecoder.decode(bytes);
}

/** Sync spawn; stdout and stderr as UTF-8 strings (combined). Does not throw on non-zero exit. */
export function spawnText(cmd: string[], opts: SpawnOpts = {}): string {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    env: opts.env ?? process.env,
    stdout: "pipe",
    stderr: "pipe",
    stdin: "ignore",
  });
  return decode(r.stdout) + decode(r.stderr);
}

/** Async spawn; stdout and stderr combined. Does not throw on non-zero exit. */
export async function spawnTextAsync(cmd: string[], opts: SpawnOpts = {}): Promise<string> {
  const subprocess = Bun.spawn(cmd, {
    cwd: opts.cwd,
    env: opts.env ?? process.env,
    stdout: "pipe",
    stderr: "pipe",
    stdin: "ignore",
  });
  const [out, err] = await Promise.all([
    new Response(subprocess.stdout).text(),
    new Response(subprocess.stderr).text(),
  ]);
  await subprocess.exited;
  return out + err;
}

/** Sync spawn with inherited stdio; returns exit code. */
export function spawnExit(cmd: string[], opts: SpawnExitOpts = {}): number {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    env: opts.env ?? process.env,
    stdin: opts.stdin ?? "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });
  return r.exitCode ?? 1;
}

/** Sync spawn with piped stdout/stderr and exit code (for success/failure checks). */
export function spawnSyncCaptured(cmd: string[], opts: SpawnOpts = {}): SpawnCaptureResult {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    env: opts.env ?? process.env,
    stdout: "pipe",
    stderr: "pipe",
    stdin: "ignore",
  });
  return {
    exitCode: r.exitCode ?? 1,
    stdout: decode(r.stdout),
    stderr: decode(r.stderr),
  };
}

export function die(msg: string): never {
  console.error(`error: ${msg}`);
  process.exit(1);
}

/** True when `process.env[name]` is `1`, `true`, or `yes` (case-insensitive). */
export function envFlagEnabled(name: string): boolean {
  const v = process.env[name]?.trim().toLowerCase();
  return v === "1" || v === "true" || v === "yes";
}

/** Non-empty trimmed lines from command or file text. */
export function nonEmptyLines(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);
}

// --------------------
// paths
// --------------------

/** Top-level workspace folders scanned for apps/packages (Moon + Bun). */
export const WORKSPACE_CHILD_DIRS = ["apps", "packages"] as const;

export function* eachWorkspaceChild(
  repoRoot: string,
): Generator<{ top: (typeof WORKSPACE_CHILD_DIRS)[number]; name: string; dir: string }> {
  for (const top of WORKSPACE_CHILD_DIRS) {
    const base = join(repoRoot, top);
    if (!existsSync(base)) continue;
    for (const ent of readdirSync(base, { withFileTypes: true })) {
      if (!ent.isDirectory()) continue;
      yield { top, name: ent.name, dir: join(base, ent.name) };
    }
  }
}

/** `package.json` `name` field, or null if missing/unreadable. */
export function readPackageJsonName(pkgJsonPath: string): string | null {
  try {
    const j: unknown = JSON.parse(readFileSync(pkgJsonPath, "utf8"));
    if (typeof j === "object" && j !== null && "name" in j) {
      const rec = j as Record<string, unknown>;
      if (typeof rec.name === "string") return rec.name;
    }
  } catch {
    /* ignore */
  }
  return null;
}

export function uniqSorted(paths: string[]): string[] {
  return [...new Set(paths.map((p) => resolve(p)))].toSorted();
}

export function formatProjectDirLabel(repoRoot: string, dir: string): string {
  try {
    const r = relative(repoRoot, resolve(dir)).replace(/\\/g, "/");
    return r && !r.startsWith("..") ? r : resolve(dir);
  } catch {
    return dir;
  }
}

// --------------------
// semver (loose numeric core)
// --------------------

/** Loose semver: numeric `[major, minor, patch]` parsed from `x.y.z` (suffixes stripped). */
export function semverCoreParts(v: string): [number, number, number] {
  const core = (v.split("-")[0] ?? v).trim().replace(/^v/, "");
  const p = core.split(".").map((x) => Number.parseInt(x, 10));
  return [p[0] ?? 0, p[1] ?? 0, p[2] ?? 0];
}

/** Loose semver compare on `x.y.z` numeric core (suffixes stripped). */
export function semverGte(a: string, b: string): boolean {
  const A = semverCoreParts(a);
  const B = semverCoreParts(b);
  for (let i = 0; i < 3; i++) {
    if (A[i] > B[i]) return true;
    if (A[i] < B[i]) return false;
  }
  return true;
}

// --------------------
// strings
// --------------------

export function trimOuterWhitespace(s: string): string {
  return s.replace(/\s+$/g, "").replace(/^\s+/g, "");
}

/** Middle-ellipsis truncation for fixed-width UI. */
export function shortenMiddle(s: string, maxLen: number): string {
  if (s.length <= maxLen) return s;
  const el = "…";
  const inner = maxLen - el.length;
  const left = Math.ceil(inner * 0.55);
  const right = inner - left;
  return s.slice(0, left) + el + s.slice(-right);
}

/** Shorten only when cell looks like a version label. */
export function shortenVersionCell(s: string, max: number): string {
  if (s.length <= max) return s;
  if (!/^v[\d._-]|^[\d]/.test(s)) return s;
  return shortenMiddle(s, max);
}

// --------------------
// env + terminal capability
// --------------------

export function readOptionalIntEnvMin(key: string, fallback: number, min: number): number {
  const raw = process.env[key];
  if (raw !== undefined && raw !== "") {
    const n = Number.parseInt(raw, 10);
    if (Number.isFinite(n) && n >= min) return n;
  }
  return fallback;
}

export function terminalAnsiStdout(): boolean {
  return !process.env.NO_COLOR && process.stdout.isTTY && process.env.TERM !== "dumb";
}

export function terminalAnsiStderr(): boolean {
  return !process.env.NO_COLOR && process.stderr.isTTY && process.env.TERM !== "dumb";
}

/** OSC 8 hyperlink support — stdout, honour NO_COLOR / OUTDATED_NO_TERMINAL_LINKS / dumb TERM. */
export function terminalHyperlinksSupported(): boolean {
  return (
    !process.env.NO_COLOR &&
    !process.env.OUTDATED_NO_TERMINAL_LINKS &&
    process.stdout.isTTY &&
    process.env.TERM !== "dumb"
  );
}
