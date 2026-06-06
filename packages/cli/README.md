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

`luna outdated` and `luna update` share a planner (`systems::deps`) that probes every
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

The CLI follows a Starbase-first modular layout:

```text
cli/src/
├── main.rs            # Binary entry: Starbase App lifecycle → commands::dispatch
├── lib.rs             # Crate root (re-exports Cli, LunaSession)
├── cli.rs             # Clap definitions (Commands enum, GlobalArgs, per-command arg structs)
├── session.rs         # Starbase AppSession wrapper (LunaSession)
├── commands/          # Command dispatch and implementations
│   ├── mod.rs         # dispatch() — async routing from parsed Commands → handler
│   ├── core.rs        # Moon task wrappers (build, test, dev, start, graph, tasks, projects, ci)
│   ├── lifecycle.rs   # Bootstrap/clean commands (install, install --workspace, clean)
│   ├── quality.rs     # Quality commands (lint, format, typecheck, check, fix)
│   ├── outdated.rs    # Probes toolchains via DependencyService, renders grouped table + snapshot
│   └── update.rs      # Snapshot-first update; updates only outdated toolchains, then bootstraps
├── systems/           # Business logic and infrastructure
│   ├── mod.rs
│   ├── deps.rs        # DependencyService — parallel plan/update orchestration (JoinSet + spawn_blocking)
│   ├── tasks.rs       # Shared bootstrap/sync building blocks (install, clean, workspace sync)
│   ├── model.rs       # ToolchainKind, ToolchainState, DependencyRow, ToolchainSnapshot, SnapshotPolicy
│   ├── snapshot.rs    # Schema, atomic read/write, validation, manifest fingerprints
│   ├── registry.rs    # npm/PyPI release-age lookups (cached, best-effort)
│   ├── runner.rs      # Process execution (run, capture, ensure_installed, run_moon, run_pm)
│   ├── security.rs    # Release-age policy, firewall resolution, Socket Firewall wrapping
│   └── workspace.rs   # Root discovery, project detection, go toolchain sync
├── toolchains/        # Per-ecosystem adapters implementing ToolchainAdapter
│   ├── mod.rs         # ToolchainAdapter trait, ProbeOutcome, UpdateOutcome, adapter_for() factory
│   ├── proto.rs       # Proto — proto outdated --json, proto outdated --update
│   ├── cargo.rs       # Rust — cargo outdated --format json, cargo update
│   ├── bun.rs         # Bun — bun outdated --recursive, bun update --recursive
│   ├── uv.rs          # Python — uv lock --upgrade --dry-run, uv lock --upgrade + uv sync
│   └── go.rs          # Go — go list -m -u, go get -tool/@latest or go get -u
└── ui/                # Console rendering and event bridge
    ├── mod.rs         # LunaConsole, new_console, notices, run_with_loader
    ├── events.rs      # Emitter — decouples systems from console rendering
    ├── status.rs      # Live/frozen StatusPanel (iocraft animated panel)
    └── tables.rs      # Outdated/update tables, release-age section, update summary
```

### Command dispatch flow

```mermaid
flowchart TD
    A["luna <command>"] --> B["Cli::parse()"]
    B --> C["Starbase App::run()"]
    C --> D["commands::dispatch(session)"]
    D --> E{Which command?}
    E -->|build/test/dev/start| F["core.rs → run_moon()"]
    E -->|lint/format/typecheck| G["quality.rs → runner::run()"]
    E -->|install/clean| H["lifecycle.rs → tasks::*"]
    E -->|outdated| I["outdated.rs → deps::plan()"]
    E -->|update| J["update.rs → deps::load_snapshot() / plan()"]
    I --> K["ToolchainAdapter::probe() (parallel)"]
    J --> L["ToolchainAdapter::update() (parallel)"]
    K --> M["Emitter → StatusPanel (live)"]
    L --> M
    M --> N["snapshot::write()"]
    N --> O["Console tables + summary"]
```

