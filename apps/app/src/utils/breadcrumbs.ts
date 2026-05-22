import { useCurrentMatches, useLocation } from "@solidjs/router";
import type { RouteMatch } from "@solidjs/router";
import { createMemo } from "solid-js";

import type { BreadcrumbItem } from "@luna/ui/breadcrumbs";

/** Slugs that should not use default title-casing (e.g. `ai` → `AI`). */
const SLUG_LABELS: Record<string, string> = {
  ai: "AI",
  ui: "UI",
  ds: "DS",
};

const KNOWN_ROOTS = new Set(Object.keys(SLUG_LABELS));

export function normalizePathname(pathname: string) {
  const trimmed = pathname.replace(/\/+$/, "");
  return trimmed || "/";
}

function slugToLabel(slug: string) {
  return SLUG_LABELS[slug] ?? titleCase(slug);
}

function titleCase(slug: string) {
  return slug
    .replace(/[-,&]/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function isCatchAllRoute(match: RouteMatch | undefined) {
  const path = match?.route.originalPath ?? match?.route.pattern ?? "";
  return path.includes("*") || path.includes("[...");
}

function isKnownRoute(pathname: string) {
  const segments = normalizePathname(pathname).split("/").filter(Boolean);
  if (segments.length === 0) {
    return false;
  }

  return KNOWN_ROOTS.has(segments[0]);
}

function itemsFromPathname(pathname: string): BreadcrumbItem[] {
  const segments = normalizePathname(pathname).split("/").filter(Boolean);
  if (segments.length === 0) {
    return [];
  }

  let path = "";
  return segments.map((segment, index) => {
    path += `/${segment}`;
    const isLast = index === segments.length - 1;

    return {
      label: slugToLabel(segment),
      href: isLast ? undefined : path,
    };
  });
}

/**
 * Breadcrumb trail from the current URL path (Solid Router `useLocation`), with
 * `useCurrentMatches` used only to detect the catch-all 404 route.
 */
export function useBreadcrumbs() {
  const location = useLocation();
  const matches = useCurrentMatches();

  return createMemo(() => {
    const pathname = normalizePathname(location.pathname);
    if (pathname === "/") {
      return [];
    }

    const leaf = matches().at(-1);
    if (isCatchAllRoute(leaf) && !isKnownRoute(pathname)) {
      return [{ label: "Not Found" }];
    }

    return itemsFromPathname(pathname);
  });
}
