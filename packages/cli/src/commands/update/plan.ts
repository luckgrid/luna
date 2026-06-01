import { bunWorkspaceHasActionableUpdates } from "../../lib/bun";
import { goModuleHasActionableUpdates } from "../../lib/go";
import { protoPinsHasActionableUpdates } from "../../lib/proto";
import { uvLockHasUpgradesFromOutput } from "../../lib/py";
import type { StoredOutdatedSnapshot } from "../outdated/types";

export type UpdatePlan = {
  proto: boolean;
  bun: boolean;
  uvRoots: string[];
  goModules: string[];
};

export function computeUpdatePlan(snap: StoredOutdatedSnapshot, major: boolean): UpdatePlan {
  const proto = protoPinsHasActionableUpdates(snap.protoReport, major);
  const bun = bunWorkspaceHasActionableUpdates(snap.bunOut, major);
  const uvRoots = snap.uvProjects
    .filter((p) => uvLockHasUpgradesFromOutput(p.dryRunOut))
    .map((p) => p.root);
  const goModules = snap.goModules
    .filter((g) => goModuleHasActionableUpdates(g.goGetDryRunOut, g.probe, major))
    .map((g) => g.root);
  return { proto, bun, uvRoots, goModules };
}

export function updatePlanHasWork(plan: UpdatePlan): boolean {
  return plan.proto || plan.bun || plan.uvRoots.length > 0 || plan.goModules.length > 0;
}
