import type { BreadcrumbItem } from "@luna/ui/breadcrumbs";
import { Link, type LinkProps } from "@luna/ui/link";
import { useCurrentMatches, useHref, useLocation, useResolvedPath } from "@solidjs/router";
import type { ParentProps } from "solid-js";
import { createMemo, splitProps } from "solid-js";

import { ROUTE_SEO } from "~/app.config";
import { isNotFoundPath, normalizePathname, slugToLabel } from "~/utils/url";

export type NavLinkProps = ParentProps<
  LinkProps & {
    end?: boolean;
  }
>;

function normalizePath(path: string) {
  const trimmed = path.replace(/^\/+|(\/)\/+$/g, "$1");
  return trimmed ? (/^[?#]/.test(trimmed) ? trimmed : `/${trimmed}`) : "";
}

export function NavLink(props: NavLinkProps) {
  const [local, rest] = splitProps(props, ["href", "end", "children"]);
  const to = useResolvedPath(() => local.href ?? "");
  const href = useHref(to);
  const location = useLocation();
  const isActive = createMemo(() => {
    const target = to();
    if (target === undefined) {
      return false;
    }

    const path = normalizePath(target.split(/[?#]/, 1)[0] ?? "").toLowerCase();
    const current = decodeURI(normalizePath(location.pathname).toLowerCase());

    return local.end ? path === current : current.startsWith(`${path}/`) || current === path;
  });

  const active = isActive();

  return (
    <Link
      {...rest}
      href={href() ?? local.href}
      link
      aria-current={active ? "page" : undefined}
      data-active={active ? true : undefined}
    >
      {local.children}
    </Link>
  );
}

export type NavigationProps = ParentProps<{
  label: string;
  hideLinks?: boolean;
}>;

function breadcrumbLabel(segment: string, pathSoFar: string) {
  return ROUTE_SEO[pathSoFar]?.title ?? slugToLabel(segment);
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
      label: breadcrumbLabel(segment, path),
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
    if (isNotFoundPath(pathname, leaf)) {
      return [{ label: "Not Found" }];
    }

    return itemsFromPathname(pathname);
  });
}

export function Navigation(props: NavigationProps) {
  const [{ children, label, hideLinks }] = splitProps(props, ["children", "label", "hideLinks"]);

  return (
    <nav aria-label={label}>
      {!hideLinks && (
        <ul>
          <li>
            <NavLink href="/ds">DS</NavLink>
          </li>
          <li>
            <NavLink href="/ui">UI</NavLink>
          </li>
          <li>
            <NavLink href="/ai">AI</NavLink>
          </li>
        </ul>
      )}
      {children}
    </nav>
  );
}
