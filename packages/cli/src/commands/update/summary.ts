import { formatProjectDirLabel, section, strictOk } from "../../lib/utils";
import { computeOutdatedSummaryState } from "../outdated/summary";
import type { OutdatedSnapshot } from "../outdated/types";
import { computeUpdatePlan, type UpdatePlan } from "./plan";

export function printUpdatePlanSummary(
  repoRoot: string,
  snap: OutdatedSnapshot,
  major: boolean,
): UpdatePlan {
  const plan = computeUpdatePlan(snap, major);
  const { stProto, stBun, stUv, stGo } = computeOutdatedSummaryState(snap);

  section("update plan");
  if (plan.proto) {
    strictOk("proto — will update pin(s) in .prototools");
  } else if (stProto) {
    strictOk("proto — skip (outdated only beyond manifest; use --major)");
  } else {
    strictOk("proto — skip (up to date)");
  }

  if (plan.bun) {
    strictOk("Bun — will bump workspace dependencies");
  } else if (stBun) {
    strictOk(
      major
        ? "Bun — skip (no actionable workspace bumps)"
        : "Bun — skip (only major bumps available; use --major)",
    );
  } else {
    strictOk("Bun — skip (up to date)");
  }

  const uvStale = snap.uvProjects.filter((p) => plan.uvRoots.includes(p.root));
  if (plan.uvRoots.length > 0) {
    strictOk(
      `Python / uv — will update: ${uvStale.map((p) => formatProjectDirLabel(repoRoot, p.root)).join(", ")}`,
    );
  } else if (stUv) {
    strictOk("Python / uv — skip (no lockfile upgrades in dry-run)");
  } else if (snap.uvProjects.length === 0) {
    strictOk("Python / uv — skip (no uv projects discovered)");
  } else {
    strictOk(`Python / uv — skip (up to date, ${snap.uvProjects.length} project(s))`);
  }

  const goStale = snap.goModules.filter((g) => plan.goModules.includes(g.root));
  if (plan.goModules.length > 0) {
    strictOk(
      `Go — will update: ${goStale.map((g) => formatProjectDirLabel(repoRoot, g.root)).join(", ")}`,
    );
  } else if (stGo) {
    strictOk(
      major
        ? "Go — skip (no actionable module bumps)"
        : "Go — skip (only non-patch upgrades on tool-only modules; use --major)",
    );
  } else if (snap.goModules.length === 0) {
    strictOk("Go — skip (no go.mod projects discovered)");
  } else {
    strictOk(`Go — skip (up to date, ${snap.goModules.length} module(s))`);
  }

  return plan;
}
