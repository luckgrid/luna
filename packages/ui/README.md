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

## Source layout

Components are grouped by category in `src/`:

- `src/action/`
- `src/display/`
- `src/form/`
- `src/link/`
- `src/feedback/`

The `tsup` config auto-discovers component entrypoints from these folders and emits ESM + d.ts files to `dist/`.

## Dist build process

`@luna/ui` ships from `dist/` and uses wildcard subpath exports in `package.json`.

Exports stay wildcard-based:

- `import` -> `./dist/*.js`
- `types` -> `./dist/*.d.ts`

The build pipeline is:

1. `tsup` compiles discovered entrypoints to `dist/*.js`.
2. `tsup` emits declaration files as top-level `dist/*.d.ts` for wildcard type exports.

### Auto-discovery and mapping

`tsup.config.ts` is the source of truth for entry discovery and export name mapping.

- The same entry map is used for JS and type generation.

This keeps wildcard exports while avoiding manual subpath mapping in `package.json`.

Current entrypoint discovery rules:

- `src/action`, `src/display`, `src/form`, `src/link`, `src/feedback`: each `*.tsx` file becomes a public subpath.
- `src/utils/cx.ts` is mapped to the public `utils` subpath (`@luna/ui/utils`).

## Build and typecheck

From the repository root:

```sh
bun run build --cwd packages/ui
bun run typecheck --cwd packages/ui
```

With moon:

```sh
moon run ui:build
moon run ui:typecheck
```

## Add a new component

1. Add `*.tsx` under one of the source category folders.
2. Run `moon run ui:build` (or `bun run build --cwd packages/ui`) to generate `dist` artifacts.
3. Consume from apps with the flat import path (for example `@luna/ui/badge`).

## Utils export caveat

`@luna/ui/utils` currently maps to a single file: `src/utils/cx.ts`.

If multiple utils are needed in the future, this must change in one of two ways:

- Add a `src/utils/index.ts` aggregator and map `utils` to that index.
- Or export utilities as separate subpaths (`@luna/ui/cx`, `@luna/ui/xyz`) instead of a single `utils` entrypoint.
