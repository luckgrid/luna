#!/usr/bin/env bun
/**
 * Luna CLI — **Bun** only. This file is the `package.json` `bin` entry (native binary: `bun run build` in this package).
 *
 * Uses `Bun.argv`, `Bun.spawnSync` in toolchains/commands, and prefix-only global flags
 * (see [How to Build CLI Applications with Bun](https://oneuptime.com/blog/post/2026-01-31-bun-cli-applications/view)).
 */
import { printRootHelp } from "./commands/help";
import { readCliPackageVersion } from "./commands/version";

function registerInterrupt(sig: NodeJS.Signals, code: number): void {
  process.once(sig, () => {
    console.error(`\n${sig}: interrupted`);
    process.exit(code);
  });
}

function installSignalHandlers(): void {
  registerInterrupt("SIGINT", 130);
  registerInterrupt("SIGTERM", 143);
}

/** Global flags only before the first non-option token. */
function stripLeadingGlobalFlags(argv: string[]): {
  help: boolean;
  version: boolean;
  rest: string[];
} {
  let i = 0;
  let help = false;
  let version = false;
  while (i < argv.length) {
    const a = argv[i];
    if (a === "-h" || a === "--help") {
      help = true;
      i += 1;
      continue;
    }
    if (a === "-v" || a === "-V" || a === "--version") {
      version = true;
      i += 1;
      continue;
    }
    break;
  }
  return { help, version, rest: argv.slice(i) };
}

function restHasHelpFlag(rest: string[]): boolean {
  return rest.some((a) => a === "-h" || a === "--help");
}

export async function main(): Promise<void> {
  installSignalHandlers();

  const argv = Bun.argv.slice(2);

  if (argv.length === 0) {
    printRootHelp();
    process.exit(0);
  }

  const { help, version, rest } = stripLeadingGlobalFlags(argv);

  if (version) {
    console.log(`luna ${readCliPackageVersion()}`);
    process.exit(0);
  }

  if (help) {
    printRootHelp();
    process.exit(0);
  }

  if (restHasHelpFlag(rest)) {
    printRootHelp();
    process.exit(0);
  }

  const cmd = rest[0];
  if (!cmd) {
    printRootHelp();
    process.exit(0);
  }

  if (rest.length > 1) {
    console.error(`error: unexpected arguments: ${rest.slice(1).join(" ")}`);
    printRootHelp();
    process.exit(2);
  }

  if (cmd === "help") {
    printRootHelp();
    process.exit(0);
  }

  let code: number;
  switch (cmd) {
    case "outdated": {
      const { runOutdated } = await import("./commands/outdated");
      code = runOutdated();
      break;
    }
    case "update": {
      const { runUpdate } = await import("./commands/update");
      code = runUpdate();
      break;
    }
    default:
      console.error(`error: unknown command: ${cmd}`);
      printRootHelp();
      code = 2;
  }

  process.exit(code);
}

if (import.meta.main) {
  await main();
}
