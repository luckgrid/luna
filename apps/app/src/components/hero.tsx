import { Show, splitProps } from "solid-js";
import type { JSX, ParentProps } from "solid-js";

export type HeroProps = ParentProps<
  {
    title: string;
    description?: string;
  } & JSX.HTMLAttributes<HTMLElement>
>;

export function Hero(props: HeroProps) {
  const [local, rest] = splitProps(props, ["title", "description", "children"]);

  return (
    <header {...rest} data-hero>
      <hgroup>
        <h1>{local.title}</h1>
        <Show when={local.description}>
          <p>{local.description}</p>
        </Show>
      </hgroup>
      {local.children}
    </header>
  );
}
