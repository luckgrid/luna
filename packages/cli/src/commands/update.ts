import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  collectInRangeMinorBumps,
  collectPrereleaseBumps,
  listBunWorkspacePackageDirs,
  runPrereleaseBumps,
  syncRootPackageManagerBun,
} from "../lib/bun";
import {
  findRepoRoot,
  requireCmd,
  runOrExit,
  section,
  spawnExit,
  tryReadOutdatedCache,
} from "../lib/commands";
import {
  goFullGraphOutdatedEnabled,
  isGoModuleToolOnly,
  readGoModToolPaths,
  verifyGoModuleAfterDependencyUpdate,
} from "../lib/go";
import { listGoModuleRoots, listUvProjectRoots } from "../lib/moon";
import { installAllProtoPinnedTools, protoOutdatedUpdateArgs, protoRunOpts } from "../lib/proto";
import { formatProjectDirLabel } from "../lib/utils";
import { gatherOutdatedSnapshotAsync, printOutdatedCheckSummary } from "./outdated";

export type RunUpdateOptions = {
  /** When true, allow major-version bumps for proto pins and Bun deps and run the prerelease catch-up step. */
  major: boolean;
  /** When true, use `.cache/outdated-snapshot.json` if fingerprint still matches (skip live precheck). */
  useOutdatedCache: boolean;
};

export async function runUpdate(opts: RunUpdateOptions): Promise<number> {
  const repoRoot = findRepoRoot();
  const { major, useOutdatedCache } = opts;

  section("current outdated snapshot");
  let snap;
  if (useOutdatedCache) {
    const cached = tryReadOutdatedCache(repoRoot);
    if (cached) {
      console.log(
        "[luna] using cached outdated snapshot (.cache/outdated-snapshot.json; fingerprint match)",
      );
      snap = cached;
    } else {
      snap = await gatherOutdatedSnapshotAsync(repoRoot);
    }
  } else {
    snap = await gatherOutdatedSnapshotAsync(repoRoot);
  }
  printOutdatedCheckSummary(repoRoot, snap);

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
  } else {
    // Widen ranges for "not a real major" upgrades that caret semver leaves stuck
    // (e.g. 0.48.0 → 0.49.0, where npm treats the minor as breaking).
    const bunOutPost = captureBunOutdatedRecursive(repoRoot);
    if (bunWorkspaceOutdatedFromOutput(bunOutPost)) {
      const minorBumps = collectInRangeMinorBumps(bunOutPost, repoRoot);
      if (minorBumps.length > 0) {
        section("Bun — widen ranges for non-major upgrades (0.x → 0.x+1 etc.)");
        runOrExit(runPrereleaseBumps(minorBumps), "bun add @latest (non-major widening)");
      }
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
      const toolOnly = isGoModuleToolOnly(root) && !goFullGraphOutdatedEnabled();
      const tools = readGoModToolPaths(root);

      if (toolOnly) {
        section(
          major
            ? `Go — ${label} (go get -tool @latest per tool && go mod tidy && verify)`
            : `Go — ${label} (go get -u=patch per tool && go mod tidy && verify)`,
        );
        if (major) {
          for (const tool of tools) {
            runOrExit(
              spawnExit(["go", "get", "-tool", `${tool}@latest`], { cwd: root }),
              `go get -tool ${tool}@latest (${label})`,
            );
          }
        } else {
          for (const tool of tools) {
            runOrExit(
              spawnExit(["go", "get", "-u=patch", tool], { cwd: root }),
              `go get -u=patch ${tool} (${label})`,
            );
          }
        }
      } else {
        section(
          major
            ? `Go — ${label} (go get go@latest + tool@latest + -u all && go mod tidy && verify)`
            : `Go — ${label} (go get -u all && go mod tidy && verify)`,
        );
        if (major) {
          runOrExit(
            spawnExit(["go", "get", "go@latest"], { cwd: root }),
            `go get go@latest (${label})`,
          );
          for (const tool of tools) {
            runOrExit(
              spawnExit(["go", "get", `${tool}@latest`], { cwd: root }),
              `go get ${tool}@latest (${label})`,
            );
          }
        }
        runOrExit(spawnExit(["go", "get", "-u", "all"], { cwd: root }), `go get -u all (${label})`);
      }

      runOrExit(spawnExit(["go", "mod", "tidy"], { cwd: root }), `go mod tidy (${label})`);
      const verify = verifyGoModuleAfterDependencyUpdate(root);
      if (!verify.ok) {
        console.error(`[luna] Go module verify failed (${label}):\n${verify.reason}`);
        return 1;
      }
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
