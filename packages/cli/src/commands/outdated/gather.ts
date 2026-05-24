import { bunWorkspaceOutdatedFromOutput, captureBunOutdatedRecursiveAsync } from "../../lib/bun";
import { captureGoModuleOutdatedAsync, goModuleOutdatedHasChanges } from "../../lib/go";
import { listGoModuleRoots, listUvProjectRoots } from "../../lib/moon";
import { captureProtoPinsOutdatedJsonAsync, protoPinsAnyOutdated } from "../../lib/proto";
import { captureUvLockDryRunAsync, uvLockHasUpgradesFromOutput } from "../../lib/py";
import type { OutdatedLiveStatus } from "./live";
import type { OutdatedSnapshot } from "./types";

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
