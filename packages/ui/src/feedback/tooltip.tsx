import type { ParentProps } from "solid-js";
import { splitProps } from "solid-js";

import { cx } from "../utils/cx";

export type TooltipProps = ParentProps<{
  content: string;
  placement?: "top" | "bottom" | "left" | "right";
  class?: string;
}>;

export function Tooltip(props: TooltipProps) {
  const [local] = splitProps(props, ["children", "content", "placement", "class"]);

  return (
    <span
      class={cx(local.class)}
      data-tooltip={local.content}
      data-placement={local.placement}
      tabindex={0}
    >
      {local.children}
    </span>
  );
}
