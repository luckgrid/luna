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

## Moon tasks

`@luna/ui` uses the shared **bun-ts-lib** definitions in [`.moon/tasks/ts-lib.yml`](../../.moon/tasks/ts-lib.yml). This project’s [`moon.yml`](moon.yml) inherits:

| Task            | Purpose                                                                         |
| --------------- | ------------------------------------------------------------------------------- |
| **`clean`**     | Clear `dist/` and `node_modules` under this package (inherited git-clean task). |
| **`typecheck`** | Run `tsc --noEmit` for this package (Moon runs the command in `packages/ui/`).  |

```sh
moon run ui:clean
moon run ui:typecheck
```

Repo-wide checks (`bun run typecheck`, `bun run check`) are documented in the [root README](../../README.md#code-quality-bun).

## Add a new component

1. Add a new `*.tsx` file under the appropriate `src/` folder.
2. Add a matching subpath export in `packages/ui/package.json`.
3. Consume from apps with a flat import (for example `@luna/ui/badge`).

## Internal imports

Inside `packages/ui/src`, prefer relative imports (for example `../utils/core` or `../utils/navigation`) instead of app-level aliases like `~/*`.

Extract shared logic into `src/utils/*` and consume it from components, rather than duplicating helpers across component files.
