# Luna Development Guide

Keep this file lean and directive-focused. Use `README.md` as the source of truth for detailed commands and workflows.

## Core Rules

- Polyglot monorepo: each stack brings its own runtime (Bun for JS/TS, Go for Hugo, Python for FastAPI, Rust for the CLI). Proto pins and installs them all — no global installs needed beyond Proto.
- Toolchain versions are pinned in `.prototools`; install **Proto** and **Moon** first. On a fresh clone run `moon run luna:install` (see README Quick Start), then use `luna install` for subsequent runs.
- Run commands from the repository root unless an app/package README says otherwise.
- `luna` is the single entry point for all orchestration: bootstrap, quality, builds, and dep management. `package.json` only holds the Bun workspace manifest and dev dependency versions — no scripts.

## Quick References

- Stacks (tooling and documentation links): [`README.md` (Tech Stacks)](README.md#tech-stacks)
- Proto / Bun / Moon pins: [`README.md` (Core monorepo and toolchain)](README.md#core-monorepo-and-toolchain)
- Workspace paths and per-project READMEs: [`README.md` (Workspaces)](README.md#workspaces)
- Root scripts, quality checks, and moon targets or queries: [`README.md` (Commands)](README.md#commands)
- Setup flow: [`README.md` (Quick Start)](README.md#quick-start)
- Config file map: [`README.md` (Configuration map)](README.md#configuration-map)
- `outdated` / `update` (CLI) and manual add-remove per stack: [`README.md` (Dependency maintenance)](README.md#dependency-maintenance)
- Ports and stuck processes: [`README.md` (Troubleshooting)](README.md#troubleshooting)

## Key Paths

- Toolchain pins: [`.prototools`](.prototools)
- Policy / intent: [`luna.toml`](luna.toml); Luna state: `.luna/`
- Pixi workspace: [`pixi.toml`](pixi.toml)
- Root manifest/dev dependencies: [`package.json`](package.json) — Bun workspace manifest only; all scripts removed in favor of `luna`
- Python workspace (uv): [`pyproject.toml`](pyproject.toml) — virtual root + [`uv.lock`](uv.lock) + [`.venv`](.venv); members `apps/api`, `packages/py-demo`
- Go workspace: [`go.work`](go.work) — members `apps/web`, `packages/go-demo`; commit `go-demo` sources (never bare `git clean` in `go-lib` — it deleted untracked files)
- Rust workspace: [`Cargo.toml`](Cargo.toml) — member `packages/cli`
- Repo-wide outdated / update: [`packages/cli`](packages/cli) — Rust CLI built with Starbase + Clap; `luna outdated` / `luna update` delegate to proto/bun/uv/go per toolchain
- Moon workspace/toolchains/tasks: [`.moon/`](.moon/)
- TypeScript project references: [`tsconfig.json`](tsconfig.json), [`tsconfig.options.json`](tsconfig.options.json)
- OXC config: [`.oxlintrc.json`](.oxlintrc.json), [`.oxfmtrc.json`](.oxfmtrc.json)

## Workspaces

### Apps workspace (`apps/*`)

- **Follow the app README first**: each app owns its ports, env files, and dev/build/start commands.
- **Keep app-specific scripts local**: add orchestration to root/moon only when it benefits multiple apps.

### Packages workspace (`packages/*`)

- **Treat packages as shared infrastructure**: keep APIs stable and changes composable across apps.
- **Validate changes**: use package-level tasks (e.g. `moon run ds:typecheck`, `moon run ui:typecheck`) before relying on app builds.

### `packages/ds` (design-system) CSS guardrails

- **Follow the DS module pattern**: start with a single root `@scope`, then put module layers **inside** it. DS docs: `packages/ds/README.md` (“Scoped layers pattern” + “Complex modules”).
- **Prefer these layers**: `base` first; add `variants` / `patterns` only when needed (avoid ad-hoc layer names).
- **Use nested scopes for complex subtrees**: if a component has deep structure (items, triggers, placement variants), add a nested `@scope (...) { :scope { … } }` for that subtree.
- **Keep state/modifiers co-located**: place hover/open/active rules next to the base block they modify; for trigger-focused state, prefer parent selectors like `:is([open], [data-open]) &`.
- **Keep small one-offs inline**: avoid extra nested scopes for simple descendants with a couple declarations.

### `packages/ui` (Solid UI) guardrails

- **Prefer DS tokens/utilities**: base look-and-feel should come from `@luna/ds`, not per-component CSS drift.
- **Keep components small and composable**: avoid baking app-specific layout/content into shared UI components.

### `packages/cli` (Rust CLI) guardrails

- **`luna.toml` is authoritative** for policy/intent; run `luna migrate`/`init` on legacy clones. Native manifests/lockfiles stay authoritative for package graphs.
- **Direct orchestration**: `luna` calls tools via backend adapters (pixi, proto, moon, bun, uv, cargo, go) — no `bun run` indirection.
- **Pixi / Proto precedence**: Proto pins language runtimes (`.prototools`); Pixi owns the shared dev environment (`pixi.toml`). When Pixi is missing, `ensure_pixi` installs via Proto-pinned `cargo install --git … pixi` if `[bootstrap].auto_install_pixi = true`.
- **Planner + execution modes**: `sync`/`build`/`test`/`ci` resolve plans then execute via adapters; honor `--dry-run`, `--mode inspect|plan|apply|offline|networked`, `--locked`/`--frozen`.
- **Moon compat backend**: task graph via Moon adapter when `[compat.moon].enabled`; scope from `[commands.*].default_scope`, not hardcoded queries.
- **`luna` owns quality across all stacks**: `luna lint`/`format`/`typecheck`/`check`/`fix` cover TS (oxlint/oxfmt/tsc), Python (ruff at root), Rust (cargo clippy/fmt/nextest), and Go (hugo config via moon).
- **`outdated`/`update` manage toolchains** (proto, cargo, bun, uv, go); Pixi is env-only, not in the 5-toolchain outdated set.
- **Lock ledger + SBOM**: `luna lock` writes `.luna/lock-ledger.json`; `luna sbom` exports inventory (`--json`, `--format cyclonedx`).
- **Agent / MCP**: `luna agent mcp` (stdio JSON-RPC, gated on `[agent].mcp`) exposes plan/doctor/config/sbom over internal APIs.
- **Proto pins are source of truth**: `luna install` / `luna update` sync `go.work` and workspace `go.mod` `go` directives from [`.prototools`](.prototools); when Pixi is inactive, subprocesses prepend `~/.proto/shims` and set `UV_PYTHON`.
- **Workspace bin resolution**: `runner::run` prepends `node_modules/.bin` and `~/.cargo/bin` to PATH when Pixi env is not active.
