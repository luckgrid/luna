import type { RouteMatch } from "@solidjs/router";

import { KNOWN_PATHS } from "~/app.config";

export function normalizePathname(pathname: string) {
  const trimmed = pathname.replace(/\/+$/, "");
  return trimmed || "/";
}

/** Turn a URL path segment into a display label (e.g. `my-page` → `My Page`). */
export function slugToLabel(slug: string) {
  return slug
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export function joinCanonicalUrl(baseUrl: string, pathname: string) {
  const origin = baseUrl.replace(/\/+$/, "");
  const path = pathname.startsWith("/") ? pathname : `/${pathname}`;
  return `${origin}${path === "/" ? "" : path}`;
}

function isCatchAllRoute(match: RouteMatch | undefined) {
  const path = match?.route.originalPath ?? match?.route.pattern ?? "";
  return path.includes("*") || path.includes("[...");
}

export function isNotFoundPath(pathname: string, leaf: RouteMatch | undefined) {
  return isCatchAllRoute(leaf) && !KNOWN_PATHS.has(normalizePathname(pathname));
}
