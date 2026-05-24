# `@luna/cli`

`luna` is a Bun-native monorepo CLI: a single [`src/main.ts`](src/main.ts) is both the **`bin` entry** (shebang + `import.meta.main`) and the router. Shared code under [`src/lib/`](src/lib): toolchain modules [`proto.ts`](src/lib/proto.ts), [`bun.ts`](src/lib/bun.ts), [`py.ts`](src/lib/py.ts), [`go.ts`](src/lib/go.ts), [`moon.ts`](src/lib/moon.ts), and [`utils.ts`](src/lib/utils.ts) (process spawn, repo root, terminal UI). Command implementations live in [`src/commands/`](src/commands/) (`outdated/*`, `update/*`, `help.ts`, `version.ts`). [`.prototools`](../../.prototools) pins **proto**, **moon**, **bun**, **python**, and **go**; `luna outdated` / `luna update` refresh those pins through `proto`. The **Hugo** CLI for `apps/web` is versioned in [`apps/web/go.mod`](../../apps/web/go.mod) as a **`go tool`**, not as a proto pin.

## Commands

| Command               | Description                                                                                                                                                                                                                                                                                                 |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `luna outdated`       | Print per-toolchain outdated sections when relevant, then a pass/fail summary. **Exits 1** if proto pins (proto, moon, bun, python, go), Bun workspaces, uv lock, or Go modules have upgrades (CI-friendly). Tool-only Go modules use `go list -m -u` on `tool` lines; code modules use `go get -n -u all`. |
| `luna update`         | Refresh proto pins **within manifest constraints**, Bun workspaces **within semver ranges** (no major bumps), uv lock + sync, Go modules (tool-only: `go get -u=patch` per tool; code: `go get -u all`), then root `bun run setup`. Safe default for CI/dev.                                                |
| `luna update --major` | Same as `luna update` but also applies **major-version bumps** (`bun update --latest`, `proto outdated --update --latest`) and runs the prerelease catch-up step.                                                                                                                                           |

Root shortcuts: `bun run outdated`, `bun run update` (no major). For majors, run `bunx luna update --major` (or add a `update:major` script).

Global flags: `-h` / `--help`, `-v` / `-V` / `--version`.

> **uv note:** `uv lock --upgrade && uv sync` is run in both modes. uv has no native "no major" toggle — major bumps are governed by the version specifiers in each project's `pyproject.toml` (e.g. `>=1.2,<2`). Tighten constraints there if you need to block majors for Python deps.

## Go modules (Hugo / `go tool`)

- **Moon**: same discovery as Python, filtered by `language: go` and `go.mod` (e.g. `apps/web`).
- **Tool-only** (no local `.go` packages, only `tool` lines in `go.mod`): `luna outdated` runs `go list -m -u` on each tool path (~1s); `luna update` runs `go get -u=patch` per tool (not `go get -u all` across Hugo’s transitive graph).
- **Code modules**: full-graph `go get -n -u all` / `go get -u all` plus `go build ./...` when packages exist.
- **`LUNA_GO_FULL_GRAPH=1`**: force the legacy full-graph probe/update on tool-only modules.

## Python projects (scaling)

- **Moon**: `luna outdated` / `luna update` discover all **`language: python`** projects via `moon query projects --language python` (same graph as `.moon/workspace.yml`).
- **Fallback**: if `moon` is missing or returns nothing useful, the CLI scans `apps/*` and `packages/*` for `moon.yml` + `pyproject.toml`.
- **Extras**: `UV_PROJECT_ROOT` adds **one** additional directory (e.g. a tool outside `apps/`), merged with discovered roots.

## Compile (optional)

From this package: `bun run build` → standalone binary under `dist/` (ignored by git).

## Roadmap

- **More top-level commands** — e.g. `clean`, `add`, `run`, `build`, `dev` (thin wrappers over moon/bun/proto as needed).
- **`--quiet` for `outdated` / `update`** — Less banner noise or machine-readable output; needs a shared verbosity flag through `src/commands/*` and `lib/utils.ts`.
