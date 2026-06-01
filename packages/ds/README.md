# `@luna/ds`

Shared, CSS-first design-system styles for Luna apps.

## Stylesheet import

Import the DS Tailwind-backed stylesheet directly from your app CSS:

```css
@import "@luna/ds/tailwind.css";
```

If needed, app-specific overrides can still live in app-level CSS after this import.

## Vite config export (optional)

`@luna/ds` exports a Vite config object with shared Tailwind and Lightning CSS defaults:

```ts
import dsConfig from "@luna/ds/vite.config";
```

Merge it into your app `vite.config.ts` with `mergeConfig`.

## Architecture

The DS is authored as CSS-first Tailwind v4 with explicit layers, custom variants/utilities, and a single `@theme inline` source (`src/theme.css`).
See `docs/design-system/` in the repository for roadmap, architecture, tokens, functions, and patterns.

## Scoped layers pattern (all modules)

CSS modules in `src/{layouts,primitives,components}/*.css` follow a **scoped root + nested layers** pattern. The goal is to keep selectors readable (scope first), then keep cascade intent explicit (layers inside that scope).

- **Root namespace first**: when a module targets a specific subtree, start with a single root `@scope` that establishes the boundary (often a `data-*` attribute), e.g. `@scope ([data-layout=\"article\"]) { … }` or `@scope ([data-component=\"tooltip\"]) { … }`.
- **Layers live inside the root scope**: most modules use `@layer base`, `@layer variants`, and (occasionally) `@layer patterns` inside the scope.
- **Layouts are the exception**: layout modules use a nested `@layer components` (instead of `variants`) to group layout “chrome” that composes base structure; `patterns` is still reserved for special cases.
- **Pattern selectors first (inside the layer)**: when adding a page-level pattern, select the pattern attribute before narrowing the subtree, e.g. `&[data-pattern=\"post\"] { … }`.
- **Scoped subtrees for containers**: use nested `@scope` blocks to “enter” major semantic containers (e.g. `main > aside`, `main > article`), generally no more than 1–2 levels deep.
- **Prefer `:scope` for root styling**: inside a scope, use `:scope { … }` (or `:scope > …`) to make it obvious you’re styling the scope root; use nested selectors for descendants.
- **Follow semantic render order**: keep blocks ordered the way elements appear (header → main → aside → article) so related styles remain contiguous and discoverable.

### Complex modules (components with deep subtrees)

When a component module starts to accumulate nested selectors, treat the module like a mini layout: add **named nested scopes** for the “subtrees that matter” and keep state close to the block it modifies.

- **Use nested scopes to “enter” a complex subtree**: wrap the subtree in `@scope (...) { … }`, then style the subtree root via `:scope { … }`.
  - **Example**: Accordion items: `@scope (:scope :is(details, …)) { :scope { … } }`
  - **Example**: Breadcrumb items: `@scope (:scope > :is(li, a, …)) { :scope { … } }`
- **Keep modifier/state rules adjacent to the base styles they modify** (same subtree, same layer) instead of collecting them at the bottom.
  - **Example**: put `:scope:has(:hover, :focus-visible)` near the item shell, not far away.
  - **Example**: if “open” behavior is mostly about the trigger, keep it in the trigger scope using parent selectors like `:is([open], [data-open]) &`.
- **Don’t create a nested scope for simple one-off descendants**: if a rule is a single selector with 1–2 declarations, keep it inline (e.g. `:scope :is([data-content], [data-slot]) { … }`).
- **Prefer nested selectors inside `:scope { … }` for cohesion**: place `&:is(...)` and `&::before/after` alongside the declarations they relate to, so the reader can scan one block.
- **Placement/variant “clusters” can be their own nested scopes**: for components like tooltip, group placement variants as multiple `@scope (:scope:is(...)) { … }` blocks inside `@layer variants`.

## Moon tasks

`@luna/ds` is source-only (no package `build` script). It uses the shared **bun-ts-lib** task definitions in [`.moon/tasks/ts-lib.yml`](../../.moon/tasks/ts-lib.yml). This project’s [`moon.yml`](moon.yml) inherits:

| Task            | Purpose                                                                         |
| --------------- | ------------------------------------------------------------------------------- |
| **`clean`**     | Clear `dist/` and `node_modules` under this package (inherited git-clean task). |
| **`typecheck`** | Run `tsc --noEmit` for this package (Moon runs the command in `packages/ds/`).  |

```sh
moon run ds:clean
moon run ds:typecheck
```

Repo-wide checks (`bun run typecheck`, `bun run check`) are documented in the [root README](../../README.md#code-quality-bun).

## Resources

- [MDN CSS](https://developer.mozilla.org/en-US/docs/Web/CSS)
  - [`@layer`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@layer) (cascade layers)
  - [`@scope`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@scope) (scoped styling + proximity)
  - [`:scope`](https://developer.mozilla.org/en-US/docs/Web/CSS/:scope) (scope root selector)
  - [CSS Nesting (`&`)](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_nesting/Using_CSS_nesting) (nesting selector)
  - [`@container`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@container) (container queries)
  - [`:is()`](https://developer.mozilla.org/en-US/docs/Web/CSS/:is) (selector lists / specificity management)
- [Lightning CSS](https://lightningcss.dev/)
- [Tailwind CSS](https://tailwindcss.com/docs)

This design system draws inspiration from the following libraries and tools:

- [Pico CSS](https://picocss.com/)
- [UnoCSS](https://unocss.dev/)
- [Daisy UI](https://daisyui.com/)
