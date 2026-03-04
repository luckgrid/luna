# Luna Development Guide

Keep this file lean and directive-focused. Use `README.md` as the source of truth for detailed commands and workflows.

## Core Rules

- Bun-first monorepo: prefer Bun commands over npm/pnpm/yarn.
- Toolchain versions are pinned in `.prototools`; install with `proto install`.
- Run commands from the repository root unless an app/package README says otherwise.
- Keep app-specific scripts in each app's `package.json`; shared orchestration goes through moon/root scripts.

## Fast References

- Project overview and stack: [`README.md` (Tech Stack)](README.md#tech-stack)
- Toolchain and `.prototools` usage: [`README.md` (Toolchains)](README.md#toolchains)
- Workspace layout and root scripts: [`README.md` (Workspaces)](README.md#workspaces)
- Setup flow: [`README.md` (Quick start)](README.md#quick-start)
- Lint/format/typecheck commands: [`README.md` (Code Quality)](README.md#code-quality)
- App task execution: [`README.md` (Moon Tasks)](README.md#moon-tasks)
- Config file map: [`README.md` (Configurations)](README.md#configurations)
- Dependency and toolchain maintenance: [`README.md` (Dependency Maintenance)](README.md#dependency-maintenance)

## Key Paths

- Toolchain pins: [`.prototools`](.prototools)
- Root scripts/workspaces: [`package.json`](package.json)
- Moon workspace/toolchains/tasks: [`.moon/`](.moon/)
- TypeScript project references: [`tsconfig.json`](tsconfig.json), [`tsconfig.options.json`](tsconfig.options.json)
- OXC config: [`.oxlintrc.json`](.oxlintrc.json), [`.oxfmtrc.json`](.oxfmtrc.json)
