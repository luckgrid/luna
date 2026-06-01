import { join } from "node:path";
import {
  formatProjectDirLabel,
  section,
  strictAllPassed,
  strictHint,
  strictNeed,
  strictOk,
  strictSummaryBullet,
  strictSummaryFailTitle,
} from "../../lib/utils";
import { bunWorkspaceOutdatedFromOutput } from "../../lib/bun";
import { goModuleOutdatedHasChanges } from "../../lib/go";
import { protoPinsAnyOutdated } from "../../lib/proto";
import { uvLockHasUpgradesFromOutput } from "../../lib/py";
import { computeOutdatedSummaryState, outdatedTierMessages } from "./summary";
import { printOutdatedTable, printProtoOutdatedTableFromReport } from "./tables";
import type { OutdatedCheckSummaryMode, OutdatedSnapshot } from "./types";

/** Human report: only tiers with something to say print a section. */
export function printOutdatedReport(repoRoot: string, snap: OutdatedSnapshot): void {
  if (protoPinsAnyOutdated(snap.protoReport)) {
    section("proto (.prototools)");
    printProtoOutdatedTableFromReport(repoRoot, snap.protoReport);
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
    if (!goModuleOutdatedHasChanges(g.goGetDryRunOut, g.probe)) continue;
    const label = formatProjectDirLabel(repoRoot, g.root);
    const goSection =
      g.probe === "tool-list"
        ? `Go (${label} — go.mod tools; go list -m -u)`
        : `Go (${label} — go.mod + go.sum; go get -n -u all)`;
    section(goSection);
    printOutdatedTable("go", g.goGetDryRunOut, {
      repoRoot,
      goModPath: join(g.root, "go.mod"),
      goProbe: g.probe,
    });
  }
}

/** CI-style check lines; returns 1 if any tier is outdated, else 0. */
export function printOutdatedCheckSummary(
  repoRoot: string,
  snap: OutdatedSnapshot,
  opts?: { mode?: OutdatedCheckSummaryMode; suppressCiFooter?: boolean },
): number {
  const mode = opts?.mode ?? "full";
  const { failed, stProto, stBun, stUv, stGo } = computeOutdatedSummaryState(snap);

  if (mode === "full") {
    section("check results");
    for (const { ok, text } of outdatedTierMessages(repoRoot, snap)) {
      if (ok) strictOk(text);
      else strictNeed(text);
    }
  }

  if (failed === 0) {
    strictAllPassed("All checks passed (nothing reported as outdated).");
  } else if (!opts?.suppressCiFooter) {
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
