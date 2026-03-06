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

export type AccordionItemProps = ParentProps<
  JSX.HTMLAttributes<HTMLDetailsElement> & {
    open?: boolean;
  }
>;

export type AccordionTriggerProps = ParentProps<JSX.HTMLAttributes<HTMLElement>>;

export type AccordionContentProps = ParentProps<JSX.HTMLAttributes<HTMLDivElement>>;

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

export function AccordionItem(props: AccordionItemProps) {
  return <details data-slot="accordion-item" {...props} />;
}

export function AccordionTrigger(props: AccordionTriggerProps) {
  return <summary data-slot="accordion-trigger" {...props} />;
}

export function AccordionContent(props: AccordionContentProps) {
  return <div data-content data-slot="accordion-content" {...props} />;
}
