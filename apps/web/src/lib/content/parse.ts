/**
 * Load and parse `src/content` Markdown: frontmatter, taxonomy, catalog paths.
 */
import matter from "gray-matter";
import type { Dirent } from "node:fs";
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

import type { Post } from "../types";
import { isENOENT } from "../utils";

interface FrontmatterShape {
  title?: string;
  slug?: string;
  category?: string;
  description?: string;
  date?: string;
  tags?: unknown;
  layout?: string;
  latest_posts_title?: string;
}

/** Markdown body → HTML via Bun (https://bun.com/docs/runtime/markdown). */
export function markdownToHtml(markdown: string): string {
  return Bun.markdown.html(markdown, {
    tables: true,
    strikethrough: true,
    tasklists: true,
    autolinks: true,
    headings: { ids: true },
  });
}

function parseDate(v: unknown): Date {
  if (v instanceof Date && !Number.isNaN(v.getTime())) return v;
  if (typeof v === "string") {
    const t = Date.parse(v);
    if (!Number.isNaN(t)) return new Date(t);
    const ymd = /^(\d{4})-(\d{2})-(\d{2})$/.exec(v.trim());
    if (ymd) return new Date(Number(ymd[1]), Number(ymd[2]) - 1, Number(ymd[3]));
  }
  return new Date(0);
}

function parseTags(v: unknown): string[] {
  if (!Array.isArray(v)) return [];
  return v.filter((t): t is string => typeof t === "string");
}

/** One Markdown file → Post (body HTML from Bun.markdown). */
export async function parseFile(section: string, filePath: string): Promise<Post> {
  const raw = await readFile(filePath, "utf8");
  const { data, content: body } = matter(raw);
  const fm = data as FrontmatterShape;

  let slug = path.basename(filePath, path.extname(filePath));
  if (typeof fm.slug === "string" && fm.slug !== "") slug = fm.slug;

  const title = typeof fm.title === "string" && fm.title !== "" ? fm.title : slug;
  const category = typeof fm.category === "string" ? fm.category : "";
  const description = typeof fm.description === "string" ? fm.description : "";
  const date = parseDate(fm.date);
  const tags = parseTags(fm.tags);
  const layout = typeof fm.layout === "string" ? fm.layout : undefined;
  const latestPostsTitle =
    typeof fm.latest_posts_title === "string" ? fm.latest_posts_title : undefined;

  return {
    section,
    slug,
    category,
    title,
    description,
    date,
    tags,
    html: markdownToHtml(body),
    layout,
    latestPostsTitle,
  };
}

/** Top-level `.md` in dir (non-recursive), sorted newest-first. */
export async function parseDir(section: string, dir: string): Promise<Post[]> {
  let entries: Dirent[];
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch (e) {
    if (isENOENT(e)) return [];
    throw e;
  }

  const mdEntries = entries.filter((e) => !e.isDirectory() && e.name.endsWith(".md"));
  const posts = await Promise.all(mdEntries.map((e) => parseFile(section, path.join(dir, e.name))));
  sortNewestFirst(posts);
  return posts;
}

/** `src/content/*.md` at project root; home is `index.md` (slug `index`). */
export async function parseTopLevelPages(
  dir: string,
): Promise<{ home: Post | null; pages: Post[] }> {
  const all = await parseDir("", dir);
  return {
    home: all.find((p) => p.slug === "index") ?? null,
    pages: all.filter((p) => p.slug !== "index"),
  };
}

const CATALOG_INDEX = "_index.md";

/** `src/content/posts/*.md` except reserved `_index.md` (catalog landing). */
export async function parsePostsFolder(
  dir: string,
): Promise<{ posts: Post[]; catalog: Post | null }> {
  let entries: Dirent[];
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch (e) {
    if (isENOENT(e)) return { posts: [], catalog: null };
    throw e;
  }

  const catalogPath = path.join(dir, CATALOG_INDEX);
  let catalog: Post | null = null;
  try {
    await stat(catalogPath);
    catalog = await parseFile("posts", catalogPath);
  } catch {
    /* no catalog landing */
  }

  const files: string[] = [];
  for (const e of entries) {
    if (e.isDirectory() || !e.name.endsWith(".md")) continue;
    if (e.name === CATALOG_INDEX) continue;
    files.push(e.name);
  }

  const posts = await Promise.all(files.map((n) => parseFile("posts", path.join(dir, n))));
  sortNewestFirst(posts);
  return { posts, catalog };
}

export function sortNewestFirst(posts: Post[]): void {
  posts.sort((a, b) => {
    if (a.date.getTime() !== b.date.getTime()) return b.date.getTime() - a.date.getTime();
    return a.title.localeCompare(b.title);
  });
}

export function groupByCategory(posts: Post[]): Record<string, Post[]> {
  const out: Record<string, Post[]> = {};
  for (const p of posts) {
    const k = p.category;
    if (!out[k]) out[k] = [];
    out[k].push(p);
  }
  return out;
}

/** Non-empty category labels, sorted (for catalog grouping). */
export function categories(posts: Post[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const p of posts) {
    if (p.category === "" || seen.has(p.category)) continue;
    seen.add(p.category);
    out.push(p.category);
  }
  return out.toSorted();
}
