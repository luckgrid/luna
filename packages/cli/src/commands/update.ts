import { fileURLToPath } from "node:url";
import {
  findRepoRoot,
  formatProjectDirLabel,
  listBunWorkspacePackageDirs,
  listUvProjectRoots,
  syncRootPackageManagerBun,
} from "../lib/repo";
import { requireCmd, spawnExit } from "../lib/process";
import { section } from "../lib/term";
import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  collectPrereleaseBumps,
  runPrereleaseBumps,
} from "../lib/toolchains";

function runOrExit(code: number, step: string): void {
  if (code !== 0) {
    console.error(`error: ${step} (exit ${code})`);
    process.exit(code);
  }
}

export function runUpdate(): number {
  const repoRoot = findRepoRoot();
  const cliEntry = fileURLToPath(new URL("../main.ts", import.meta.url));

  section("current outdated snapshot");
  Bun.spawnSync(["bun", cliEntry, "outdated"], {
    cwd: repoRoot,
    stdin: "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });

  section("proto — write latest tool versions to .prototools");
  requireCmd("proto");
  runOrExit(
    spawnExit(["proto", "outdated", "--update", "--latest", "-y"], { cwd: repoRoot }),
    "proto outdated --update",
  );

  section("proto — install pins from .prototools");
  runOrExit(spawnExit(["proto", "install"], { cwd: repoRoot }), "proto install");

  section("sync root packageManager with .prototools bun pin");
  syncRootPackageManagerBun(repoRoot);

  section("Bun — bump workspace deps to latest semver");
  requireCmd("bun");
  const bunUpdateLatest = ["bun", "update", "--latest", "--force", "--ignore-scripts"] as const;
  // Skip lifecycle scripts: avoids redundant work during semver bumps (bootstrap is `bun run setup`).
  runOrExit(
    spawnExit([...bunUpdateLatest, "--recursive"], { cwd: repoRoot }),
    "bun update (repo root)",
  );
  for (const dir of listBunWorkspacePackageDirs(repoRoot)) {
    const label = formatProjectDirLabel(repoRoot, dir);
    runOrExit(spawnExit([...bunUpdateLatest], { cwd: dir }), `bun update (${label})`);
  }

  const bunOutPost = captureBunOutdatedRecursive(repoRoot);
  if (bunWorkspaceOutdatedFromOutput(bunOutPost)) {
    const bumps = collectPrereleaseBumps(bunOutPost, repoRoot);
    runOrExit(runPrereleaseBumps(bumps), "bun add @latest (prerelease bumps)");
  }

  const uvRoots = listUvProjectRoots(repoRoot);
  if (uvRoots.length === 0) {
    section(
      "Python / uv — no projects discovered (add moon.yml + language: python + pyproject.toml, or set UV_PROJECT_ROOT)",
    );
  } else {
    requireCmd("uv");
    for (const root of uvRoots) {
      const label = formatProjectDirLabel(repoRoot, root);
      section(`Python / uv — ${label} (uv lock --upgrade && uv sync)`);
      runOrExit(
        spawnExit(["uv", "lock", "--upgrade"], { cwd: root }),
        `uv lock --upgrade (${label})`,
      );
      runOrExit(spawnExit(["uv", "sync"], { cwd: root }), `uv sync (${label})`);
    }
  }

  section("repo setup — proto + bun workspaces + moon stacks");
  runOrExit(spawnExit(["bun", "run", "setup"], { cwd: repoRoot }), "bun run setup");

  section("done — verify with bun run outdated and bun run check");
  console.log("Update steps finished. Review changes before committing.");
  return 0;
}
