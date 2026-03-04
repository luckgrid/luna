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
    <div
      class={cx("w-full divide-y divide-gray-200 rounded-lg border border-gray-200", local.class)}
    >
      <For each={local.items}>
        {(item) => (
          <details class="p-4" open={item.open}>
            <summary class="flex cursor-pointer list-none items-center justify-between gap-3 font-medium text-gray-900">
              <span>{item.title}</span>
              <span class="text-gray-500">+</span>
            </summary>
            <Show when={item.content}>
              <div class="pt-3 text-sm leading-6 text-gray-700">{item.content}</div>
            </Show>
          </details>
        )}
      </For>
    </div>
  );
}
