import { Breadcrumbs } from "@luna/ui/breadcrumbs";
import { useLocation } from "@solidjs/router";
import { createMemo, Show } from "solid-js";

import { normalizePathname } from "~/utils/url";
import { Navigation, useBreadcrumbs } from "./navigation";

export function Header() {
  const location = useLocation();
  const items = useBreadcrumbs();
  const showHeader = createMemo(
    () => normalizePathname(location.pathname) !== "/" && items().length > 0,
  );

  return (
    <Show when={showHeader()}>
      <header data-sticky>
        <Navigation label="Header navigation" hideLinks>
          <Breadcrumbs items={items()} />
        </Navigation>
      </header>
    </Show>
  );
}
