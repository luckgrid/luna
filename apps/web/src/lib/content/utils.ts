import path from "node:path";

import type { Post } from "../types";

export function postURL(p: Post): string {
  if (p.section === "") return `/${p.slug}.html`;
  return `/${p.section}/${p.slug}/`;
}

export function outputPath(p: Post): string {
  if (p.section === "") return `${p.slug}.html`;
  return path.join(p.section, p.slug, "index.html");
}

export function caption(p: Post): string {
  if (p.category !== "") return p.category;
  return p.section;
}

export function contentDir(root: string): string {
  return path.join(root, "src", "content");
}
