import { join } from "node:path";
import {
  computeOutdatedSummaryState,
  findRepoRoot,
  isOutdatedLiveStatusEnabled,
  outdatedTierMessages,
  OutdatedLiveStatus,
  printOutdatedTable,
  printProtoOutdatedTableFromReport,
  requireCmd,
  section,
  strictAllPassed,
  strictHint,
  strictNeed,
  strictOk,
  strictSummaryBullet,
  strictSummaryFailTitle,
  tryReadOutdatedCache,
  type StoredOutdatedSnapshot,
  writeOutdatedCache,
} from "../lib/commands";
import { bunWorkspaceOutdatedFromOutput, captureBunOutdatedRecursiveAsync } from "../lib/bun";
import { captureGoModuleOutdatedAsync, goModuleOutdatedHasChanges } from "../lib/go";
import type { GoOutdatedProbe } from "../lib/go";
import { listGoModuleRoots, listUvProjectRoots } from "../lib/moon";
import { captureProtoPinsOutdatedJsonAsync, protoPinsAnyOutdated } from "../lib/proto";
import { captureUvLockDryRunAsync, uvLockHasUpgradesFromOutput } from "../lib/py";
import { formatProjectDirLabel } from "../lib/utils";

export type OutdatedSnapshot = StoredOutdatedSnapshot;

export type UvProjectSnap = { root: string; dryRunOut: string };

export type GoModuleSnap = {
  root: string;
  goGetDryRunOut: string;
  probe?: GoOutdatedProbe;
};

function outdatedProgressTimingsEnabled(): boolean {
  return process.env.LUNA_OUTDATED_PROGRESS === "1";
}

async function runTier<T>(
  tier: "proto" | "bun" | "uv" | "go",
  ok: (v: T) => boolean,
  work: Promise<T>,
  live: OutdatedLiveStatus | null,
): Promise<T> {
  const t0 = Date.now();
  const v = await work;
  const ms = Date.now() - t0;
  live?.markDone(tier, ok(v), ms);
  if (!live && outdatedProgressTimingsEnabled()) {
    console.error(`[luna] ok ${tier} (${ms}ms)`);
  }
  return v;
}

export async function gatherOutdatedSnapshotAsync(
  repoRoot: string,
  live?: OutdatedLiveStatus | null,
): Promise<OutdatedSnapshot> {
  const uvRoots = listUvProjectRoots(repoRoot);
  const goRoots = listGoModuleRoots(repoRoot);

  live?.begin();

  const [protoReport, bunOut, uvProjects, goModules] = await Promise.all([
    runTier(
      "proto",
      (r) => !protoPinsAnyOutdated(r),
      captureProtoPinsOutdatedJsonAsync(repoRoot),
      live ?? null,
    ),
    runTier(
      "bun",
      (o) => !bunWorkspaceOutdatedFromOutput(o),
      captureBunOutdatedRecursiveAsync(repoRoot),
      live ?? null,
    ),
    runTier(
      "uv",
      (rows) => rows.every((p) => !uvLockHasUpgradesFromOutput(p.dryRunOut)),
      Promise.all(
        uvRoots.map((root) =>
          captureUvLockDryRunAsync(root).then((dryRunOut) => ({ root, dryRunOut })),
        ),
      ),
      live ?? null,
    ),
    runTier(
      "go",
      (rows) => rows.every((g) => !goModuleOutdatedHasChanges(g.goGetDryRunOut, g.probe)),
      Promise.all(goRoots.map((root) => captureGoModuleOutdatedAsync(root))),
      live ?? null,
    ),
  ]);

  return { protoReport, bunOut, uvProjects, goModules };
}

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

export type OutdatedCheckSummaryMode = "full" | "rollup";

/** CI-style check lines; returns 1 if any tier is outdated, else 0. */
export function printOutdatedCheckSummary(
  repoRoot: string,
  snap: OutdatedSnapshot,
  opts?: { mode?: OutdatedCheckSummaryMode },
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

function printCachedOutdatedNotice(): void {
  console.error(
    "[luna] outdated: using .cache/outdated-snapshot.json (fingerprint match). Omit --use-cache for a live check (CI-safe).\n",
  );
}

/** Report all tiers, then enforce CI-style exit (1 if any tier has upgrades). */
export async function runOutdated(opts?: { useCache?: boolean }): Promise<number> {
  const repoRoot = findRepoRoot();
  const goRootsPrecheck = listGoModuleRoots(repoRoot);
  requireCmd("proto");
  requireCmd("bun");
  requireCmd("uv");
  if (goRootsPrecheck.length > 0) requireCmd("go");

  if (opts?.useCache) {
    const cached = tryReadOutdatedCache(repoRoot);
    if (cached) {
      printCachedOutdatedNotice();
      const ttyLive = isOutdatedLiveStatusEnabled();
      if (ttyLive) {
        new OutdatedLiveStatus().printSummaryBlock(
          outdatedTierMessages(repoRoot, cached),
          (ok, text) => {
            if (ok) strictOk(text);
            else strictNeed(text);
          },
        );
        printOutdatedReport(repoRoot, cached);
        return printOutdatedCheckSummary(repoRoot, cached, { mode: "rollup" });
      }
      printOutdatedReport(repoRoot, cached);
      return printOutdatedCheckSummary(repoRoot, cached);
    }
    console.error(
      "[luna] outdated: no valid cache (run `luna outdated` without --use-cache first)\n",
    );
  }

  const live = isOutdatedLiveStatusEnabled() ? new OutdatedLiveStatus() : null;
  const snap = await gatherOutdatedSnapshotAsync(repoRoot, live);
  if (live) {
    live.finishAfterGather(outdatedTierMessages(repoRoot, snap), (ok, text) => {
      if (ok) strictOk(text);
      else strictNeed(text);
    });
    printOutdatedReport(repoRoot, snap);
    const failed = printOutdatedCheckSummary(repoRoot, snap, { mode: "rollup" });
    try {
      writeOutdatedCache(repoRoot, snap);
    } catch {
      /* best-effort cache */
    }
    return failed;
  }

  printOutdatedReport(repoRoot, snap);
  const failed = printOutdatedCheckSummary(repoRoot, snap);
  try {
    writeOutdatedCache(repoRoot, snap);
  } catch {
    /* best-effort cache */
  }
  return failed;
}
