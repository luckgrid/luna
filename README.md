# Luna

Luna is a polyglot monorepo starter built around **moonrepo** for the task graph, **proto** for pinned runtimes, and the **`luna` CLI** (Rust, Starbase + Clap) for orchestration across all toolchain layers. Each stack brings its own runtime — **Bun** for JS/TS, **Go** for Hugo, **Python** for FastAPI — managed and pinned by Proto. It layers three application stacks: a **SolidStart** interactive app with SSR and streaming, a **Hugo** static site for Markdown and templates, and a **FastAPI** service with Pydantic and Pydantic AI. Shared libraries live under **`packages/*`**. Ports, environment files, and moon tasks are covered in the sections and READMEs below.

## Tech Stacks

### Core monorepo and toolchain

- 🌙 [Moon](https://moonrepo.dev/) ([documentation](https://moonrepo.dev/docs)) — task orchestration and project graph
- ⚙️ [Proto](https://moonrepo.dev/proto) — installs and pins all tools from [`.prototools`](.prototools)
- 🦀 [Rust](https://www.rust-lang.org/) — powers the `luna` CLI (`packages/cli`); pinned in [`.prototools`](.prototools)

### Interactive application stack (SolidStart)

SolidStart on **Vite** and **Nitro** is the stack for SPA-style apps that need **SSR**, **client-side fine-grained signals** (SolidJS), **routing** (Solid Router), and **streaming** responses. Nitro gives you a server runtime alongside the browser bundle, so the same project can grow full-stack APIs and server logic without leaving the framework. **Bun** is pinned in [`.prototools`](.prototools) as the JS/TS runtime and package manager for this stack.

- 🟢 [Bun](https://bun.sh/) ([documentation](https://bun.sh/docs)) — JS/TS runtime, package manager, and workspace resolver for `apps/app` and shared packages
- ⚛️ [SolidStart](https://start.solidjs.com/) ([documentation](https://docs.solidjs.com/solid-start)) — full-stack app framework for `apps/app`
- 🧩 [SolidJS](https://www.solidjs.com/) ([documentation](https://docs.solidjs.com/)) — reactive UI library
- 🔀 [Solid Router](https://docs.solidjs.com/solid-router/) — routing for Solid apps
- ⚡ [Vite](https://vite.dev/) ([guide](https://vite.dev/guide/)) — dev server and build tooling
- 🔥 [Nitro](https://nitro.build/) ([guide](https://nitro.build/guide)) — server runtime used by SolidStart
- 🎨 [Tailwind CSS v4](https://tailwindcss.com/) ([documentation](https://tailwindcss.com/docs)) — utility CSS used by `@luna/ds` (consumed from the interactive app and the static site pipeline)

### Static site generator stack (Hugo)

The `apps/web` project is a static-site workflow: Markdown and front matter in `src/content/`, [Go HTML templates](https://gohugo.io/templates/) in `src/layouts/`, and the published site in **`dist/`**. **`@luna/ds`** styles are compiled by the **[Tailwind CSS v4](https://tailwindcss.com/)** CLI into `src/assets/css/bundle.css` before each build (same design system as `apps/app`). **Go** is pinned in [`.prototools`](.prototools); the **Hugo** CLI version is pinned in [`apps/web/go.mod`](apps/web/go.mod) as a [`go tool`](https://go.dev/doc/go1.24#tools) (same idea as Python deps living under `apps/api`). See [apps/web/README.md](apps/web/README.md).

- 📰 [Hugo](https://gohugo.io/) ([documentation](https://gohugo.io/documentation/)) — static site generator; HTML, RSS, and sitemap from content + templates
- ✍️ [Goldmark](https://github.com/yuin/goldmark) — CommonMark-compatible Markdown (Hugo’s default renderer)
- 🧩 [Go HTML templates](https://gohugo.io/templates/) — partials, blocks, and `baseof` layouts under `src/layouts/`
- 🎨 [Tailwind CSS v4](https://tailwindcss.com/) ([documentation](https://tailwindcss.com/docs)) — utility CSS for **`@luna/ds`** ([package](packages/ds/README.md)); **`@tailwindcss/cli`** emits `src/assets/css/bundle.css` (same tokens as `apps/app`)
- 🖍️ [Chroma](https://github.com/alecthomas/chroma) — syntax highlighting for fenced code blocks (`[markup.highlight]` in [`hugo.toml`](apps/web/hugo.toml))

### API service stack (FastAPI and Pydantic)

The `apps/api` stack centers on **FastAPI**, **Pydantic**, and **Pydantic AI** for a **pure backend** HTTP API: validation, settings, and agent-style features stay on the server. The same patterns extend to larger deployments (multiple services, workers, or runtimes) when you outgrow a single process; this repo keeps one API project as the starting point.

- 🐍 [Python](https://www.python.org/) — runtime (version pinned in [`.prototools`](.prototools))
- 📦 [uv](https://docs.astral.sh/uv/) — environments and lockfiles
- 🚀 [FastAPI](https://fastapi.tiangolo.com/) — API framework
- 🤖 [Pydantic AI](https://ai.pydantic.dev/) — AI agent patterns on the backend
- ✅ [Pydantic](https://docs.pydantic.dev/) — schemas and models
- ⚙️ [pydantic-settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/) — environment-driven settings
- 🌐 [Uvicorn](https://uvicorn.dev/) — ASGI server

### Workspace quality (TypeScript and OXC)

- 🧹 [OXC (`oxlint` + `oxfmt`)](https://oxc.rs/) ([documentation](https://oxc.rs/docs/guide/usage/linter.html)) — linting and formatting for JS/TS at the repo root
- 🟦 [TypeScript](https://www.typescriptlang.org/) ([documentation](https://www.typescriptlang.org/docs/)) — static typing and project references across workspaces

## Quick Start

Run commands from the repository root unless an app README says otherwise.

**Prerequisites:** install [Proto](https://moonrepo.dev/docs/proto/install) so `proto` is on your `PATH`. Proto installs everything else — moon, rust, bun, python, go — from the pins in [`.prototools`](.prototools).

### First-time install

The `luna` binary doesn't exist on a fresh clone — it must be built first. Bootstrap with proto and moon directly:

```sh
proto install                  # install moon, rust, bun, python, go (pinned in .prototools)
moon run cli:build             # compile the luna CLI binary (also runs bun install for node_modules)
```

After that, `luna` is available and finishes the remaining install:

```sh
luna install       # proto install + bun install (TS workspaces) + moon builds (cli, web, api)
luna dev         # all application-layer dev tasks (see moon query / app READMEs for subsets)
```

Subsequent runs only need `luna install` — it re-runs all bootstrap steps (including `moon run cli:build`). New pins in [`.prototools`](.prototools) are picked up by **`proto install`** (or **`luna update`**, which refreshes pins then re-runs the same bootstrap steps). [`.moon/toolchains.yml`](.moon/toolchains.yml) disables **`javascript.installDependencies`** (Moon skips redundant `bun install` before each task) and sets **`bun.installArgs: ["--ignore-scripts"]`** as a safeguard.

For a full compile of every application project first, run **`luna build`**.

**Local dev ports** (copy [`.env.example`](.env.example) → `.env.local` to customize): **FastAPI / Uvicorn `8000`** (common tutorial default — avoids stealing **`3000`** from the SPA), **SolidStart `3000`**, **Hugo static site `3001`**. Details and CORS URLs are in each app README.

## Workspaces

- **`apps/api/`** — FastAPI + Pydantic AI · [README](apps/api/README.md)
- **`apps/app/`** — SolidStart (SSR, Vite, Nitro) · [README](apps/app/README.md)
- **`apps/web/`** — Hugo + Tailwind v4 + `@luna/ds` · [README](apps/web/README.md)
- **`packages/cli/`** — `luna` CLI (Rust + Starbase; orchestrates Moon/Proto/Bun commands; `outdated`/`update` across all toolchains)
- **`packages/ds/`** — design system / Tailwind · [README](packages/ds/README.md)
  - **Entrypoints**: `packages/ds/src/tailwind.css` (main import), `packages/ds/src/components.css`, `packages/ds/src/layouts.css`, `packages/ds/src/primitives.css`
  - **Modules**: `packages/ds/src/{components,layouts,primitives}/*.css` (authored as modular CSS)
  - **Patterns**: scoped root + nested layers (`@scope` + `@layer base|variants|patterns`); see [DS README](packages/ds/README.md#scoped-layers-pattern-all-modules)
- **`packages/ui/`** — shared Solid UI · [README](packages/ui/README.md)

### Design system CSS (DS)

When updating `packages/ds`, follow the **scoped root + nested layers** CSS module pattern (`@scope` + `@layer base|variants|patterns`). The DS README is the source of truth: [DS CSS patterns](packages/ds/README.md#scoped-layers-pattern-all-modules).

Moon wires build and dev tasks per project; **`api:dev`** depends on **`api:build`** (`uv sync`) so first-time dev pulls the API venv. `luna install` runs **`api:build`** after installs so the Python workspace is ready before **`luna dev`**. For step-by-step task graphs (`luna build`, Uvicorn, SolidStart), follow each workspace README above.

## Commands

All day-to-day commands go through the **`luna` CLI** (`packages/cli`). It orchestrates Moon, Proto, and Bun directly — there are no root `package.json` scripts to remember.

### Bootstrap and lifecycle

```sh
luna install       # proto install + bun install (TS workspaces) + moon builds (cli, web, api)
luna clean       # moon :clean + moon clean --all + git clean .cache .moon/cache node_modules
luna dev         # moon run :dev --query "projectLayer=application"
luna build       # moon run :build --query "projectLayer=application"
luna start       # moon run :start --query "projectLayer=application"
luna test        # moon run :test --query "projectLayer=application"
```

All build/dev/start/test commands accept an optional project name (`luna build app`) and `--affected` flag (`luna build --affected`).

### Code quality

`luna` orchestrates quality across **all stacks** in one command — TS (oxlint/oxfmt/tsc), Python (ruff via moon), Rust (cargo clippy/fmt), and Go (hugo config via moon):

```sh
luna lint             # TS: oxlint, Python: ruff check, Rust: clippy
luna format           # TS: oxfmt, Python: ruff format, Rust: cargo fmt
luna typecheck        # TS: tsc --build, Go: hugo config
luna check            # lint + format:check + typecheck (all stacks)

luna lint --fix       # apply fixes (oxlint --fix, ruff --fix, clippy --fix)
luna format --check   # check only (oxfmt --list-different, ruff --check, fmt --check)
luna fix              # lint:fix + format (all stacks)
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
moon run :typecheck --query "projectLayer=library"   # shared packages (per-project inherited tasks)
moon query projects --help                           # filters: --id, --language, --layer, etc.
```

Examples for **shared packages** (see each package README for inherited tasks):

```sh
moon run ds:typecheck
moon run ui:typecheck
```

## Configuration map

- Tool/version pins: [`.prototools`](.prototools)
- Workspace manifest + dev dependencies: [`package.json`](package.json) — Bun workspaces and dev tool versions only; all orchestration goes through `luna`
- Moon workspace graph + VCS: [`.moon/workspace.yml`](.moon/workspace.yml)
- Moon toolchains: [`.moon/toolchains.yml`](.moon/toolchains.yml) — `javascript.installDependencies: false`, `bun.installArgs: ["--ignore-scripts"]`; bootstrap with **`luna install`**. After changing JS deps, run **`bun install`** or **`luna install`**.
- Shared TS app tasks: [`.moon/tasks/ts-app.yml`](.moon/tasks/ts-app.yml) (`language: typescript`, `layer: application`, `stack: frontend`)
- Shared TS lib tasks: [`.moon/tasks/ts-lib.yml`](.moon/tasks/ts-lib.yml) (`language: typescript`, `layer: library`)
- Shared Python API tasks: [`.moon/tasks/py-api.yml`](.moon/tasks/py-api.yml) (`language: python`, `stack: backend`)
- Shared Go web tasks: [`.moon/tasks/go-web.yml`](.moon/tasks/go-web.yml) (`language: go`, `stack: frontend`) — **`go tool hugo`** from [`apps/web/go.mod`](apps/web/go.mod)
- Shared Rust bin tasks: [`.moon/tasks/rs-bin.yml`](.moon/tasks/rs-bin.yml) (`language: rust`) — build/test/lint/format-check/clean via `cargo`

- Root moon config: [`moon.yml`](moon.yml)
- TypeScript project graph: [`tsconfig.json`](tsconfig.json)
- Shared TypeScript options: [`tsconfig.options.json`](tsconfig.options.json)
- OXC formatter config: [`.oxfmtrc.json`](.oxfmtrc.json)
- OXC linter config: [`.oxlintrc.json`](.oxlintrc.json)

## Dependency maintenance

Repo-wide **outdated checks** and **upgrades** go through the **`luna` CLI** so every toolchain stays in sync:

The `luna` CLI (`packages/cli`, built with Rust + Starbase) reports outdated **proto pins**, **Bun workspace** packages, **Python / uv** lockfile upgrades (dry-run), and **Go** modules (`language: go` + `go.mod`, plus optional `GO_MODULE_ROOT`) (`luna outdated`). Tool-only Go modules (e.g. Hugo via `go tool` in `apps/web`) are checked with fast `go list -m -u` on `tool` lines; modules with local Go code use `go get -n -u all`. `luna outdated` **always** exits **1** if any tier or project has upgrades (CI-friendly). `luna update` refreshes each discovered project. After `luna update`, review diffs and run `luna check` before committing.

```sh
luna outdated      # report + summary; exit 1 if anything is outdated
luna update        # bump pins and dependencies repo-wide; then review and run luna check
luna update --major  # also apply major-version bumps where supported
```

**Per stack (manual add / remove)** — use these when you are changing one project, not refreshing everything:

- **Toolchain (proto)** — edit [`.prototools`](.prototools), then `luna install` or `proto install` individually. Removing a tool line drops it from proto’s install set for this repo.
- **Hugo (`apps/web`)** — `luna update` bumps the `tool` line with `go get -u=patch` (or `go get -tool …@latest` with `luna update --major`). To pin a specific release manually: `cd apps/web` then `go get -tool github.com/gohugoio/hugo@vX.Y.Z` (updates [`apps/web/go.mod`](apps/web/go.mod) / `go.sum`). Set `LUNA_GO_FULL_GRAPH=1` to restore slow full-graph `go get -u all` on tool-only modules.
- **Bun / workspaces** — from the repo root, add to a workspace with `bun add <pkg> --cwd apps/app` (or `--cwd packages/ui`, etc.); use `bun add -d <pkg> --cwd <path>` for devDependencies. Remove with `bun remove <pkg> --cwd <path>`. Root-only deps: `bun add <pkg>` at the root.
- **Python (`apps/api`)** — `cd apps/api` then `uv add <package>` / `uv remove <package>` (updates `pyproject.toml` and `uv.lock`); sync with `uv sync`.

## Troubleshooting

### Port already in use (`EADDRINUSE`)

Another process is still bound to the port (often after stopping a dev server).

```sh
lsof -i :8000    # API (Uvicorn) default; :3000 app; :3001 web
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
