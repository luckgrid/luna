import type { JSX } from "solid-js";
import { splitProps } from "solid-js";

export type InputProps = JSX.InputHTMLAttributes<HTMLInputElement>;

export function Input(props: InputProps) {
  const [local, rest] = splitProps(props, ["type"]);

  return <input type={local.type ?? "text"} data-slot="input" {...rest} />;
}
