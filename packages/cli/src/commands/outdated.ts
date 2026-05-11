import { join } from "node:path";
import {
  findRepoRoot,
  formatProjectDirLabel,
  listGoModuleRoots,
  listUvProjectRoots,
} from "../lib/repo";
import {
  printOutdatedTable,
  section,
  strictAllPassed,
  strictHint,
  strictNeed,
  strictOk,
  strictSummaryBullet,
  strictSummaryFailTitle,
} from "../lib/terminal";
import { requireCmd } from "../lib/process";
import {
  bunWorkspaceOutdatedFromOutput,
  captureBunOutdatedRecursive,
  captureGoGetNDryRunUAll,
  captureProtoPinsOutdatedJson,
  captureUvLockDryRun,
  goGetDryRunHasModuleChanges,
  type ProtoPinsOutdatedReport,
  printProtoOutdated,
  protoPinsAnyOutdated,
  uvLockHasUpgradesFromOutput,
} from "../lib/toolchains";

export type UvProjectSnap = { root: string; dryRunOut: string };

export type GoModuleSnap = { root: string; goGetDryRunOut: string };

export type OutdatedSnapshot = {
  protoReport: ProtoPinsOutdatedReport;
  bunOut: string;
  uvProjects: UvProjectSnap[];
  goModules: GoModuleSnap[];
};

export function gatherOutdatedSnapshot(repoRoot: string): OutdatedSnapshot {
  const protoReport = captureProtoPinsOutdatedJson(repoRoot);
  const bunOut = captureBunOutdatedRecursive(repoRoot);

  const uvProjects: UvProjectSnap[] = listUvProjectRoots(repoRoot).map((root) => ({
    root,
    dryRunOut: captureUvLockDryRun(root),
  }));

  const goModules: GoModuleSnap[] = listGoModuleRoots(repoRoot).map((root) => ({
    root,
    goGetDryRunOut: captureGoGetNDryRunUAll(root),
  }));

  return { protoReport, bunOut, uvProjects, goModules };
}

/** Human report: only tiers with something to say print a section. */
export function printOutdatedReport(repoRoot: string, snap: OutdatedSnapshot): void {
  if (protoPinsAnyOutdated(snap.protoReport)) {
    section("proto (.prototools)");
    printProtoOutdated(repoRoot);
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

  for (const g of snap.goModules) {
    if (!goGetDryRunHasModuleChanges(g.goGetDryRunOut)) continue;
    const label = formatProjectDirLabel(repoRoot, g.root);
    section(`Go (${label} — go.mod + go.sum; go get -n -u all)`);
    printOutdatedTable("go", g.goGetDryRunOut, {
      repoRoot,
      goModPath: join(g.root, "go.mod"),
    });
  }
}

/** Report all tiers, then enforce CI-style exit (1 if any tier has upgrades). */
export function runOutdated(): number {
  const repoRoot = findRepoRoot();
  const goRootsPrecheck = listGoModuleRoots(repoRoot);
  requireCmd("proto");
  requireCmd("bun");
  requireCmd("uv");
  if (goRootsPrecheck.length > 0) requireCmd("go");

  const snap = gatherOutdatedSnapshot(repoRoot);
  printOutdatedReport(repoRoot, snap);

  section("check results");
  let failed = 0;
  let stProto = 0;
  let stBun = 0;
  let stUv = 0;
  let stGo = 0;

  if (protoPinsAnyOutdated(snap.protoReport)) {
    strictNeed("proto — outdated pin(s) in .prototools");
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
      `Python / uv — lockfile(s) can update: ${uvBad.map((p) => formatProjectDirLabel(repoRoot, p.root)).join(", ")} (see luna update)`,
    );
    stUv = 1;
    failed = 1;
  } else if (snap.uvProjects.length === 0) {
    strictOk("Python / uv — OK (no uv projects discovered)");
  } else {
    strictOk(`Python / uv — OK (${snap.uvProjects.length} project(s))`);
  }

  const goBad = snap.goModules.filter((g) => goGetDryRunHasModuleChanges(g.goGetDryRunOut));
  if (goBad.length > 0) {
    strictNeed(
      `Go — go.mod can advance: ${goBad.map((g) => formatProjectDirLabel(repoRoot, g.root)).join(", ")} (see luna update)`,
    );
    stGo = 1;
    failed = 1;
  } else if (snap.goModules.length === 0) {
    strictOk("Go — OK (no go.mod projects discovered)");
  } else {
    strictOk(`Go — OK (${snap.goModules.length} module(s))`);
  }

  if (failed === 0) {
    strictAllPassed("All checks passed (nothing reported as outdated).");
  } else {
    console.error("");
    strictSummaryFailTitle("Outdated check failed — upgrades reported in:");
    if (stProto) strictSummaryBullet("proto (.prototools)");
    if (stBun) strictSummaryBullet("Bun workspaces");
    if (stUv) strictSummaryBullet("Python / uv lockfile(s)");
    if (stGo) strictSummaryBullet("Go module(s)");
    console.error("");
    strictHint("Exit code 1 is intentional (use in CI). To refresh everything run: luna update");
  }

  return failed;
}
