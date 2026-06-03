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

| Command                 | Description                                                                     |
| ----------------------- | ------------------------------------------------------------------------------- |
| `luna build`            | Run application-layer build tasks (`moon run :build`)                           |
| `luna build <project>`  | Build a specific project (`moon run <project>:build`)                           |
| `luna build --affected` | Build affected projects only                                                    |
| `luna dev`              | Start dev servers (`moon run :dev`)                                             |
| `luna start`            | Start production servers (`moon run :start`)                                    |
| `luna test`             | Run tests (`moon run :test`)                                                    |
| `luna run <targets...>` | Run Moon targets directly                                                       |
| `luna graph`            | Display project graph (`moon project-graph`)                                    |
| `luna tasks`            | List all Moon tasks                                                             |
| `luna projects`         | List all Moon projects                                                          |
| `luna ci`               | Run affected tasks in CI (`moon ci`)                                            |
| `luna install`          | Bootstrap workspace (proto + CLI + bun + moon builds)                           |
| `luna clean`            | Apps/packages → `moon clean --all` → root outputs (full reset for re-bootstrap) |
| `luna lint`             | Lint all stacks (oxlint, ruff, cargo clippy)                                    |
| `luna lint --fix`       | Apply lint fixes                                                                |
| `luna format`           | Format all stacks (oxfmt, ruff, cargo fmt)                                      |
| `luna format --check`   | Check formatting without writing                                                |
| `luna typecheck`        | Typecheck all stacks (tsc, hugo)                                                |
| `luna check`            | Lint + format:check + typecheck                                                 |
| `luna fix`              | Lint:fix + format                                                               |
| `luna outdated`         | Report outdated toolchains/dependencies (exits 1 if any)                        |
| `luna update`           | Update toolchains and dependencies, re-run install                              |
| `luna update --major`   | Also apply major-version bumps                                                  |

## Aliases (optional)

`cargo install` also installs shorter binary names that are drop-in replacements for `luna` — same subcommands, flags, and arguments:

| Alias | Example   |
| ----- | --------- |
| `lna` | `lna dev` |

After `moon run cli:install`, these are available in `~/.cargo/bin/` alongside `luna` (ensure that directory is on your `PATH`).

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
moon run cli:build      # target/debug/luna (+ lna)
moon run cli:install    # ~/.cargo/bin/luna (+ lna)
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
  - **outdated.rs** — Toolchain/dependency outdated detection
  - **update.rs** — Toolchain/dependency updates
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
