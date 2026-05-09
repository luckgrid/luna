# `@luna/cli`

`luna` is a Bun-native monorepo CLI: a single [`src/main.ts`](src/main.ts) is both the **`bin` entry** (shebang + `import.meta.main`) and the router. Shared helpers under [`src/lib/`](src/lib) (`repo`, `process`, `term`, `format`) and toolchain helpers in [`src/lib/toolchains.ts`](src/lib/toolchains.ts) (`proto`, `bun`, `uv`).

## Commands

| Command         | Description                                                                                                                                                       |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `luna outdated` | Print per-toolchain outdated sections when relevant, then a pass/fail summary. **Exits 1** if proto pins, Bun workspaces, or uv lock have upgrades (CI-friendly). |
| `luna update`   | Refresh proto pins, Bun workspaces, uv lock + sync, then root `bun run setup` (proto + workspaces + `moon run api:build`).                                        |

Root shortcuts: `bun run outdated`, `bun run update`.

Global flags: `-h` / `--help`, `-v` / `-V` / `--version`.

## Python projects (scaling)

- **Moon**: `luna outdated` / `luna update` discover all **`language: python`** projects via `moon query projects --language python` (same graph as `.moon/workspace.yml`).
- **Fallback**: if `moon` is missing or returns nothing useful, the CLI scans `apps/*` and `packages/*` for `moon.yml` + `pyproject.toml`.
- **Extras**: `UV_PROJECT_ROOT` adds **one** additional directory (e.g. a tool outside `apps/`), merged with discovered roots.

## Compile (optional)

From this package: `bun run build` → standalone binary under `dist/` (ignored by git).

## Roadmap

- **More top-level commands** — e.g. `clean`, `setup`, `add`, `run`, `build`, `dev` (thin wrappers over moon/bun/proto as needed).
- **`--quiet` for `outdated` / `update`** — Less banner noise or machine-readable output; needs a shared verbosity flag through `commands/*`, `lib/term`, `lib/format`, and `lib/toolchains.ts`.
