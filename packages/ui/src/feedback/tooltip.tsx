import type { JSX, ParentProps } from "solid-js";
import { splitProps } from "solid-js";

export type TooltipProps = ParentProps<
  JSX.HTMLAttributes<HTMLDivElement> & {
    content: string;
    placement?: "top" | "bottom" | "left" | "right";
  }
>;

export function Tooltip(props: TooltipProps) {
  const [local, rest] = splitProps(props, ["content", "placement"]);

  return (
    <div data-tooltip={local.content} data-placement={local.placement} tabindex={0} {...rest} />
  );
}
