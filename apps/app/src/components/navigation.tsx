import { Link, type LinkProps } from "@luna/ui/link";
import { useHref, useLocation, useResolvedPath } from "@solidjs/router";
import type { ParentProps } from "solid-js";
import { createMemo, splitProps } from "solid-js";

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
