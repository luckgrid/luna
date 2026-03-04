import type { JSX, ParentProps } from "solid-js";
import { splitProps } from "solid-js";

export type ButtonProps = ParentProps<JSX.ButtonHTMLAttributes<HTMLButtonElement>>;

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["children", "type"]);

  return (
    <button {...rest} type={local.type ?? "button"}>
      {local.children}
    </button>
  );
}

export type ButtonLinkProps = ParentProps<
  Omit<JSX.AnchorHTMLAttributes<HTMLAnchorElement>, "role">
>;

export function ButtonLink(props: ButtonLinkProps) {
  const [local, rest] = splitProps(props, ["children"]);

  return (
    <a role="button" {...rest}>
      {local.children}
    </a>
  );
}
