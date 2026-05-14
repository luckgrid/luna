import { die } from "./terminal";

export function requireCmd(name: string): void {
  const r = Bun.spawnSync(["/bin/sh", "-c", `command -v "${name.replace(/"/g, '\\"')}"`], {
    stdout: "ignore",
    stderr: "ignore",
  });
  if (r.exitCode !== 0) die(`missing required command: ${name}`);
}

/** stdout+stderr combined, UTF-8. Does not throw on non-zero exit. */
export function spawnText(
  cmd: string[],
  opts: { cwd?: string; env?: NodeJS.ProcessEnv } = {},
): string {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    env: opts.env ?? process.env,
    stdout: "pipe",
    stderr: "pipe",
    stdin: "ignore",
  });
  const out = new TextDecoder().decode(r.stdout);
  const err = new TextDecoder().decode(r.stderr);
  return out + err;
}

/** Async variant of {@link spawnText} for parallel toolchain probes. */
export async function spawnTextAsync(
  cmd: string[],
  opts: { cwd?: string; env?: NodeJS.ProcessEnv } = {},
): Promise<string> {
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

export function spawnExit(
  cmd: string[],
  opts: { cwd?: string; stdin?: "ignore" | "inherit"; env?: NodeJS.ProcessEnv } = {},
): number {
  const r = Bun.spawnSync(cmd, {
    cwd: opts.cwd,
    env: opts.env ?? process.env,
    stdin: opts.stdin ?? "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });
  return r.exitCode ?? 1;
}

export function runOrExit(code: number, step: string): void {
  if (code !== 0) {
    console.error(`error: ${step} (exit ${code})`);
    process.exit(code);
  }
}
