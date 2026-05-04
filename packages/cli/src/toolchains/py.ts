import { spawnText } from "../lib/process";

/** Python / uv: `uv lock --upgrade --dry-run` lines that indicate upgrades. */
export function uvLockHasUpgradesFromOutput(out: string): boolean {
  return /^Update /m.test(out);
}

export function captureUvLockDryRun(uvProjectRoot: string): string {
  return spawnText(["uv", "lock", "--upgrade", "--dry-run"], { cwd: uvProjectRoot });
}
