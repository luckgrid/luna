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
    <label class={cx("field", local.error && "error")} for={inputId()}>
      <Show when={local.label}>
        <span>{local.label}</span>
      </Show>
      <input {...rest} id={inputId()} class={cx(local.class)} />
      <Show
        when={local.error}
        fallback={
          <Show when={local.hint}>
            <small data-hint>{local.hint}</small>
          </Show>
        }
      >
        <small data-error>{local.error}</small>
      </Show>
    </label>
  );
}
