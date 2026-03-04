# Luna Development Guide

Keep this file lean and directive-focused. Use `README.md` as the source of truth for detailed commands and workflows.

## Core Rules

- Bun-first monorepo: prefer Bun commands over npm/pnpm/yarn.
- Toolchain versions are pinned in `.prototools`; install with `proto install`.
- Run commands from the repository root unless an app/package README says otherwise.
- Keep app-specific scripts in each app's `package.json`; shared orchestration goes through moon/root scripts.

## Fast References

- Project overview and stack: `README.md` -> `Tech Stack`
- Toolchain and `.prototools` usage: `README.md` -> `Toolchains`
- Workspace layout and root scripts: `README.md` -> `Workspaces`
- Setup flow: `README.md` -> `Quick start`
- Lint/format/typecheck commands: `README.md` -> `Code Quality`
- App task execution: `README.md` -> `Moon Tasks`
- Config file map: `README.md` -> `Configurations`
- Dependency and toolchain maintenance: `README.md` -> `Dependency Maintenance`

## Key Paths

- Toolchain pins: `.prototools`
- Root scripts/workspaces: `package.json`
- Moon workspace/toolchains/tasks: `.moon/`
- TypeScript project references: `tsconfig.json`, `tsconfig.options.json`
- OXC config: `.oxlintrc.json`, `.oxfmtrc.json`
