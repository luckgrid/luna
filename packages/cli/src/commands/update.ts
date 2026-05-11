import { fileURLToPath } from "node:url";
import {
  findRepoRoot,
  formatProjectDirLabel,
  listBunWorkspacePackageDirs,
  listGoModuleRoots,
  listUvProjectRoots,
  readGoModToolPaths,
  syncRootPackageManagerBun,
} from "../lib/repo";
import { requireCmd, runOrExit, spawnExit } from "../lib/process";
import { section } from "../lib/terminal";
import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  collectPrereleaseBumps,
  installAllProtoPinnedTools,
  protoOutdatedUpdateArgs,
  protoRunOpts,
  runPrereleaseBumps,
} from "../lib/toolchains";

export type RunUpdateOptions = {
  /** When true, allow major-version bumps for proto pins and Bun deps and run the prerelease catch-up step. */
  major: boolean;
};

export function runUpdate(opts: RunUpdateOptions): number {
  const repoRoot = findRepoRoot();
  const cliEntry = fileURLToPath(new URL("../main.ts", import.meta.url));
  const { major } = opts;

  section("current outdated snapshot");
  Bun.spawnSync(["bun", cliEntry, "outdated"], {
    cwd: repoRoot,
    stdin: "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });

  section(
    major
      ? "proto — write latest pin versions to .prototools (incl. major)"
      : "proto — update pin versions in .prototools (within manifest; no major)",
  );
  requireCmd("proto");
  runOrExit(
    spawnExit([...protoOutdatedUpdateArgs(major)], protoRunOpts(repoRoot)),
    "proto outdated --update",
  );

  section("proto — install pins from .prototools");
  runOrExit(
    installAllProtoPinnedTools(repoRoot),
    "proto install (per-tool; see .proto/logs on failure)",
  );

  section("sync root packageManager with .prototools bun pin");
  syncRootPackageManagerBun(repoRoot);

  section(
    major
      ? "Bun — bump workspace deps to latest (incl. major)"
      : "Bun — bump workspace deps within ranges (no major)",
  );
  requireCmd("bun");
  const bunUpdateBase = major
    ? (["bun", "update", "--latest", "--force", "--ignore-scripts"] as const)
    : (["bun", "update", "--force", "--ignore-scripts"] as const);
  // Skip lifecycle scripts: avoids redundant work during semver bumps (bootstrap is `bun run setup`).
  runOrExit(
    spawnExit([...bunUpdateBase, "--recursive"], { cwd: repoRoot }),
    "bun update (repo root)",
  );
  for (const dir of listBunWorkspacePackageDirs(repoRoot)) {
    const label = formatProjectDirLabel(repoRoot, dir);
    runOrExit(spawnExit([...bunUpdateBase], { cwd: dir }), `bun update (${label})`);
  }

  if (major) {
    // Prerelease catch-up only makes sense alongside --latest; in default mode, ranges drive the result.
    const bunOutPost = captureBunOutdatedRecursive(repoRoot);
    if (bunWorkspaceOutdatedFromOutput(bunOutPost)) {
      const bumps = collectPrereleaseBumps(bunOutPost, repoRoot);
      runOrExit(runPrereleaseBumps(bumps), "bun add @latest (prerelease bumps)");
    }
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
      section(
        major
          ? `Go — ${label} (go get go@latest + tool@latest + -u all && go mod tidy)`
          : `Go — ${label} (go get -u all && go mod tidy)`,
      );
      if (major) {
        runOrExit(
          spawnExit(["go", "get", "go@latest"], { cwd: root }),
          `go get go@latest (${label})`,
        );
        for (const tool of readGoModToolPaths(root)) {
          runOrExit(
            spawnExit(["go", "get", `${tool}@latest`], { cwd: root }),
            `go get ${tool}@latest (${label})`,
          );
        }
      }
      runOrExit(spawnExit(["go", "get", "-u", "all"], { cwd: root }), `go get -u all (${label})`);
      runOrExit(spawnExit(["go", "mod", "tidy"], { cwd: root }), `go mod tidy (${label})`);
    }
  }

  section("repo setup — proto + bun workspaces + moon stacks");
  runOrExit(spawnExit(["bun", "run", "setup"], { cwd: repoRoot }), "bun run setup");

  section("done — verify with bun run outdated and bun run check");
  console.log("Update steps finished. Review changes before committing.");
  if (!major) {
    console.log("Tip: re-run with `luna update --major` to also apply major-version bumps.");
  }
  return 0;
}
