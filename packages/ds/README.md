# `@luna/ds`

Shared, CSS-first design-system styles for Luna apps.

## Stylesheet import

Import the DS stylesheet directly from your app CSS:

```css
@import "@luna/ds/styles.css";
```

If needed, app-specific overrides can still live in app-level CSS after this import.

## Vite config export (optional)

`@luna/ds` exports a Vite config object with Lightning CSS defaults:

```ts
import dsConfig from "@luna/ds/vite.config";
```

Merge it into your app `vite.config.ts` with `mergeConfig`.

## Architecture

The DS is authored as pure CSS with explicit layers and tokens.
See `docs/design-system/` in the repository for roadmap, architecture, tokens, functions, and patterns.

## Scripts and moon tasks

`@luna/ds` is intentionally source-only and currently exposes only typecheck:

```sh
bun run typecheck --cwd packages/ds
moon run ds:typecheck
```

## Resources

- [MDN CSS](https://developer.mozilla.org/en-US/docs/Web/CSS)
- [Lightning CSS](https://lightningcss.dev/)

This design system draws inspiration from the following libraries and tools:

- [Pico CSS](https://picocss.com/)
- [UnoCSS](https://unocss.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Daisy UI](https://daisyui.com/)
