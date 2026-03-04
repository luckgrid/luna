# `@luna/ui`

Reusable Solid UI components for Luna apps.

## Imports

Use flat subpath imports:

- `@luna/ui/button`
- `@luna/ui/accordion`
- `@luna/ui/input`
- `@luna/ui/link`
- `@luna/ui/tooltip`
- `@luna/ui/utils`

## Source-first package

`@luna/ui` is consumed directly from `src/` entry files in `package.json` exports.
There is no local `dist/` build step for this package.

This keeps JSX transformation in the app's SolidStart/Vite pipeline and avoids runtime mismatches.

## Source layout

Component folders in `src/`:

- `src/action/`
- `src/display/`
- `src/feedback/`
- `src/form/`
- `src/navigation/`

Shared utilities (non-component logic):

- `src/utils/`

## Typecheck

From the repository root:

```sh
bun run typecheck --cwd packages/ui
```

With moon:

```sh
moon run ui:typecheck
```

## Add a new component

1. Add a new `*.tsx` file under the appropriate `src/` folder.
2. Add a matching subpath export in `packages/ui/package.json`.
3. Consume from apps with a flat import (for example `@luna/ui/badge`).

## Internal imports

Inside `packages/ui/src`, prefer relative imports (for example `../utils/cx` or `../utils/navigation`) instead of app-level aliases like `~/*`.

Extract shared logic into `src/utils/*` and consume it from components, rather than duplicating helpers across component files.
