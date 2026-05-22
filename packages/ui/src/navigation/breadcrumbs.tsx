import { For, Show, splitProps } from "solid-js";

import { Brand, type BrandProps } from "../display/brand";
import { Link } from "./link";

export type BreadcrumbItem = {
  label: string;
  href?: string;
};

export type BreadcrumbsProps = {
  items: BreadcrumbItem[];
  brand?: BrandProps;
  showBrandName?: boolean;
};

export function Breadcrumbs(props: BreadcrumbsProps) {
  const [{ items, brand, showBrandName = false }] = splitProps(props, [
    "items",
    "brand",
    "showBrandName",
  ]);

  return (
    <ol data-breadcrumbs>
      <li>
        <Brand showName={showBrandName} {...brand} />
      </li>
      <For each={items}>
        {(item) => (
          <li aria-current={item.href ? undefined : "page"}>
            <Show when={item.href} fallback={item.label}>
              <Link href={item.href}>{item.label}</Link>
            </Show>
          </li>
        )}
      </For>
    </ol>
  );
}
