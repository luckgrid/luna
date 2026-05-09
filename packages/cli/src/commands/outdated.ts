import { join } from "node:path";
import { printOutdatedTable } from "../lib/format";
import { findRepoRoot, formatProjectDirLabel, listUvProjectRoots } from "../lib/repo";
import {
  section,
  strictAllPassed,
  strictHint,
  strictNeed,
  strictOk,
  strictSummaryBullet,
  strictSummaryFailTitle,
} from "../lib/term";
import { requireCmd } from "../lib/process";
import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  captureUvLockDryRun,
  printProtoOutdated,
  protoHasOutdatedPins,
  uvLockHasUpgradesFromOutput,
} from "../lib/toolchains";

export type UvProjectSnap = { root: string; dryRunOut: string };

export type OutdatedSnapshot = {
  bunOut: string;
  uvProjects: UvProjectSnap[];
};

export function gatherOutdatedSnapshot(repoRoot: string): OutdatedSnapshot {
  const bunOut = captureBunOutdatedRecursive(repoRoot);

  const uvProjects: UvProjectSnap[] = listUvProjectRoots(repoRoot).map((root) => ({
    root,
    dryRunOut: captureUvLockDryRun(root),
  }));

  return { bunOut, uvProjects };
}

/** Human report: only tiers with something to say print a section. */
export function printOutdatedReport(repoRoot: string, snap: OutdatedSnapshot): void {
  if (protoHasOutdatedPins()) {
    section("proto (.prototools — moon, bun, python, proto)");
    printProtoOutdated();
  }

  if (bunWorkspaceOutdatedFromOutput(snap.bunOut)) {
    section("Bun (workspaces — root package.json + apps/* + packages/*)");
    printOutdatedTable("bun", snap.bunOut, { repoRoot });
  }

  for (const p of snap.uvProjects) {
    if (!uvLockHasUpgradesFromOutput(p.dryRunOut)) continue;
    const label = formatProjectDirLabel(repoRoot, p.root);
    section(`Python / uv (${label} — pyproject.toml + uv.lock)`);
    printOutdatedTable("uv", p.dryRunOut, {
      repoRoot,
      pyprojectPath: join(p.root, "pyproject.toml"),
    });
  }
}

/** Report all tiers, then enforce CI-style exit (1 if any tier has upgrades). */
export function runOutdated(): number {
  const repoRoot = findRepoRoot();
  requireCmd("proto");
  requireCmd("bun");
  requireCmd("uv");

  const snap = gatherOutdatedSnapshot(repoRoot);
  printOutdatedReport(repoRoot, snap);

  section("check results");
  let failed = 0;
  let stProto = 0;
  let stBun = 0;
  let stUv = 0;

  if (protoHasOutdatedPins()) {
    strictNeed("proto — outdated tool pin(s) (.prototools)");
    stProto = 1;
    failed = 1;
  } else {
    strictOk("proto — OK (.prototools)");
  }

  if (bunWorkspaceOutdatedFromOutput(snap.bunOut)) {
    strictNeed("Bun — outdated direct dependencies in workspaces");
    stBun = 1;
    failed = 1;
  } else {
    strictOk("Bun — OK (workspaces)");
  }

  const uvBad = snap.uvProjects.filter((p) => uvLockHasUpgradesFromOutput(p.dryRunOut));
  if (uvBad.length > 0) {
    strictNeed(
      `Python / uv — lockfile(s) can update: ${uvBad.map((p) => formatProjectDirLabel(repoRoot, p.root)).join(", ")} (see bun run update)`,
    );
    stUv = 1;
    failed = 1;
  } else if (snap.uvProjects.length === 0) {
    strictOk("Python / uv — OK (no uv projects discovered)");
  } else {
    strictOk(`Python / uv — OK (${snap.uvProjects.length} project(s))`);
  }

  if (failed === 0) {
    strictAllPassed("All checks passed (nothing reported as outdated).");
  } else {
    console.error("");
    strictSummaryFailTitle("Outdated check failed — upgrades reported in:");
    if (stProto) strictSummaryBullet("proto (.prototools)");
    if (stBun) strictSummaryBullet("Bun workspaces");
    if (stUv) strictSummaryBullet("Python / uv lockfile(s)");
    console.error("");
    strictHint("Exit code 1 is intentional (use in CI). To refresh everything run: luna update");
  }

  return failed;
}
