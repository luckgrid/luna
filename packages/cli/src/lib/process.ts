import { die } from "./term";

export function requireCmd(name: string): void {
  const r = Bun.spawnSync(["/bin/sh", "-c", `command -v "${name.replace(/"/g, '\\"')}"`], {
    stdout: "ignore",
    stderr: "ignore",
  });
  if (r.exitCode !== 0) die(`missing required command: ${name}`);
}

/** stdout+stderr combined, UTF-8. Does not throw on non-zero exit. */
export function spawnText(cmd: string[], opts: { cwd?: string } = {}): string {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    stdout: "pipe",
    stderr: "pipe",
  });
  const out = new TextDecoder().decode(r.stdout);
  const err = new TextDecoder().decode(r.stderr);
  return out + err;
}

export function spawnExit(
  cmd: string[],
  opts: { cwd?: string; stdin?: "ignore" | "inherit" } = {},
): number {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    stdin: opts.stdin ?? "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });
  return r.exitCode ?? 1;
}
