import { tryReadOutdatedCache, writeOutdatedCache } from "../outdated/cache";
import { gatherOutdatedSnapshotAsync } from "../outdated/gather";
import type { OutdatedSnapshot } from "../outdated/types";

export async function loadOutdatedSnapshot(
  repoRoot: string,
  refreshOutdated: boolean,
): Promise<{ snap: OutdatedSnapshot; fromCache: boolean }> {
  if (!refreshOutdated) {
    const cached = tryReadOutdatedCache(repoRoot);
    if (cached) {
      console.log(
        "[luna] using cached outdated snapshot (.cache/outdated-snapshot.json; fingerprint match)",
      );
      return { snap: cached, fromCache: true };
    }
    console.error(
      "[luna] no valid outdated cache (run `luna outdated` first, or pass `--refresh-outdated`)\n",
    );
  }

  const snap = await gatherOutdatedSnapshotAsync(repoRoot);
  try {
    writeOutdatedCache(repoRoot, snap);
  } catch {
    /* best-effort cache */
  }
  return { snap, fromCache: false };
}
