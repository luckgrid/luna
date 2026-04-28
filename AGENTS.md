# Luna Development Guide

Keep this file lean and directive-focused. Use `README.md` as the source of truth for detailed commands and workflows.

## Core Rules

- Bun-first monorepo: prefer Bun commands over npm/pnpm/yarn.
- Toolchain versions are pinned in `.prototools`; install with `proto install`.
- Run commands from the repository root unless an app/package README says otherwise.
- Keep app-specific scripts in each app's `package.json`; shared orchestration goes through moon/root scripts.

## Quick References

- Stacks (tooling and documentation links): [`README.md` (Tech Stacks)](README.md#tech-stacks)
- Proto / Bun / Moon pins: [`README.md` (Core monorepo and toolchain)](README.md#core-monorepo-and-toolchain)
- Workspace paths and per-project READMEs: [`README.md` (Workspaces)](README.md#workspaces)
- Root scripts, quality checks, and moon targets or queries: [`README.md` (Commands)](README.md#commands)
- Setup flow: [`README.md` (Quick Start)](README.md#quick-start)
- Config file map: [`README.md` (Configuration map)](README.md#configuration-map)
- `outdated` / `update` scripts and manual add-remove per stack: [`README.md` (Dependency maintenance)](README.md#dependency-maintenance)
- Ports, stuck processes, shell `go` alias: [`README.md` (Troubleshooting)](README.md#troubleshooting)

## Key Paths

- Toolchain pins: [`.prototools`](.prototools)
- Root scripts/workspaces: [`package.json`](package.json)
- Repo-wide outdated / update: [`scripts/outdated.sh`](scripts/outdated.sh), [`scripts/update.sh`](scripts/update.sh)
- Moon workspace/toolchains/tasks: [`.moon/`](.moon/)
- TypeScript project references: [`tsconfig.json`](tsconfig.json), [`tsconfig.options.json`](tsconfig.options.json)
- OXC config: [`.oxlintrc.json`](.oxlintrc.json), [`.oxfmtrc.json`](.oxfmtrc.json)