### Outdated / Update lifecycle

```mermaid
flowchart TD
    A["luna outdated"] --> B["deps::plan() — probe all toolchains in parallel"]
    B --> C["Emitter → live StatusPanel"]
    C --> D["SnapshotPolicy + ToolchainSnapshot[]"]
    D --> E["snapshot::write() → .cache/outdated.snapshot.json"]
    E --> F["Render grouped table + release age section"]

    G["luna update"] --> H{"Valid snapshot < 8h?"}
    H -->|Yes| I["Reuse snapshot toolchains"]
    H -->|No| B
    I --> J["outdated_kinds() — filter to outdated only"]
    J --> K{"Any outdated?"}
    K -->|No| L["✓ All up to date"]
    K -->|Yes| M["deps::update() — Proto first, rest parallel"]
    M --> N["sync_workspace_quiet() — re-bootstrap"]
    N --> O["Render update table + summary"]
```

### Module dependency graph

```mermaid
graph LR
    main["main.rs"] --> lib["lib.rs"]
    main --> session["session.rs"]
    main --> cmd["commands/"]
    main --> ws["workspace"]

    cmd --> core["core.rs"]
    cmd --> lifecycle["lifecycle.rs"]
    cmd --> quality["quality.rs"]
    cmd --> outdated_cmd["outdated.rs"]
    cmd --> update_cmd["update.rs"]

    core --> runner["runner"]
    lifecycle --> tasks["tasks"]
    quality --> runner
    quality --> ws
    outdated_cmd --> deps["deps"]
    update_cmd --> deps
    deps --> adapter["toolchains/*"]
    deps --> snapshot["snapshot"]
    deps --> security["security"]
    deps --> registry["registry"]
    deps --> ws

    adapter --> runner
    adapter --> security
    adapter --> ws

    outdated_cmd --> ui["ui"]
    update_cmd --> ui
    deps --> ui

    ui --> emitter["events (Emitter)"]
    ui --> panel["status (StatusPanel)"]
    ui --> tables["tables"]
```

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

## Resources

### Framework and toolchain docs

- [Starbase](https://github.com/moonrepo/starbase) — Rust application framework (async runtime, diagnostics, logging)
- [starbase_console](https://docs.rs/starbase_console) — Terminal UI and console rendering
- [Clap](https://docs.rs/clap) — CLI argument parser (derive mode)
- [Moon](https://moonrepo.dev/docs) — Task orchestration, caching, project management
  - [Moon project config](https://moonrepo.dev/docs/config/project) — `moon.yml` task definitions
  - [Moon CI](https://moonrepo.dev/docs/guides/ci) — `moon ci` and affected-targets workflow
  - [Moon queries](https://moonrepo.dev/docs/guides/queries) — `moon query projects` for discovery
- [Proto](https://moonrepo.dev/docs/proto) — Multi-language toolchain version pinning (`.prototools`)

### Internal requirement docs

- [CLI Product Requirements (PRD)](../../docs/bin/luna-cli-PRD.md) — High-level feature spec for the refactor
- [CLI Technical Requirements (TRD)](../../docs/bin/luna-cli-TRD.md) — Technical specification (Starbase integration, service extraction)
- [CLI Architecture (ARD)](../../docs/bin/luna-cli-ARD.md) — Layer mapping and dependency overview
- [Rust/Starbase Research](../../docs/bin/luna-cli-rust-starbase-research.md) — Reference guidance on project structure and patterns

### Monorepo docs

- [Root README](../../README.md#tech-stacks) — Tech stacks and toolchain pins
- [Root README — Quick Start](../../README.md#quick-start) — Bootstrap flow
- [Root README — Commands](../../README.md#commands) — Quality checks, moon targets
- [Root README — Dependency Maintenance](../../README.md#dependency-maintenance) — `luna outdated` / `luna update` usage
