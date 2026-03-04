import { splitProps } from "solid-js";
import type { JSX, ParentProps } from "solid-js";

import { cx } from "~/utils/cx";

export type ButtonProps = ParentProps<JSX.ButtonHTMLAttributes<HTMLButtonElement>>;

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["children", "class", "type"]);

  return (
    <button
      {...rest}
      type={local.type ?? "button"}
      class={cx(
        "inline-flex h-10 items-center justify-center rounded-md bg-blue-600 px-4 text-sm font-medium text-white transition-colors",
        "hover:bg-blue-500",
        "focus:outline-none focus:ring-2 focus:ring-blue-300",
        "disabled:cursor-not-allowed disabled:opacity-60",
        local.class,
      )}
    >
      {local.children}
    </button>
  );
}
