# Luna Development Guide

Keep this file lean and directive-focused. Use `README.md` as the source of truth for detailed commands and workflows.

## Core Rules

- Bun-first monorepo: prefer Bun commands over npm/pnpm/yarn.
- Toolchain versions are pinned in `.prototools`; bootstrap with `bun run setup` (or `proto install` alone).
- Run commands from the repository root unless an app/package README says otherwise.
- Keep app-specific scripts in each app's `package.json`; shared orchestration goes through moon/root scripts.

## Quick References

- Stacks (tooling and documentation links): [`README.md` (Tech Stacks)](README.md#tech-stacks)
- Proto / Bun / Moon pins: [`README.md` (Core monorepo and toolchain)](README.md#core-monorepo-and-toolchain)
- Workspace paths and per-project READMEs: [`README.md` (Workspaces)](README.md#workspaces)
- Root scripts, quality checks, and moon targets or queries: [`README.md` (Commands)](README.md#commands)
- Setup flow: [`README.md` (Quick Start)](README.md#quick-start)
- Config file map: [`README.md` (Configuration map)](README.md#configuration-map)
- `outdated` / `update` (CLI) and manual add-remove per stack: [`README.md` (Dependency maintenance)](README.md#dependency-maintenance)
- Ports, stuck processes, shell `go` alias: [`README.md` (Troubleshooting)](README.md#troubleshooting)

## Key Paths

- Toolchain pins: [`.prototools`](.prototools)
- Root scripts/workspaces: [`package.json`](package.json)
- Repo-wide outdated / update: [`packages/cli`](packages/cli) — `luna outdated` / `luna update` (router in `packages/cli/src/main.ts`; Python/Go roots from Moon + `lib/repo.ts`)
- Moon workspace/toolchains/tasks: [`.moon/`](.moon/)
- TypeScript project references: [`tsconfig.json`](tsconfig.json), [`tsconfig.options.json`](tsconfig.options.json)
- OXC config: [`.oxlintrc.json`](.oxlintrc.json), [`.oxfmtrc.json`](.oxfmtrc.json)

## Workspaces

### Apps workspace (`apps/*`)

- **Follow the app README first**: each app owns its ports, env files, and dev/build/start commands.
- **Keep app-specific scripts local**: add orchestration to root/moon only when it benefits multiple apps.

### Packages workspace (`packages/*`)

- **Treat packages as shared infrastructure**: keep APIs stable and changes composable across apps.
- **Validate changes**: use package-level tasks (e.g. `moon run ds:typecheck`, `moon run ui:typecheck`) before relying on app builds.

### `packages/ds` (design-system) CSS guardrails

- **Follow the DS module pattern**: start with a single root `@scope`, then put module layers **inside** it. DS docs: `packages/ds/README.md` (“Scoped layers pattern” + “Complex modules”).
- **Prefer these layers**: `base` first; add `variants` / `patterns` only when needed (avoid ad-hoc layer names).
- **Use nested scopes for complex subtrees**: if a component has deep structure (items, triggers, placement variants), add a nested `@scope (...) { :scope { … } }` for that subtree.
- **Keep state/modifiers co-located**: place hover/open/active rules next to the base block they modify; for trigger-focused state, prefer parent selectors like `:is([open], [data-open]) &`.
- **Keep small one-offs inline**: avoid extra nested scopes for simple descendants with a couple declarations.

### `packages/ui` (Solid UI) guardrails

- **Prefer DS tokens/utilities**: base look-and-feel should come from `@luna/ds`, not per-component CSS drift.
- **Keep components small and composable**: avoid baking app-specific layout/content into shared UI components.
