# Luna

Luna is a polyglot monorepo starter built around **moonrepo** for the task graph, **proto** and [`.prototools`](.prototools) for pinned runtimes (Bun, Go, Python, moon), and **Bun workspaces** for JavaScript and TypeScript packages. The template layers three application stacks on that foundation: a **SolidStart** interactive app with SSR and streaming, a **Go + templ** static site generator for server-rendered HTML and Markdown, and a **FastAPI** service with Pydantic and Pydantic AI. Shared libraries live under `packages/*`. Use each app or package README for ports, env files, and moon task details.

## Tech Stacks

### Core monorepo and toolchain

- 🟢 [Bun](https://bun.sh/) ([documentation](https://bun.sh/docs)) — JavaScript runtime, package manager, and workspace command runner
- 🌙 [Moon](https://moonrepo.dev/) ([documentation](https://moonrepo.dev/docs)) — task orchestration and project graph
- ⚙️ [Proto](https://moonrepo.dev/proto) — installs and pins tools from [`.prototools`](.prototools)

### Interactive application stack (SolidStart)

SolidStart on **Vite** and **Nitro** is the stack for SPA-style apps that need **SSR**, **client-side fine-grained signals** (SolidJS), **routing** (Solid Router), and **streaming** responses. Nitro gives you a server runtime alongside the browser bundle, so the same project can grow full-stack APIs and server logic without leaving the framework.

- ⚛️ [SolidStart](https://start.solidjs.com/) ([documentation](https://docs.solidjs.com/solid-start)) — full-stack app framework for `apps/app`
- 🧩 [SolidJS](https://www.solidjs.com/) ([documentation](https://docs.solidjs.com/)) — reactive UI library
- 🔀 [Solid Router](https://docs.solidjs.com/solid-router/) — routing for Solid apps
- ⚡ [Vite](https://vite.dev/) ([guide](https://vite.dev/guide/)) — dev server and build tooling
- 🔥 [Nitro](https://nitro.build/) ([guide](https://nitro.build/guide)) — server runtime used by SolidStart
- 🎨 [Tailwind CSS v4](https://tailwindcss.com/) ([documentation](https://tailwindcss.com/docs)) — utility CSS used by `@luna/ds` (consumed from the interactive app and the static site pipeline)

### Static site generator stack (Go and templ)

The `apps/web` stack is a **server-side static site generator**: Go drives **templ** templates and **goldmark** Markdown into HTML on the server, with a classic SSG output layout. It fits marketing and content sites first, but you can add **signal-style** or fragment-driven interactivity (for example [Datastar](https://data-star.dev/) or similar) **without** a client-side router or heavy SPA shell when you outgrow pure static pages.

- 🐹 [Go](https://go.dev/) ([documentation](https://go.dev/doc/)) — runtime for the generator; version pinned in [`.prototools`](.prototools)
- 📄 [templ](https://templ.guide/) — compile-time HTML components; the CLI is a [Go 1.24+ tool dependency](https://templ.guide/quick-start/installation#go-install-as-tool) in [`apps/web/go.mod`](apps/web/go.mod) (`go tool templ` in moon tasks — no separate global install)
- 📝 [goldmark](https://github.com/yuin/goldmark) — Markdown rendering (including frontmatter-related extensions as wired in the app)

### API service stack (FastAPI and Pydantic)

The `apps/api` stack centers on **FastAPI**, **Pydantic**, and **Pydantic AI** for a **pure backend** HTTP API: validation, settings, and agent-style features stay on the server. The same patterns extend to larger deployments (multiple services, workers, or runtimes) when you outgrow a single process; this repo keeps one API project as the starting point.

- 🐍 [Python](https://www.python.org/) — runtime (version pinned in [`.prototools`](.prototools))
- 📦 [uv](https://docs.astral.sh/uv/) — environments and lockfiles
- 🚀 [FastAPI](https://fastapi.tiangolo.com/) — API framework
- 🤖 [Pydantic AI](https://ai.pydantic.dev/) — AI agent patterns on the backend
- ✅ [Pydantic](https://docs.pydantic.dev/) — schemas and models
- ⚙️ [pydantic-settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/) — environment-driven settings
- 🌐 [Uvicorn](https://www.uvicorn.org/) — ASGI server

### Workspace quality (TypeScript and OXC)

- 🧹 [OXC (`oxlint` + `oxfmt`)](https://oxc.rs/) ([documentation](https://oxc.rs/docs/guide/usage/linter.html)) — linting and formatting for JS/TS at the repo root
- 🟦 [TypeScript](https://www.typescriptlang.org/) ([documentation](https://www.typescriptlang.org/docs/)) — static typing and project references across workspaces

## Quick Start

Run commands from the repository root unless an app README says otherwise.

```sh
bun run install  # proto + Bun workspaces + Go modules (web:install) + Python venv (api:build / uv sync)
bun run dev      # all application-layer dev tasks (see moon query / app READMEs for subsets)
```

The `install` script runs `proto install`, then `bun install --ignore-scripts` (the flag avoids re-entering this root lifecycle when `bun install` invokes it), then `moon run web:install api:build` so Go and Python stacks match what `web:build` / `api:dev` expect without waiting for a first dev run. To install only JS workspaces, run `bun install` alone.

For a full compile of every application project first, run **`bun run build`**. Default ports are documented in each app README and can be overridden via [`.env.local`](.env.local).

## Workspaces

- **`apps/app/`** — SolidStart (SSR, Vite, Nitro) · [README](apps/app/README.md)
- **`apps/web/`** — Go + templ SSG · [README](apps/web/README.md)
- **`apps/api/`** — FastAPI + Pydantic AI · [README](apps/api/README.md)
- **`packages/ui/`** — shared Solid UI · [README](packages/ui/README.md)
- **`packages/ds/`** — design system / Tailwind · [README](packages/ds/README.md)

Moon wires install, templates, styles, build, and dev tasks per project; **`web:dev`** depends on an initial **`web:build`**, and **`api:dev`** depends on **`api:build`** (`uv sync`) so first-time dev pulls toolchains and dependencies. Root **`bun run install`** runs **`web:install`** and **`api:build`** up front so Go modules and the API venv are ready before **`bun run dev`**. For step-by-step task graphs (templ, Tailwind CLI, `go run`, Uvicorn, Vite), follow each workspace README above.

## Commands

### Root scripts (Bun)

These scripts target the **application layer** (all `apps/*` projects), matching what you use day to day from the repo root:

```sh
bun run install     # proto + Bun workspaces + web:install + api:build (see Quick Start)
bun run dev         # moon run :dev --query "projectLayer=application"
bun run build       # moon run :build --query "projectLayer=application"
bun run start       # moon run :start --query "projectLayer=application"
bun run clean       # moon clean --all (+ root clean steps — see package.json)
```

### Code quality (Bun)

```sh
bun run lint
bun run format:check
bun run typecheck
bun run check       # lint + format:check + typecheck (includes api/web moon tasks)

bun run lint:fix
bun run format
bun run fix         # lint:fix + format (includes api/web moon tasks)
```

### Moon: single apps, subsets, and packages

Pass **multiple `project:task` targets** to run them in one invocation:

```sh
moon run app:dev api:dev              # interactive app + API only (no web)
moon run app:build api:build web:build
```

Use **`--query`** to filter the graph instead of listing every target (same query language as `moon query projects`):

```sh
moon run :dev --query 'project=[app,api]'
moon run :typecheck --query "projectLayer=library"   # shared packages (tasks they inherit)
moon query projects --help                           # filters: --id, --language, --layer, etc.
```

Examples for **shared packages**:

```sh
moon run ds:typecheck
moon run ui:typecheck
```

## Configuration map

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

## Dependency maintenance

Repo-wide **outdated checks** and **upgrades** are scripted so every toolchain stays in sync:

[`scripts/outdated.sh`](scripts/outdated.sh) reports outdated **proto pins**, **Bun workspace** packages, **Python / uv** lockfile upgrades (dry-run), and **Go** modules. [`scripts/update.sh`](scripts/update.sh) applies updates across those tiers (proto → Bun → uv → Go). After `bun run update`, review diffs and run `bun run check` before committing.

```sh
bun run outdated:check   # print reports only (exit 0)
bun run outdated         # strict: exit 1 if anything is outdated (CI-friendly)
bun run update           # bump pins and dependencies repo-wide; then review and run bun run check
```

**Per stack (manual add / remove)** — use these when you are changing one project, not refreshing everything:

- **Toolchain (proto)** — edit [`.prototools`](.prototools), then `proto install` or `bun run install` (or `proto pin <tool> <version>`). Removing a tool line drops it from proto’s install set for this repo.
- **Bun / workspaces** — from the repo root, add to a workspace with `bun add <pkg> --cwd apps/app` (or `--cwd packages/ui`, etc.); use `bun add -d <pkg> --cwd <path>` for devDependencies. Remove with `bun remove <pkg> --cwd <path>`. Root-only deps: `bun add <pkg>` at the root.
- **Python (`apps/api`)** — `cd apps/api` then `uv add <package>` / `uv remove <package>` (updates `pyproject.toml` and `uv.lock`); sync with `uv sync`.
- **Go (`apps/web`)** — `cd apps/web` then `go get example.com/module@v1.2.3` (or `@latest`); drop a direct dependency with `go get example.com/module@none` and run `go mod tidy`. The **templ** CLI stays aligned with the library via `go get -tool github.com/a-h/templ/cmd/templ@<version>` when you bump templ intentionally.

## Troubleshooting

### Port already in use (`EADDRINUSE`)

Another process is still bound to the port (often after stopping a dev server).

```sh
lsof -i :8080    # API default; try :3000 for web, :3001 for app
```

Note the `PID` from `lsof`, then:

```sh
kill -9 <PID>
```

### Stale API or Vite processes

If you know the command line, you can narrow cleanup:

```sh
ps aux | grep -E "(uvicorn|vite)" | grep -v grep
pkill -f "uvicorn src.main:app"   # API (adjust if your entrypoint differs)
```
