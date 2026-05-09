/** Shared helpers: paths, dates, errno, dist I/O, and default pages. */
import { mkdir, copyFile, readdir, stat } from "node:fs/promises";
import path from "node:path";

import type { Post } from "./types";

/** Fallback home page when `src/content/index.md` is missing. */
export function syntheticHome(): Post {
  return {
    section: "",
    slug: "index",
    category: "",
    title: "Luna",
    description: "Luna template static site.",
    date: new Date(0),
    tags: [],
    html: "<p>Add <code>src/content/index.md</code> for home content.</p>",
    layout: "default",
    latestPostsTitle: "Latest posts",
  };
}

/** Fallback posts catalog when `_index.md` is missing but posts exist. */
export function syntheticCatalog(): Post {
  return {
    section: "posts",
    slug: "_index",
    category: "",
    title: "Posts",
    description: "Articles, announcements, and guides.",
    date: new Date(0),
    tags: [],
    html: "",
    layout: "catalog",
  };
}

export function isENOENT(e: unknown): boolean {
  if (e === null || typeof e !== "object") return false;
  return Reflect.get(e, "code") === "ENOENT";
}

export function formatYmd(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function formatDisplayDate(d: Date): string {
  return d.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export async function writePage(distDir: string, rel: string, html: string): Promise<void> {
  const out = path.join(distDir, rel);
  await mkdir(path.dirname(out), { recursive: true });
  await Bun.write(out, html);
}

/** Copy src tree into dst (no-op if src missing). */
/* oxlint-disable eslint/no-await-in-loop -- sequential tree walk */
export async function copyTree(src: string, dst: string): Promise<void> {
  try {
    await stat(src);
  } catch (e) {
    if (isENOENT(e)) return;
    throw e;
  }

  async function walk(cur: string): Promise<void> {
    const entries = await readdir(cur, { withFileTypes: true });
    for (const e of entries) {
      const curPath = path.join(cur, e.name);
      const rel = path.relative(src, curPath);
      const targetPath = path.join(dst, rel);
      if (e.isDirectory()) {
        await mkdir(targetPath, { recursive: true });
        await walk(curPath);
      } else {
        await mkdir(path.dirname(targetPath), { recursive: true });
        await copyFile(curPath, targetPath);
      }
    }
  }

  await mkdir(dst, { recursive: true });
  await walk(src);
}
