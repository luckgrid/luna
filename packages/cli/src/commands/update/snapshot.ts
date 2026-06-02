import {
  isOutdatedSnapshotStale,
  tryReadOutdatedCacheEntry,
  writeOutdatedCache,
} from "../outdated/cache";
import { gatherOutdatedSnapshotAsync } from "../outdated/gather";
import type { OutdatedSnapshot } from "../outdated/types";

export async function loadOutdatedSnapshot(
  repoRoot: string,
  refreshOutdated: boolean,
): Promise<{ snap: OutdatedSnapshot; fromCache: boolean }> {
  if (!refreshOutdated) {
    const cached = tryReadOutdatedCacheEntry(repoRoot);
    if (cached && !isOutdatedSnapshotStale(cached.writtenAt)) {
      console.log(
        "[luna] using cached outdated snapshot (.cache/outdated-snapshot.json; fingerprint match)",
      );
      return { snap: cached.snap, fromCache: true };
    }
    if (cached && isOutdatedSnapshotStale(cached.writtenAt)) {
      console.log(
        `[luna] outdated snapshot is stale (written ${cached.writtenAt}; ≥12h old); refreshing…`,
      );
    } else {
      console.error(
        "[luna] no valid outdated cache (run `luna outdated` first, or pass `--refresh-outdated`)\n",
      );
    }
  }

  const snap = await gatherOutdatedSnapshotAsync(repoRoot);
  try {
    writeOutdatedCache(repoRoot, snap);
  } catch {
    /* best-effort cache */
  }
  return { snap, fromCache: false };
}
