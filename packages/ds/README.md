# `@luna/ds`

Shared design-system styles and UnoCSS config for Luna apps.

## Stylesheet import

Import the DS app entry from your app root:

```ts
import "@luna/ds/app";
```

This entry imports both Uno's generated stylesheet and `@luna/ds/styles.css`.

If needed, app-specific overrides can still live in app-level CSS after this import.

You can also import only shared tokens/base styles:

```css
@import "@luna/ds/styles.css";
```

This keeps app-specific overrides local to each app stylesheet.

## UnoCSS config

`@luna/ds` exports its shared UnoCSS config:

- `@luna/ds/uno.config` -> default Uno config export

Apps can point Uno's Vite plugin at this config via `configFile` and avoid per-app `uno.config.ts` files.

## Scripts and moon tasks

`@luna/ds` is intentionally source-only and currently exposes only typecheck:

```sh
bun run typecheck --cwd packages/ds
moon run ds:typecheck
```
