# `cli`

Rust CLI orchestrator for Luna, backed by [Moon](https://moonrepo.dev) and [Proto](https://moonrepo.dev/docs/proto).

## Purpose

Single-entry CLI for all monorepo operations — build, dev, test, lint, format, dependency management. Delegates to Moon for task execution and caching, and to Proto for toolchain pinning.

## Stack

- 🦀 [Rust](https://www.rust-lang.org/) — CLI runtime
- 🌙 [Moon](https://moonrepo.dev) — task orchestration, caching, project management
- 📦 [Proto](https://moonrepo.dev/docs/proto) — toolchain version pinning (`.prototools`)
- 📦 [Starbase](https://github.com/moonrepo/starbase) — CLI framework (async runtime, diagnostics, logging)

See root [README Tech Stacks](../../README.md#tech-stacks) for toolchain details.

## Commands

| Command                 | Description                                                                                                      |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `luna build`            | Run application-layer build tasks (`moon run :build`)                                                            |
| `luna build <project>`  | Build a specific project (`moon run <project>:build`)                                                            |
| `luna build --affected` | Build affected projects only                                                                                     |
| `luna dev`              | Start dev servers (`moon run :dev`)                                                                              |
| `luna start`            | Start production servers (`moon run :start`)                                                                     |
| `luna test`             | Run tests (`moon run :test`)                                                                                     |
| `luna run <targets...>` | Run Moon targets directly                                                                                        |
| `luna graph`            | Display project graph (`moon project-graph`)                                                                     |
| `luna tasks`            | List all Moon tasks                                                                                              |
| `luna projects`         | List all Moon projects                                                                                           |
| `luna ci`               | Run affected tasks in CI (`moon ci`)                                                                             |
| `luna install`          | Bootstrap workspace (proto + CLI + bun + uv sync + sync Go pins from `.prototools` + go work sync + web setup)   |
| `luna clean`            | Apps/packages → `moon clean --all` → root outputs (full reset for re-bootstrap)                                  |
| `luna lint`             | Lint all stacks (oxlint, ruff, cargo clippy)                                                                     |
| `luna lint --fix`       | Apply lint fixes                                                                                                 |
| `luna format`           | Format all stacks (oxfmt, ruff, cargo fmt)                                                                       |
| `luna format --check`   | Check formatting without writing                                                                                 |
| `luna typecheck`        | Typecheck all stacks (tsc, hugo)                                                                                 |
| `luna check`            | Lint + format:check + typecheck                                                                                  |
| `luna fix`              | Lint:fix + format                                                                                                |
| `luna outdated`         | Probe proto/rust/bun/uv/go in parallel, print one grouped table, cache a snapshot (exits 0)                      |
| `luna update`           | Snapshot-first: reuse a `< 8h` snapshot (else preflight), update only outdated toolchains, then re-run bootstrap |
| `luna update --major`   | Also apply major-version bumps                                                                                   |

## Dependency management

`luna outdated` and `luna update` share a planner (`src/deps/`) that probes every
toolchain in parallel behind a Luna-owned status panel, normalizes results into a
common row model, and renders one grouped table with toolchain divider rows.

- **Snapshot** — `luna outdated` always overwrites `.cache/outdated.snapshot.json`
  (atomic write). `luna update` reuses it only when it is `< 8h` old and the repo
  root, policy flags, schema, and manifest fingerprints all still match; otherwise
  it preflights the same probe phase first.
- **Selective updates** — `luna update` only runs package managers for toolchains
  the snapshot marks outdated; the rest show as skipped (`—`).
- **Release Age** — the Newest/Latest publish ages are looked up from the npm and
  PyPI registries. Newest is green when `≥ LUNA_MIN_RELEASE_AGE` days old (default
  14, supply-chain cooldown), red when younger; Latest is yellow when exactly one
  major ahead of Current. Set `LUNA_MIN_RELEASE_AGE=0` to disable the cooldown.

## Global flags

- `-v, --verbose` — increase logging verbosity (`--log debug` / `--log trace`)
- `-q, --quiet` — silence Luna and Moon output (`moon -q`)

## Local Development

**First-time install** (from repo root; requires proto + moon on PATH):

```sh
moon run luna:install
```

Or with `luna` already on PATH: `luna install`.

CLI only:

```sh
moon run cli:build      # target/debug/luna
moon run cli:install    # ~/.cargo/bin/luna
```

Build without installing:

```sh
./target/debug/luna --help
```

Run tests:

```sh
moon run cli:test
```

## Architecture

The CLI uses a simple module structure:

- **cli.rs** — Clap-based command-line interface (subcommands, args, help)
- **commands/** — Command implementations
  - **moon.rs** — Moon task wrappers (build, test, dev, start, graph, tasks, projects, ci)
  - **scripts.rs** — Quality commands (install, clean, lint, format, typecheck, check, fix)
  - **outdated.rs** — Consumes the shared planner; renders the grouped table + snapshot
  - **update.rs** — Snapshot-first; updates only outdated toolchains, then bootstraps
- **deps/** — Shared dependency planner
  - **mod.rs** — Planner entrypoint, parallel orchestration, toolchain discovery
  - **model.rs** — `DependencyRow`, `ToolchainSnapshot`, `SnapshotPolicy`
  - **snapshot.rs** — Schema, atomic read/write, validation, manifest fingerprints
  - **registry.rs** — npm/PyPI release-age lookups (cached, best-effort)
  - **ui.rs** — Live panel, divider-row tables, color rules, Release Age footer
  - **probes/** — Per-ecosystem silent outdated probes (proto, bun, uv, cargo, go)
- **runner.rs** — Process execution (run, capture, ensure_installed)
- **workspace.rs** — Root discovery, project detection (Moon + fallback scanning)
- **session.rs** — Starbase session wrapper

Commands dispatch from `commands/mod.rs` → individual command modules → `runner` for subprocess calls.

## Moon Tasks

| Task          | Purpose                                               |
| ------------- | ----------------------------------------------------- |
| `cli:build`   | Compile the CLI (`cargo build` → `target/debug/luna`) |
| `cli:install` | Install to `~/.cargo/bin` (`cargo install`)           |
| `cli:test`    | Run unit and integration tests                        |
| `cli:check`   | Run `cargo check` (via inherited task)                |
| `cli:clippy`  | Run `cargo clippy`                                    |
| `cli:fmt`     | Run `cargo fmt`                                       |

```sh
moon run cli:build
moon run cli:test
moon run cli:check
```
