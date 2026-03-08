# Luna

Monorepo starter template powered by Bun + moonrepo, with:

- `apps/web`: SolidStart frontend (Vite + Nitro + Solid Router)
- `apps/api`: FastAPI backend with Pydantic AI
- shared UI/design-system packages under `packages/*`

## Tech Stack

- Monorepo orchestration with moonrepo + Bun workspaces
- SolidStart frontend (`apps/web`) with Vite and Nitro
- FastAPI backend (`apps/api`) using Pydantic AI + Pydantic
- OXC + TypeScript for quality and type safety
- Toolchains pinned with proto (`.prototools`)

For the complete dependency list (with links), see [Dependencies and Tools](#dependencies-and-tools).

## Quick Start

Run everything from the repository root unless an app README says otherwise.

```sh
# 1) Install pinned tools from .prototools (proto, moon, bun, python)
proto install

# 2) Install JS/TS dependencies
bun install

# 3) Install API Python dependencies
moon run api:setup

# 4) Verify quality checks
bun run check
```

## Workspace Layout

```txt
apps/
  web/        # SolidStart marketing app
  api/        # FastAPI + Pydantic AI backend
packages/
  ui/         # shared UI components
  ds/         # shared design system
```

More details:

- Web app: [`apps/web/README.md`](apps/web/README.md)
- API app: [`apps/api/README.md`](apps/api/README.md)
- UI package: [`packages/ui/README.md`](packages/ui/README.md)
- Design system package: [`packages/ds/README.md`](packages/ds/README.md)

## Common Commands

### Root Scripts

```sh
bun run dev         # run application-layer dev tasks
bun run build       # build application-layer projects
bun run start       # start application-layer projects
bun run clean       # moon clean --all
```

### Code Quality

```sh
bun run lint
bun run format:check
bun run typecheck
bun run check       # lint + format:check + typecheck

bun run lint:fix
bun run format
bun run fix         # lint:fix + format
```

### App Tasks (moon)

```sh
# Web
moon run web:dev
moon run web:build
moon run web:start

# API
moon run api:setup
moon run api:dev
moon run api:start
```

## Run Both Apps Locally

```sh
# Single command from repo root
bun run dev

# Or run in separate terminals
# Terminal 1: moon run web:dev
# Terminal 2: moon run api:dev
```

`bun run dev` uses the root script to run all app-level dev tasks together.

Web runs via Vite dev server. API runs via Uvicorn on port `8080`.

## Managing Running Servers

### Check Running Processes

```sh
# Check if a port is in use (e.g., 8080 for API, 3000 for web)
lsof -i :8080
lsof -i :3000

# List all node/python processes running uvicorn or vite
ps aux | grep -E "(uvicorn|vite)" | grep -v grep
```

### Kill Running Servers

```sh
# Kill process on a specific port
kill -9 <PID>

# Or use pkill to kill by process name
pkill -f "uvicorn src.main:app"   # API
pkill -f "vite"                   # Web

# Kill all node processes (if needed)
pkill -f "node"
```

### Common Issues

- **Address already in use**: Port is still held by a previous process. Find and kill it with `lsof -i :<port>`.
- **Multiple dev servers**: Running `bun run dev` or `moon run :dev` starts all apps - use `moon run api:dev` or `moon run web:dev` to run a single app.

## Dependencies and Tools

### TypeScript and Frontend

- 🟢 [Bun](https://bun.sh/) - JS runtime, package manager, and workspace command runner
- 🌙 [Moon](https://moonrepo.dev/) - task orchestration and project graph
- ⚙️ [Proto](https://moonrepo.dev/proto) - pinned toolchain management via [`.prototools`](.prototools)
- ⚛️ [SolidStart](https://start.solidjs.com/) - full-stack app framework for `apps/web`
- 🧩 [SolidJS](https://www.solidjs.com/) - reactive UI library
- 🔀 [Solid Router](https://docs.solidjs.com/solid-router/) - routing for Solid apps
- ⚡ [Vite](https://vite.dev/) - dev server and build tooling
- 🔥 [Nitro](https://nitro.build/) - server runtime used by SolidStart
- 🧹 [OXC (`oxlint` + `oxfmt`)](https://oxc.rs/) - linting and formatting
- 🟦 [TypeScript](https://www.typescriptlang.org/) - static typing and project references

### Python and Backend AI API

- 🐍 [Python](https://www.python.org/) - backend runtime (version pinned in [`.prototools`](.prototools))
- 📦 [uv](https://docs.astral.sh/uv/) - Python dependency and environment management
- 🚀 [FastAPI](https://fastapi.tiangolo.com/) - API framework
- 🤖 [Pydantic AI](https://ai.pydantic.dev/) - AI agent framework for backend AI features
- ✅ [Pydantic](https://docs.pydantic.dev/) - schema validation and data modeling
- ⚙️ [pydantic-settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/) - environment-driven settings
- 🌐 [Uvicorn](https://www.uvicorn.org/) - ASGI server

## Configuration Map

- Tool/version pins: [`.prototools`](.prototools)
- Workspace root scripts: [`package.json`](package.json)
- Moon workspace graph + VCS: [`.moon/workspace.yml`](.moon/workspace.yml)
- Moon toolchains: [`.moon/toolchains.yml`](.moon/toolchains.yml)
- Shared app tasks: [`.moon/tasks/app.yml`](.moon/tasks/app.yml)
- Shared lib tasks: [`.moon/tasks/lib.yml`](.moon/tasks/lib.yml)
- Root moon config: [`moon.yml`](moon.yml)
- TypeScript project graph: [`tsconfig.json`](tsconfig.json)
- Shared TypeScript options: [`tsconfig.options.json`](tsconfig.options.json)
- OXC formatter config: [`.oxfmtrc.json`](.oxfmtrc.json)
- OXC linter config: [`.oxlintrc.json`](.oxlintrc.json)

## Dependency Maintenance

### JS/TS (Bun workspaces)

```sh
# list outdated packages across workspaces
bun outdated --recursive

# update within semver ranges
bun update --recursive

# update to latest versions
bun update --recursive --latest
```

### Python (`apps/api`)

```sh
# sync dependencies from pyproject/lock
cd apps/api
uv sync

# add a dependency
uv add <package>
```

### Toolchain Pins (`.prototools`)

```sh
# install or update pinned tools
proto install

# check for newer versions
proto outdated

# pin a specific version
proto pin <tool> <version>
```

## Resources

### Core Tooling

- [proto docs](https://moonrepo.dev/proto)
- [moonrepo docs](https://moonrepo.dev/docs)
- [Bun docs](https://bun.sh/docs)
- [uv docs](https://docs.astral.sh/uv/)

### Frontend Web Apps

- [SolidStart docs](https://docs.solidjs.com/solid-start)
- [SolidJS docs](https://docs.solidjs.com/)
- [Solid Router docs](https://docs.solidjs.com/solid-router)
- [Vite docs](https://vite.dev/guide/)
- [Nitro docs](https://nitro.build/guide)

### Backend AI API

- [FastAPI docs](https://fastapi.tiangolo.com/)
- [Pydantic AI docs](https://ai.pydantic.dev/)
- [Pydantic docs](https://docs.pydantic.dev/)
- [pydantic-settings docs](https://docs.pydantic.dev/latest/concepts/pydantic_settings/)
- [Uvicorn docs](https://www.uvicorn.org/)

### Quality and Language

- [OXC docs](https://oxc.rs/docs/guide/usage/linter.html)
- [TypeScript docs](https://www.typescriptlang.org/docs/)
