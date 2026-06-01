import type { JSX, ParentProps } from "solid-js";

import { Link } from "../navigation/link";

export type ButtonProps = ParentProps<JSX.ButtonHTMLAttributes<HTMLButtonElement>>;

export type ButtonLinkProps = ParentProps<
  Omit<JSX.AnchorHTMLAttributes<HTMLAnchorElement>, "role">
>;

export function Button(props: ButtonProps) {
  return <button type="button" {...props} />;
}

export function GhostButton(props: ButtonProps) {
  return <Button {...props} data-button="ghost" />;
}

export function HollowButton(props: ButtonProps) {
  return <Button {...props} data-button="hollow" />;
}

export function IconButton(props: ButtonProps) {
  return <Button {...props} data-button="icon" />;
}

export function TextButton(props: ButtonProps) {
  return <Button {...props} data-button="text" />;
}

export function WrapperButton(props: ButtonProps) {
  return <Button {...props} data-button="wrapper" />;
}

export function ButtonLink(props: ButtonLinkProps) {
  return <Link role="button" {...props} />;
}

export function GhostButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} data-button="ghost" />;
}

export function HollowButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} data-button="hollow" />;
}

export function IconButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} data-button="icon" />;
}

export function TextButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} data-button="text" />;
}

export function WrapperButtonLink(props: ButtonLinkProps) {
  return <ButtonLink {...props} data-button="wrapper" />;
}
