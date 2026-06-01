import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  collectInRangeMinorBumps,
  collectPrereleaseBumps,
  listBunWorkspacePackageDirs,
  runPrereleaseBumps,
  syncRootPackageManagerBun,
} from "../../lib/bun";
import {
  goFullGraphOutdatedEnabled,
  isGoModuleToolOnly,
  readGoModToolPaths,
  verifyGoModuleAfterDependencyUpdate,
} from "../../lib/go";
import { listGoModuleRoots, listUvProjectRoots } from "../../lib/moon";
import { installAllProtoPinnedTools, protoOutdatedUpdateArgs, protoRunOpts } from "../../lib/proto";
import {
  findRepoRoot,
  formatProjectDirLabel,
  requireCmd,
  runOrExit,
  section,
  spawnExit,
  strictAllPassed,
  strictHint,
} from "../../lib/utils";
import { computeOutdatedSummaryState } from "../outdated/summary";
import { printOutdatedCheckSummary } from "../outdated/report";
import { printUpdatePlanSummary } from "./summary";
import { updatePlanHasWork } from "./plan";
import { loadOutdatedSnapshot } from "./snapshot";

export type RunUpdateOptions = {
  /** When true, allow major-version bumps for proto pins and Bun deps and run the prerelease catch-up step. */
  major: boolean;
  /** When true, force a live outdated precheck instead of reusing `.cache/outdated-snapshot.json`. */
  refreshOutdated: boolean;
};

export async function runUpdate(opts: RunUpdateOptions): Promise<number> {
  const repoRoot = findRepoRoot();
  const { major, refreshOutdated } = opts;

  section("current outdated snapshot");
  const { snap } = await loadOutdatedSnapshot(repoRoot, refreshOutdated);
  printOutdatedCheckSummary(repoRoot, snap, { suppressCiFooter: true });

  const plan = printUpdatePlanSummary(repoRoot, snap, major);
  if (!updatePlanHasWork(plan)) {
    console.log("");
    const { failed } = computeOutdatedSummaryState(snap);
    if (failed > 0) {
      strictAllPassed(
        "Nothing to update within current policy. Upgrades remain for `luna outdated` / CI.",
      );
      if (!major) {
        strictHint("Tip: re-run with `luna update --major` to also apply major-version bumps.");
      }
    } else {
      strictAllPassed("Nothing to update — all toolchains are up to date.");
    }
    return 0;
  }

  let anyTierUpdated = false;

  if (plan.proto) {
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
    anyTierUpdated = true;
  }

  if (plan.bun) {
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
      const bunOutPost = captureBunOutdatedRecursive(repoRoot);
      if (bunWorkspaceOutdatedFromOutput(bunOutPost)) {
        const bumps = collectPrereleaseBumps(bunOutPost, repoRoot);
        runOrExit(runPrereleaseBumps(bumps), "bun add @latest (prerelease bumps)");
      }
    } else {
      const bunOutPost = captureBunOutdatedRecursive(repoRoot);
      if (bunWorkspaceOutdatedFromOutput(bunOutPost)) {
        const minorBumps = collectInRangeMinorBumps(bunOutPost, repoRoot);
        if (minorBumps.length > 0) {
          section("Bun — widen ranges for non-major upgrades (0.x → 0.x+1 etc.)");
          runOrExit(runPrereleaseBumps(minorBumps), "bun add @latest (non-major widening)");
        }
      }
    }
    anyTierUpdated = true;
  }

  if (plan.uvRoots.length > 0) {
    requireCmd("uv");
    for (const root of plan.uvRoots) {
      const label = formatProjectDirLabel(repoRoot, root);
      section(`Python / uv — ${label} (uv lock --upgrade && uv sync)`);
      runOrExit(
        spawnExit(["uv", "lock", "--upgrade"], { cwd: root }),
        `uv lock --upgrade (${label})`,
      );
      runOrExit(spawnExit(["uv", "sync"], { cwd: root }), `uv sync (${label})`);
    }
    anyTierUpdated = true;
  } else if (listUvProjectRoots(repoRoot).length === 0) {
    section(
      "Python / uv — no projects discovered (add moon.yml + language: python + pyproject.toml, or set UV_PROJECT_ROOT)",
    );
  }

  if (plan.goModules.length > 0) {
    requireCmd("go");
    for (const root of plan.goModules) {
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
    anyTierUpdated = true;
  } else if (listGoModuleRoots(repoRoot).length === 0) {
    section(
      "Go — no modules discovered (add moon.yml + language: go + go.mod, or set GO_MODULE_ROOT)",
    );
  }

  if (anyTierUpdated) {
    section("repo setup — proto + bun workspaces + moon stacks");
    runOrExit(spawnExit(["bun", "run", "setup"], { cwd: repoRoot }), "bun run setup");

    section("done — verify with bun run outdated and bun run check");
    console.log("Update steps finished. Review changes before committing.");
    if (!major) {
      console.log("Tip: re-run with `luna update --major` to also apply major-version bumps.");
    }
  }

  return 0;
}
