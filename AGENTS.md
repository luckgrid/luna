# Luna Development Guide

Keep this file lean and directive-focused. Use `README.md` as the source of truth for detailed commands and workflows.

## Core Rules

- Bun-first monorepo: prefer Bun commands over npm/pnpm/yarn.
- Toolchain versions are pinned in `.prototools`; install with `proto install`.
- Run commands from the repository root unless an app/package README says otherwise.
- Keep app-specific scripts in each app's `package.json`; shared orchestration goes through moon/root scripts.

## Quick References

- Project overview and stack: [`README.md` (Tech Stack)](README.md#tech-stack)
- Toolchain and `.prototools` usage: [`README.md` (Dependencies and Tools)](README.md#dependencies-and-tools)
- Workspace layout and root scripts: [`README.md` (Workspace Layout)](README.md#workspace-layout), [`README.md` (Common Commands)](README.md#common-commands)
- Setup flow: [`README.md` (Quick start)](README.md#quick-start)
- Lint/format/typecheck commands: [`README.md` (Code Quality)](README.md#code-quality), [`README.md` (Common Commands)](README.md#common-commands)
- App task execution: [`README.md` (App Tasks)](README.md#app-tasks-moon)
- Config file map: [`README.md` (Configuration Map)](README.md#configuration-map)
- Dependency and toolchain maintenance: [`README.md` (Dependency Maintenance)](README.md#dependency-maintenance)

## Key Paths

- Toolchain pins: [`.prototools`](.prototools)
- Root scripts/workspaces: [`package.json`](package.json)
- Moon workspace/toolchains/tasks: [`.moon/`](.moon/)
- TypeScript project references: [`tsconfig.json`](tsconfig.json), [`tsconfig.options.json`](tsconfig.options.json)
- OXC config: [`.oxlintrc.json`](.oxlintrc.json), [`.oxfmtrc.json`](.oxfmtrc.json)
