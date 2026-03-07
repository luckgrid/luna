import type { JSX, ParentProps } from "solid-js";

import { Link } from "../navigation/link";
import { cx } from "../utils/cx";

export type ButtonProps = ParentProps<JSX.ButtonHTMLAttributes<HTMLButtonElement>>;

export type ButtonLinkProps = ParentProps<
  Omit<JSX.AnchorHTMLAttributes<HTMLAnchorElement>, "role">
>;

export function Button(props: ButtonProps) {
  return <button type="button" {...props} />;
}

export function PrimaryButton(props: ButtonProps) {
  return <Button {...props} class={cx("button-primary", props.class)} />;
}

export function SecondaryButton(props: ButtonProps) {
  return <Button {...props} class={cx("button-secondary", props.class)} />;
}

export function GhostButton(props: ButtonProps) {
  return <Button {...props} class={cx("button-ghost", props.class)} />;
}

export function OutlineButton(props: ButtonProps) {
  return <Button {...props} class={cx("button-outline", props.class)} />;
}

export function TextButton(props: ButtonProps) {
  return <Button {...props} class={cx("button-text", props.class)} />;
}

export function ButtonLink(props: ButtonLinkProps) {
  return <Link role="button" {...props} />;
}

export function PrimaryButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} class={cx("button-primary", props.class)} />;
}

export function SecondaryButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} class={cx("button-secondary", props.class)} />;
}

export function GhostButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} class={cx("button-ghost", props.class)} />;
}

export function OutlineButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} class={cx("button-outline", props.class)} />;
}

export function TextButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} class={cx("button-text", props.class)} />;
}
