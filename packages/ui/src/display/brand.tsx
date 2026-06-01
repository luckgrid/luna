import { Show, splitProps } from "solid-js";

export type BrandProps = {
  href?: string;
  name?: string;
  showName?: boolean;
  src?: string;
};

export function Brand(props: BrandProps) {
  const [{ href = "/", name = "Luna", showName = false, src = "/favicon.ico" }] = splitProps(
    props,
    ["href", "name", "showName", "src"],
  );

  return (
    <a href={href} data-brand aria-label={name}>
      <img src={src} alt="" width="24" height="24" />
      <Show when={showName}>
        <span>{name}</span>
      </Show>
    </a>
  );
}
