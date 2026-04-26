# Luna

Monorepo starter template powered by Bun + moonrepo, with:

- `apps/app`: SolidStart interactive/SSR app (Vite + Nitro + Solid Router)
- `apps/web`: Static marketing/content site (Go + templ + goldmark)
- `apps/api`: FastAPI backend with Pydantic AI
- shared UI/design-system packages under `packages/*`

## Tech Stack

- Monorepo orchestration with moonrepo + Bun workspaces
- SolidStart interactive app (`apps/app`) with Vite and Nitro
- Go + templ static site (`apps/web`) with goldmark Markdown and reusable layout templates
- FastAPI backend (`apps/api`) using Pydantic AI + Pydantic
- OXC + TypeScript for quality and type safety
- Toolchains pinned with proto (`.prototools`)

For the complete dependency list (with links), see [Dependencies and Tools](#dependencies-and-tools).

## Quick Start

Run everything from the repository root unless an app README says otherwise.

```sh
# 1) Install pinned tools from .prototools (proto, moon, bun, python, go)
proto install

# 2) Install JS/TS dependencies
bun install

# 3) Boot all apps with one command
bun run dev
```

`bun run dev` (= `moon run :dev --query "projectLayer=application"`) runs each
app's `dev` task, which has its `setup`/`generate`/`css` deps wired up — so the
first run also handles:

- `web:setup` → `go mod download` (pulls templ, goldmark, goldmark-meta)
- `web:css` → builds `apps/web/dist/styles.css` via the Tailwind v4 CLI (`src/styles.css` imports `@luna/ds`)
- `web:dev` → `go tool templ generate --watch` + `go run . --serve` in `apps/web`
- `api:setup` → `uv sync` (creates `.venv`, installs Python deps)
- `api:dev` → Uvicorn with reload
- `app:dev` → Vite dev server

The [`templ`](https://templ.guide/) CLI is pinned per-project via the `tool`
directive in [`apps/web/go.mod`](apps/web/go.mod) and invoked with `go tool
templ` — no global Go install required.

Default ports: `app` → `3001`, `web` → `3000`, `api` → `8080` (override via
[`.env.local`](.env.local)).

To verify quality before committing:

```sh
bun run check     # oxlint + oxfmt --check + tsc --build
```

## Workspace Layout

```txt
apps/
  app/        # SolidStart interactive/SSR app
  web/        # Go + templ static marketing/content site
  api/        # FastAPI + Pydantic AI backend
packages/
  ui/         # shared Solid UI components (consumed by apps/app)
  ds/         # shared design-system styles (consumed by both apps)
```

More details:

- App: [`apps/app/README.md`](apps/app/README.md)
- Web: [`apps/web/README.md`](apps/web/README.md)
- API: [`apps/api/README.md`](apps/api/README.md)
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
# App (SolidStart)
moon run app:dev
moon run app:build
moon run app:start

# Web (Go + templ static site)
moon run web:dev
moon run web:build
moon run web:setup

# API
moon run api:setup
moon run api:dev
moon run api:start
```

## Run All Apps Locally

```sh
# Single command from repo root (all application-layer dev tasks)
bun run dev

# Or run in separate terminals
# Terminal 1: moon run app:dev
# Terminal 2: moon run web:dev
# Terminal 3: moon run api:dev
```

`bun run dev` uses the root script to run all app-level dev tasks together.

App runs via Vite dev server (`APP_PORT=3001`). Web serves generated `dist/`
on `WEB_PORT=3000`. API runs via Uvicorn on `API_PORT=8080`.

## Managing Running Servers

### Check Running Processes

```sh
# Check if a port is in use (e.g., 8080 for API, 3001 for app, 3000 for web)
lsof -i :8080
lsof -i :3001
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
- **`go` runs `gh browse` or errors like "accepts at most 1 arg"**: An interactive shell alias for `go` is still active. Sourcing `.zshrc` does not remove aliases already defined in that session — run `unalias go` (or open a new terminal), then ensure real Go is on `PATH` (after `proto install`, use `command -v go` and `go version`). To bypass an alias once, use `\go` or `command go`. Moon tasks invoke `go` without your zsh aliases, so `moon run web:generate` still works.

## Dependencies and Tools

### TypeScript and Frontend

- 🟢 [Bun](https://bun.sh/) - JS runtime, package manager, and workspace command runner
- 🌙 [Moon](https://moonrepo.dev/) - task orchestration and project graph
- ⚙️ [Proto](https://moonrepo.dev/proto) - pinned toolchain management via [`.prototools`](.prototools)
- ⚛️ [SolidStart](https://start.solidjs.com/) - full-stack app framework for `apps/app`
- 🧩 [SolidJS](https://www.solidjs.com/) - reactive UI library
- 🔀 [Solid Router](https://docs.solidjs.com/solid-router/) - routing for Solid apps
- ⚡ [Vite](https://vite.dev/) - dev server and build tooling
- 🔥 [Nitro](https://nitro.build/) - server runtime used by SolidStart
- 🎨 [Tailwind CSS v4](https://tailwindcss.com/) - utility CSS used by `@luna/ds`
- 🧹 [OXC (`oxlint` + `oxfmt`)](https://oxc.rs/) - linting and formatting
- 🟦 [TypeScript](https://www.typescriptlang.org/) - static typing and project references

### Go and Static Site

- 🐹 [Go](https://go.dev/) - runtime for the static site generator (`apps/web`), version pinned in [`.prototools`](.prototools)
- 📄 [templ](https://templ.guide/) - compile-time HTML components for Go; the CLI is a [Go 1.24+ tool dependency](https://templ.guide/quick-start/installation#go-install-as-tool) in [`apps/web/go.mod`](apps/web/go.mod) (`go tool templ`, used by moon tasks — no separate global install)
- 📝 [goldmark](https://github.com/yuin/goldmark) - Markdown rendering with frontmatter

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
- Shared TS app tasks: [`.moon/tasks/ts-app.yml`](.moon/tasks/ts-app.yml) (`language: typescript`, `layer: application`, `stack: frontend`)
- Shared TS lib tasks: [`.moon/tasks/ts-lib.yml`](.moon/tasks/ts-lib.yml) (`language: typescript`, `layer: library`)
- Shared Python API tasks: [`.moon/tasks/py-api.yml`](.moon/tasks/py-api.yml) (`language: python`, `stack: backend`)
- Shared Go web tasks: [`.moon/tasks/go-web.yml`](.moon/tasks/go-web.yml) (`language: go`, `stack: frontend`)
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

### Go (`apps/web`)

```sh
cd apps/web

# bump the templ CLI (tool) and library together, then tidy
go get -tool github.com/a-h/templ/cmd/templ@latest
go mod tidy
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
- [Tailwind CSS v4 docs](https://tailwindcss.com/docs)

### Static Site (Go)

- [Go docs](https://go.dev/doc/)
- [templ docs](https://templ.guide/)
- [goldmark docs](https://github.com/yuin/goldmark)

### Backend AI API

- [FastAPI docs](https://fastapi.tiangolo.com/)
- [Pydantic AI docs](https://ai.pydantic.dev/)
- [Pydantic docs](https://docs.pydantic.dev/)
- [pydantic-settings docs](https://docs.pydantic.dev/latest/concepts/pydantic_settings/)
- [Uvicorn docs](https://www.uvicorn.org/)

### Quality and Language

- [OXC docs](https://oxc.rs/docs/guide/usage/linter.html)
- [TypeScript docs](https://www.typescriptlang.org/docs/)
