import { spawnText, spawnTextAsync } from "./utils";

// --------------------
// python / uv
// --------------------

const uvUpdateRe = /^Update ([^ ]+) v(.+) -> v(.+)$/;

/** Python / uv: `uv lock --upgrade --dry-run` lines that indicate upgrades. */
export function uvLockHasUpgradesFromOutput(out: string): boolean {
  return /^Update /m.test(out);
}

export function captureUvLockDryRun(uvProjectRoot: string): string {
  return spawnText(["uv", "lock", "--upgrade", "--dry-run"], { cwd: uvProjectRoot });
}

export async function captureUvLockDryRunAsync(uvProjectRoot: string): Promise<string> {
  return spawnTextAsync(["uv", "lock", "--upgrade", "--dry-run"], { cwd: uvProjectRoot });
}

/** Parse `uv lock --upgrade --dry-run` lines into outdated table rows. */
export function parseUvDryRunTableRows(text: string, pyproject: string): string[][] {
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
