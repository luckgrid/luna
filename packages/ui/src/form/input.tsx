import type { JSX } from "solid-js";
import { createUniqueId, Show, splitProps } from "solid-js";

import { cx } from "../utils/cx";

export type InputProps = JSX.InputHTMLAttributes<HTMLInputElement> & {
  label?: string;
  hint?: string;
  error?: string;
};

export function Input(props: InputProps) {
  const [local, rest] = splitProps(props, ["label", "hint", "error", "class", "id"]);
  const fallbackId = createUniqueId();
  const inputId = () => local.id ?? `input-${fallbackId}`;

  return (
    <label class="flex w-full flex-col gap-1 text-sm text-gray-700" for={inputId()}>
      <Show when={local.label}>
        <span class="font-medium text-gray-900">{local.label}</span>
      </Show>
      <input
        {...rest}
        id={inputId()}
        class={cx(
          "h-10 w-full rounded-md border border-gray-300 bg-white px-3 text-gray-900 shadow-sm",
          "placeholder:text-gray-400 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-200",
          "disabled:cursor-not-allowed disabled:bg-gray-100",
          local.error && "border-red-500 focus:border-red-500 focus:ring-red-200",
          local.class,
        )}
      />
      <Show
        when={local.error}
        fallback={
          <Show when={local.hint}>
            <span class="text-gray-500">{local.hint}</span>
          </Show>
        }
      >
        <span class="text-red-600">{local.error}</span>
      </Show>
    </label>
  );
}
