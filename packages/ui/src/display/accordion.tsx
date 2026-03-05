import type { JSX } from "solid-js";
import { For, Show, splitProps } from "solid-js";

import { cx } from "../utils/cx";

export type AccordionItem = {
  title: string;
  content: JSX.Element;
  open?: boolean;
};

export type AccordionProps = {
  items: AccordionItem[];
  class?: string;
};

export function Accordion(props: AccordionProps) {
  const [local] = splitProps(props, ["items", "class"]);

  return (
    <div class={cx("stack", local.class)}>
      <For each={local.items}>
        {(item) => (
          <details open={item.open}>
            <summary>{item.title}</summary>
            <Show when={item.content}>
              <div data-content>{item.content}</div>
            </Show>
          </details>
        )}
      </For>
    </div>
  );
}
