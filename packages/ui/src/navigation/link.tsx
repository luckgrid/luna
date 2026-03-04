import type { JSX, ParentProps } from "solid-js";
import { splitProps } from "solid-js";

import { isExternalHref, withExternalRel } from "../utils/navigation";

export type LinkProps = ParentProps<JSX.AnchorHTMLAttributes<HTMLAnchorElement>>;

export function Link(props: LinkProps) {
  const [local, rest] = splitProps(props, ["children", "href", "target", "rel"]);
  const external = isExternalHref(local.href);

  return (
    <a
      {...rest}
      href={local.href}
      target={external ? (local.target ?? "_blank") : local.target}
      rel={external ? withExternalRel(local.rel) : local.rel}
    >
      {local.children}
    </a>
  );
}
