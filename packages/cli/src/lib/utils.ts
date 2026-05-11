// --------------------
// semver (loose numeric core)
// --------------------

function semverCoreParts(v: string): [number, number, number] {
  const core = (v.split("-")[0] ?? v).trim();
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
