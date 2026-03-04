import type { JSX, ParentProps } from "solid-js";
import { splitProps } from "solid-js";

import { cx } from "../utils/cx";

export type LinkProps = ParentProps<JSX.AnchorHTMLAttributes<HTMLAnchorElement>>;

export function Link(props: LinkProps) {
  const [local, rest] = splitProps(props, ["children", "class"]);

  return (
    <a
      {...rest}
      class={cx(
        "font-medium text-blue-700 underline decoration-blue-300 underline-offset-4 transition-colors",
        "hover:text-blue-600 hover:decoration-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-300",
        local.class,
      )}
    >
      {local.children}
    </a>
  );
}
