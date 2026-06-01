import { listGoModuleRoots } from "../../lib/moon";
import { findRepoRoot, requireCmd, strictNeed, strictOk } from "../../lib/utils";
import { tryReadOutdatedCache, writeOutdatedCache } from "./cache";
import { gatherOutdatedSnapshotAsync } from "./gather";
import { OutdatedLiveStatus, isOutdatedLiveStatusEnabled } from "./live";
import { printOutdatedCheckSummary, printOutdatedReport } from "./report";
import { outdatedTierMessages } from "./summary";

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
