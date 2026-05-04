import { join } from "node:path";
import { printOutdatedTable } from "../lib/format";
import { formatProjectDirLabel, listGoModuleRoots, listUvProjectRoots } from "../lib/repo";
import { findRepoRoot } from "../lib/repo";
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
import { captureBunOutdatedRecursive, bunWorkspaceOutdatedFromOutput } from "../toolchains/bun";
import {
  captureGoListUModule,
  goEnvGomod,
  goFilterGoOutLinesModfileUpdates,
  goModHasUpgrades,
  goModUListHasTableRows,
} from "../toolchains/go";
import { printProtoOutdated, protoHasOutdatedPins } from "../toolchains/proto";
import { captureUvLockDryRun, uvLockHasUpgradesFromOutput } from "../toolchains/py";

export type UvProjectSnap = { root: string; dryRunOut: string };
export type GoProjectSnap = { root: string; gomod: string; fullList: string; table: string };

export type OutdatedSnapshot = {
  bunOut: string;
  uvProjects: UvProjectSnap[];
  goProjects: GoProjectSnap[];
};

export function gatherOutdatedSnapshot(repoRoot: string): OutdatedSnapshot {
  const bunOut = captureBunOutdatedRecursive(repoRoot);

  const uvProjects: UvProjectSnap[] = listUvProjectRoots(repoRoot).map((root) => ({
    root,
    dryRunOut: captureUvLockDryRun(root),
  }));

  const goProjects: GoProjectSnap[] = [];
  for (const root of listGoModuleRoots(repoRoot)) {
    const gomod = goEnvGomod(root);
    if (!gomod) continue;
    const fullList = captureGoListUModule(root);
    const table = goFilterGoOutLinesModfileUpdates(gomod, fullList);
    goProjects.push({ root, gomod, fullList, table });
  }

  return { bunOut, uvProjects, goProjects };
}

/** Human report: only tiers with something to say print a section. */
export function printOutdatedReport(repoRoot: string, snap: OutdatedSnapshot): void {
  if (protoHasOutdatedPins()) {
    section("proto (.prototools — moon, bun, python, go, proto)");
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

  for (const p of snap.goProjects) {
    if (!goModUListHasTableRows(p.gomod, p.fullList)) continue;
    const label = formatProjectDirLabel(repoRoot, p.root);
    section(`Go (${label} — go.mod / go.sum; […] = newer from go list -u)`);
    printOutdatedTable("go", p.table, { repoRoot, goModPath: p.gomod });
  }
}

/** Report all tiers, then enforce CI-style exit (1 if any tier has upgrades). */
export function runOutdated(): number {
  const repoRoot = findRepoRoot();
  requireCmd("proto");
  requireCmd("bun");
  requireCmd("uv");
  requireCmd("go");

  const snap = gatherOutdatedSnapshot(repoRoot);
  printOutdatedReport(repoRoot, snap);

  section("check results");
  let failed = 0;
  let stProto = 0;
  let stBun = 0;
  let stUv = 0;
  let stGo = 0;

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

  const goBad = snap.goProjects.filter((p) => goModHasUpgrades(p.root));
  if (goBad.length > 0) {
    strictNeed(
      `Go — go.mod require(s) newer in: ${goBad.map((p) => formatProjectDirLabel(repoRoot, p.root)).join(", ")}`,
    );
    stGo = 1;
    failed = 1;
  } else if (snap.goProjects.length === 0) {
    strictOk("Go — OK (no Go modules discovered)");
  } else {
    strictOk(`Go — OK (${snap.goProjects.length} module(s))`);
  }

  if (failed === 0) {
    strictAllPassed("All checks passed (nothing reported as outdated).");
  } else {
    console.error("");
    strictSummaryFailTitle("Outdated check failed — upgrades reported in:");
    if (stProto) strictSummaryBullet("proto (.prototools)");
    if (stBun) strictSummaryBullet("Bun workspaces");
    if (stUv) strictSummaryBullet("Python / uv lockfile(s)");
    if (stGo) strictSummaryBullet("Go (go.mod requires)");
    console.error("");
    strictHint("Exit code 1 is intentional (use in CI). To refresh everything: bun run update");
  }

  return failed;
}
