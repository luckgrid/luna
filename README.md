# Luna

A monorepo starter template using moonrepo, bun, solid-start, and oxc.

## Tech Stack

- **Monorepo/build orchestration**: moonrepo
- **Tool/version manager**: proto
- **Package manager/runtime**: Bun
- **Quality tooling**: OXC (`oxlint`, `oxfmt`)
- **Type checking**: TypeScript project references (`tsc --build`)
- **Current app (`apps/web`)**: SolidStart + Vite + Nitro + Solid Router

## Toolchains

- Versions are pinned in `.prototools` (`proto`, `moon`, `bun`).
- Install toolchain with `proto install`.
- Bun is the package manager/runtime (`.moon/toolchains.yml` uses `packageManager: bun`).

## Workspaces

- Workspaces live under `apps/*` and `packages/*`.
- Scripts in `package.json` are the main entry points for quality checks and app-layer workflows.

### Apps

- Web app: [`apps/web/README.md`](apps/web/README.md)
- API app: [`apps/api/README.md`](apps/api/README.md) - FastAPI + Pydantic AI backend for AI features

### Packages

- UI components: [`packages/ui/README.md`](packages/ui/README.md)
- Design system: [`packages/ds/README.md`](packages/ds/README.md)
- Design system docs: [`docs/design-system/`](docs/design-system/)

## Setup

```sh
# install tools pinned in .prototools (proto, moon, bun)
proto install

# install workspace dependencies with bun
bun install

# run quality checks and app tasks
bun run check
moon run web:dev
```

## Code Quality

Code quality commands (root `package.json` scripts):

```sh
bun run lint
bun run format:check
bun run typecheck
bun run check      # lint + format:check + typecheck

bun run lint:fix
bun run format
bun run fix        # lint:fix + format
```

## Moon Tasks

Moon commands:

```sh
moon run web:dev
moon run web:build
moon run web:start

moon run api:dev
moon run api:start
moon run api:setup

# run across application-layer projects from root scripts
bun run dev
bun run build
```

## Running Both Apps

Run both the web app and API concurrently:

```sh
# Terminal 1 - Web app
moon run web:dev

# Terminal 2 - API
moon run api:dev
```

## Configurations

### Proto

- tool/version pins: [`.prototools`](.prototools)

### OXC

- formatter: [`.oxfmtrc.json`](.oxfmtrc.json)
- linter: [`.oxlintrc.json`](.oxlintrc.json)

### TypeScript

- root project graph: [`tsconfig.json`](tsconfig.json)
- shared compiler options: [`tsconfig.options.json`](tsconfig.options.json)
- project-level configs: `apps/*/tsconfig.json` and `packages/*/tsconfig.json`
- TS build cache location: `.cache/`

### Moon

- workspace projects and VCS: [`.moon/workspace.yml`](.moon/workspace.yml)
- toolchain config: [`.moon/toolchains.yml`](.moon/toolchains.yml)
- shared app tasks: [`.moon/tasks/app.yml`](.moon/tasks/app.yml)
- shared library tasks: [`.moon/tasks/lib.yml`](.moon/tasks/lib.yml)
- root project config: [`moon.yml`](moon.yml)
- project configs: `apps/*/moon.yml` and `packages/*/moon.yml`

## Dependency Maintenance

Use Bun workspace commands from the repository root for Node.js dependencies:

```sh
# check outdated dependencies across all workspaces
bun outdated --recursive

# update dependencies within current semver ranges across all workspaces
bun update --recursive

# update to latest versions across all workspaces
bun update --recursive --latest
```

Python dependencies (API app):

```sh
# Install/update Python dependencies
cd apps/api
uv sync

# Add a dependency
uv add <package>
```

Toolchain (`.prototools`) maintenance:

```sh
# install/update pinned tools (proto, moon, bun)
proto install

# check for newer tool versions
proto outdated

# update a pinned tool version in .prototools
proto pin <tool> <version>
```
