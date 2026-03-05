import type { JSX, ParentProps } from "solid-js";
import { For, Show, splitProps } from "solid-js";

export type AccordionEntry = {
  title: string;
  content: JSX.Element;
  open?: boolean;
};

export type AccordionProps = ParentProps<
  Omit<JSX.HTMLAttributes<HTMLDivElement>, "role"> & {
    items?: AccordionEntry[];
  }
>;

export function Accordion(props: AccordionProps) {
  const [local, rest] = splitProps(props, ["items", "children"]);

  return (
    <div role="group" {...rest}>
      <For each={local.items ?? []}>
        {(item) => (
          <AccordionItem open={item.open}>
            <AccordionTrigger>{item.title}</AccordionTrigger>
            <Show when={item.content}>
              <AccordionContent>{item.content}</AccordionContent>
            </Show>
          </AccordionItem>
        )}
      </For>
      {local.children}
    </div>
  );
}

export type AccordionItemProps = ParentProps<
  JSX.HTMLAttributes<HTMLDetailsElement> & {
    open?: boolean;
  }
>;

export function AccordionItem(props: AccordionItemProps) {
  const [local, rest] = splitProps(props, ["children"]);

  return (
    <details data-slot="accordion-item" {...rest}>
      {local.children}
    </details>
  );
}

export type AccordionTriggerProps = ParentProps<JSX.HTMLAttributes<HTMLElement>>;

export function AccordionTrigger(props: AccordionTriggerProps) {
  const [local, rest] = splitProps(props, ["children"]);

  return (
    <summary data-slot="accordion-trigger" {...rest}>
      {local.children}
    </summary>
  );
}

export type AccordionContentProps = ParentProps<JSX.HTMLAttributes<HTMLDivElement>>;

export function AccordionContent(props: AccordionContentProps) {
  const [local, rest] = splitProps(props, ["children"]);

  return (
    <div data-content data-slot="accordion-content" {...rest}>
      {local.children}
    </div>
  );
}
