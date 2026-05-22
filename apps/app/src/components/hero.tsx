import { Show } from "solid-js";
import type { ParentProps } from "solid-js";

export type HeroProps = ParentProps<{
  title: string;
  description?: string;
}>;

export function Hero(props: HeroProps) {
  return (
    <header data-hero>
      <hgroup>
        <h1>{props.title}</h1>
        <Show when={props.description}>
          <p>{props.description}</p>
        </Show>
      </hgroup>
      {props.children}
    </header>
  );
}
