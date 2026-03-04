import type { JSX, ParentProps } from "solid-js";
import { splitProps } from "solid-js";

import { cx } from "../utils/cx";

export type TooltipProps = ParentProps<{
  content: JSX.Element;
  class?: string;
  contentClass?: string;
}>;

export function Tooltip(props: TooltipProps) {
  const [local] = splitProps(props, ["children", "content", "class", "contentClass"]);

  return (
    <span class={cx("group relative inline-flex", local.class)}>
      <span>{local.children}</span>
      <span
        role="tooltip"
        class={cx(
          "pointer-events-none absolute bottom-full left-0 z-10 mb-2 w-max max-w-xs",
          "rounded-md bg-gray-900 px-2 py-1 text-xs text-white opacity-0 shadow transition-opacity",
          "group-hover:opacity-100 group-focus-within:opacity-100",
          local.contentClass,
        )}
      >
        {local.content}
      </span>
    </span>
  );
}
