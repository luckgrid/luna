import { bunWorkspaceOutdatedFromOutput } from "../../lib/bun";
import { goModuleOutdatedHasChanges } from "../../lib/go";
import { protoPinsAnyOutdated } from "../../lib/proto";
import { uvLockHasUpgradesFromOutput } from "../../lib/py";
import { formatProjectDirLabel } from "../../lib/utils";
import type { OutdatedSummaryState, StoredOutdatedSnapshot } from "./types";

export function computeOutdatedSummaryState(snap: StoredOutdatedSnapshot): OutdatedSummaryState {
  let stProto = 0;
  let stBun = 0;
  let stUv = 0;
  let stGo = 0;

  if (protoPinsAnyOutdated(snap.protoReport)) stProto = 1;
  if (bunWorkspaceOutdatedFromOutput(snap.bunOut)) stBun = 1;
  const uvBad = snap.uvProjects.filter((p) => uvLockHasUpgradesFromOutput(p.dryRunOut));
  if (uvBad.length > 0) stUv = 1;
  const goBad = snap.goModules.filter((g) => goModuleOutdatedHasChanges(g.goGetDryRunOut, g.probe));
  if (goBad.length > 0) stGo = 1;

  const failed = stProto | stBun | stUv | stGo;
  return { failed, stProto, stBun, stUv, stGo };
}

/** One line per tier (same wording as `printOutdatedCheckSummary` full mode). */
export function outdatedTierMessages(
  repoRoot: string,
  snap: StoredOutdatedSnapshot,
): { ok: boolean; text: string }[] {
  const { stProto, stBun, stUv, stGo } = computeOutdatedSummaryState(snap);
  const uvBad = snap.uvProjects.filter((p) => uvLockHasUpgradesFromOutput(p.dryRunOut));
  const goBad = snap.goModules.filter((g) => goModuleOutdatedHasChanges(g.goGetDryRunOut, g.probe));

  const lines: { ok: boolean; text: string }[] = [];
  lines.push(
    stProto
      ? { ok: false, text: "proto — outdated pin(s) in .prototools" }
      : { ok: true, text: "proto — OK (.prototools)" },
  );
  lines.push(
    stBun
      ? { ok: false, text: "Bun — outdated direct dependencies in workspaces" }
      : { ok: true, text: "Bun — OK (workspaces)" },
  );
  if (stUv) {
    lines.push({
      ok: false,
      text: `Python / uv — lockfile(s) can update: ${uvBad.map((p) => formatProjectDirLabel(repoRoot, p.root)).join(", ")} (see luna update)`,
    });
  } else if (snap.uvProjects.length === 0) {
    lines.push({ ok: true, text: "Python / uv — OK (no uv projects discovered)" });
  } else {
    lines.push({ ok: true, text: `Python / uv — OK (${snap.uvProjects.length} project(s))` });
  }
  if (stGo) {
    lines.push({
      ok: false,
      text: `Go — go.mod can advance: ${goBad.map((g) => formatProjectDirLabel(repoRoot, g.root)).join(", ")} (see luna update)`,
    });
  } else if (snap.goModules.length === 0) {
    lines.push({ ok: true, text: "Go — OK (no go.mod projects discovered)" });
  } else {
    lines.push({ ok: true, text: `Go — OK (${snap.goModules.length} module(s))` });
  }
  return lines;
}
