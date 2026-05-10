# Luna

Luna is a polyglot monorepo starter built around **moonrepo** for the task graph, **proto** and [`.prototools`](.prototools) for pinned runtimes (Bun, Python, moon), and **Bun workspaces** for JavaScript and TypeScript packages. The template layers three application stacks on that foundation: a **SolidStart** interactive app with SSR and streaming, a **Bun + Vite** static site generator for HTML and Markdown, and a **FastAPI** service with Pydantic and Pydantic AI. Shared libraries live under `packages/*`. Use each app or package README for ports, env files, and moon task details.

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

### Static site generator stack (Bun and Vite)

The `apps/web` stack is a **file-based static site generator**: a small **Bun** / TypeScript build writes HTML into `dist/`, using **[`Bun.markdown`](https://bun.com/docs/runtime/markdown)** for Markdown bodies (after YAML frontmatter) and string templates for layout. **Vite** compiles the Tailwind entry that imports [`@luna/ds`](packages/ds/README.md) into minified **`dist/styles.css`**—same design system as `apps/app`, without routing Markdown through Vite. Dev runs **`vite build --watch`** plus an SSG watcher so `packages/ds` and `src/content/` edits rebuild without restarting the process. For **signal-style** HTML-driven UI you can add [Datastar](https://data-star.dev/) later (optional script hook in the shell template) without a SPA shell.

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
bun run setup    # proto + Bun workspaces + Python venv (api:build / uv sync)
bun run dev      # all application-layer dev tasks (see moon query / app READMEs for subsets)
```

The **`setup`** script (not named `install`) runs `proto install`, then `bun install --ignore-scripts`, then `moon run api:build`. A root script named **`install` is a Bun/npm lifecycle hook**: any `bun install` that runs lifecycle scripts (including Moon’s dependency sync or `bunx`) could recurse into Moon and **hang `dev` / `build`**. Use **`bun run setup`** for full-stack bootstrap; use plain **`bun install`** when you only need workspace `node_modules`. [`.moon/toolchains.yml`](.moon/toolchains.yml) disables **`javascript.installDependencies`** (Moon skips redundant `bun install` before each task) and sets **`bun.installArgs: ["--ignore-scripts"]`** as a safeguard.

For a full compile of every application project first, run **`bun run build`**. Default ports are documented in each app README and can be overridden via [`.env.local`](.env.local).

## Workspaces

- **`apps/api/`** — FastAPI + Pydantic AI · [README](apps/api/README.md)
- **`apps/app/`** — SolidStart (SSR, Vite, Nitro) · [README](apps/app/README.md)
- **`apps/web/`** — Bun SSG + Vite (`@luna/ds`) · [README](apps/web/README.md)
- **`packages/cli/`** — internal `luna` CLI (Bun entry; `deps …` today, `ds …` reserved for design-system / UI tooling) · [README](packages/cli/README.md)
- **`packages/ds/`** — design system / Tailwind · [README](packages/ds/README.md)
  - **Entrypoints**: `packages/ds/src/tailwind.css` (main import), `packages/ds/src/components.css`, `packages/ds/src/layouts.css`, `packages/ds/src/primitives.css`
  - **Modules**: `packages/ds/src/{components,layouts,primitives}/*.css` (authored as modular CSS)
  - **Patterns**: scoped root + nested layers (`@scope` + `@layer base|variants|patterns`); see [DS README](packages/ds/README.md#scoped-layers-pattern-all-modules)
- **`packages/ui/`** — shared Solid UI · [README](packages/ui/README.md)

### Design system CSS (DS)

When updating `packages/ds`, follow the **scoped root + nested layers** CSS module pattern (`@scope` + `@layer base|variants|patterns`). The DS README is the source of truth: [DS CSS patterns](packages/ds/README.md#scoped-layers-pattern-all-modules).

Moon wires build and dev tasks per project; **`api:dev`** depends on **`api:build`** (`uv sync`) so first-time dev pulls the API venv. Root **`bun run setup`** runs **`api:build`** after installs so the Python workspace is ready before **`bun run dev`**. For step-by-step task graphs (`bun run build`, Uvicorn, SolidStart), follow each workspace README above.

## Commands

### Root scripts (Bun)

These scripts target the **application layer** (all `apps/*` projects), matching what you use day to day from the repo root:

```sh
bun run setup       # proto + Bun workspaces + api:build (see Quick Start)
bun run dev         # moon run :dev --query "projectLayer=application"
bun run build       # moon run :build --query "projectLayer=application"
bun run start       # moon run :start --query "projectLayer=application"
bun run clean       # moon :clean (per-project, uncached) + moon clean --all + git clean .cache .moon/cache node_modules
```

### Code quality (Bun)

```sh
bun run lint
bun run format:check
bun run typecheck
bun run check       # lint + format:check + typecheck (includes api moon tasks where applicable)

bun run lint:fix
bun run format
bun run fix         # lint:fix + format (includes api moon tasks where applicable)
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
- Moon toolchains: [`.moon/toolchains.yml`](.moon/toolchains.yml) — `javascript.installDependencies: false`, `bun.installArgs: ["--ignore-scripts"]`; bootstrap with **`bun run setup`** (never a root `install` script — see Quick Start). After changing JS deps, run **`bun install`** or **`bun run setup`**.
- Shared TS app tasks: [`.moon/tasks/ts-app.yml`](.moon/tasks/ts-app.yml) (`language: typescript`, `layer: application`, `stack: frontend`)
- Shared TS lib tasks: [`.moon/tasks/ts-lib.yml`](.moon/tasks/ts-lib.yml) (`language: typescript`, `layer: library`)
- Shared Python API tasks: [`.moon/tasks/py-api.yml`](.moon/tasks/py-api.yml) (`language: python`, `stack: backend`)

- Root moon config: [`moon.yml`](moon.yml)
- TypeScript project graph: [`tsconfig.json`](tsconfig.json)
- Shared TypeScript options: [`tsconfig.options.json`](tsconfig.options.json)
- OXC formatter config: [`.oxfmtrc.json`](.oxfmtrc.json)
- OXC linter config: [`.oxlintrc.json`](.oxlintrc.json)

## Dependency maintenance

Repo-wide **outdated checks** and **upgrades** go through the **`luna` CLI** so every toolchain stays in sync:

The internal CLI [`@luna/cli`](packages/cli) (`luna`) reports outdated **proto pins**, **Bun workspace** packages, and **Python / uv** lockfile upgrades (dry-run) (`luna outdated`). Python targets are **Moon projects** with `language: python` (plus a filesystem fallback and optional `UV_PROJECT_ROOT` for an extra dir). `luna outdated` **always** exits **1** if any tier or project has upgrades (CI-friendly). `luna update` refreshes each discovered project. After `bun run update`, review diffs and run `bun run check` before committing.

```sh
luna outdated      # same as below; report + summary; exit 1 if anything is outdated
luna update        # same as below; bump pins and dependencies repo-wide; then review and run bun run check

bun run outdated   # wraps `luna outdated`
bun run update     # wraps `luna update`; then review and run bun run check
```

**Per stack (manual add / remove)** — use these when you are changing one project, not refreshing everything:

- **Toolchain (proto)** — edit [`.prototools`](.prototools), then `proto install` or `bun run setup` (or `proto pin <tool> <version>`). Removing a tool line drops it from proto’s install set for this repo.
- **Bun / workspaces** — from the repo root, add to a workspace with `bun add <pkg> --cwd apps/app` (or `--cwd packages/ui`, etc.); use `bun add -d <pkg> --cwd <path>` for devDependencies. Remove with `bun remove <pkg> --cwd <path>`. Root-only deps: `bun add <pkg>` at the root.
- **Python (`apps/api`)** — `cd apps/api` then `uv add <package>` / `uv remove <package>` (updates `pyproject.toml` and `uv.lock`); sync with `uv sync`.

## Troubleshooting

### Port already in use (`EADDRINUSE`)

Another process is still bound to the port (often after stopping a dev server).

```sh
lsof -i :8080    # API default; try :3000 for app, :3001 for web
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
