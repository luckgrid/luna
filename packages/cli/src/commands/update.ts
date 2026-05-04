import { fileURLToPath } from "node:url";
import { formatProjectDirLabel, listGoModuleRoots, listUvProjectRoots } from "../lib/repo";
import { requireCmd, spawnExit } from "../lib/process";
import { findRepoRoot, syncRootPackageManagerBun } from "../lib/repo";
import { section } from "../lib/term";
import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  collectPrereleaseBumps,
  runPrereleaseBumps,
} from "../toolchains/bun";
import { goApplyModfileModuleUpdates } from "../toolchains/go";

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
  runOrExit(
    spawnExit(["bun", "update", "--latest", "--recursive", "--force"], { cwd: repoRoot }),
    "bun update",
  );

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

  const goRoots = listGoModuleRoots(repoRoot);
  if (goRoots.length === 0) {
    section(
      "Go — no modules discovered (add moon.yml + language: go + go.mod, or set GO_MODULE_ROOT)",
    );
  } else {
    requireCmd("go");
    for (const root of goRoots) {
      const label = formatProjectDirLabel(repoRoot, root);
      section(`Go — ${label} (module upgrades + tidy)`);
      goApplyModfileModuleUpdates(root);
    }
  }

  section("done — verify with bun run outdated and bun run check");
  console.log("Update steps finished. Review changes before committing.");
  return 0;
}
