# Luna Tech Debt Roadmap

A documented list of tech debt notes, details, etc... to serve as a roadmap for paying back the tech debt before public release.

## Critical tech debt to solve before production

- Implement low level build and dependency management tools using existing libs to improve cli package and multi-language configs
  - https://github.com/prefix-dev/pixi
    - https://pixi.prefix.dev/latest/
    - https://prefix.dev/tools/pixi
    - https://prefix.dev/tools/rattler-build
  - https://gittup.org/tup/
  - https://vboussange.github.io/post/research-project-dependencies/
- A permanent docs/ workspace/dir should include a collection of docs with detailed content that's rendered as a documentation site (using existing tools and packages - can be a hugo ssg like the web app)
- Create actual useful packages that can be used by go (hugo apps) and py (python apps) in a similar way as the ui and ds packages are used. The go package can contain common layouts, content, and other configs/patterns. The python package can contain backend patterns/tools/configs for pydantic ai, storage, data models, etc.
- Setup an env variable management tool for encryption/decryption of env vars, key vaults, etc (a low level rust tool preferred, use an open source solution or combine open source tools/crates to create a minimal solution that integrates with the cli, or use something like dotenvx which works with multiple toolchains as well).
  - https://docs.rs/secret-vault/latest/secret_vault/
  - https://github.com/Tongsuo-Project/RustyVault
  - https://github.com/Infisical/infisical-cli
  - https://github.com/dotenvx/dotenvx

## Complex tech debt to refactor with deeper research and planning

- Update ds package to dist or provide ds tailwind styles/configs that don't depend on vite and ts (ds should work with hugo/go apps without needing to add package.json and tsconfig in order to run tailwind cli -- ds package can have a build/dev script that can serve the necessary css files to apps, to remove tailwind/ts as a required dependency for using ds) -- can also add/plan tree shaking to update css imports/sources based on what the app needs/uses to reduce the css file size/length
- Update ui package to have reusable html go template layouts, partials, etc (to share between web and future docs apps). ui package should also dist the necessary ui components and markup (with tree shaking of unused components/layouts based on each app) to make the ui usable apps running different stacks and toolchains (i.e. solid/ts, hugo/go, etc) -- integrating the ui with the cli (or low level rust code) to transform/compile ds primitives and ui templates/configs into reusable components, templates, partials, layouts, etc based on which app/package is consuming/using it (i.e. tsx -> solidjs/react/etc, html -> html go, templ, handlebars, etc).

## Risks and mitigations (Pixi/control-plane refactor)

Phases 0–7 of the Luna CLI control-plane refactor are **implemented**. Historical context below; items marked **Done** no longer need action.

### Dual-authority config drift (`luna.toml` vs `pixi.toml` vs `.moon/*` vs `.prototools`) — **Done**

- **Mitigation shipped**: [`luna.toml`](luna.toml) owns policy; `luna config validate` / `luna doctor` surface drift via [`validate_against_repo`](packages/cli/src/config/validate.rs).
- **Done**: runtime legacy fallback retired — `config::load_required` requires `luna.toml`; use `luna migrate` for one-time import from [`compat.rs`](packages/cli/src/config/compat.rs).
- **Done**: `pyproject.toml` workspace members parsed with `toml` in compat (not string scraping).

### Moon compatibility breaks during transition — **Done (compat mode)**

- Moon remains behind `[compat.moon]` and the Moon adapter ([`adapters/moon.rs`](packages/cli/src/adapters/moon.rs)); `luna ci --backend moon` falls back to `moon ci`.
- **Done**: single Moon entry point via `MoonBackend::run_moon_argv`; scope from `[commands.*].default_scope`.

### Pixi + Proto runtime conflict — **Done**

- Pixi-first sync in [`systems/tasks.rs`](packages/cli/src/systems/tasks.rs); `ensure_pixi` via Proto-pinned cargo when `[bootstrap].auto_install_pixi`.
- **Done**: Pixi removed from 5-toolchain outdated set; modeled in adapter layer only.
- **Done**: runner PATH/`UV_PYTHON` gated when Pixi env is active.

### Bespoke per-ecosystem dependency logic — **Partially done**

- **Done**: `BackendAdapter` trait with `export_inventory`, lock ledger, SBOM command.
- **Remaining**: delegate more outdated/update paths to adapter `lock`/`sync`; CycloneDX coverage is basic.

### `.luna/` state isolation — **Done**

- Snapshots, lock-ledger, cache, plans under `.luna/` via [`systems/state.rs`](packages/cli/src/systems/state.rs).

### Agent mutation safety — **Done (v1)**

- Execution modes (`--mode`), plan fingerprints (`plan --out` / `apply`), schema-versioned `--json`.
- Thin MCP stdio server (`luna agent mcp`) gated on `[agent].mcp`.
- **Deferred**: MCP auth/transport hardening.

### Cross-platform / Windows shell drift — **Partial**

- Explicit `program + args` Steps; Windows CI smoke job in [`.github/workflows/ci.yml`](.github/workflows/ci.yml).
- **Remaining**: broader Windows parity testing.

### Remaining / future debt

- CycloneDX SBOM enrichment (purl accuracy, licenses).
- MCP auth and remote transport.
- Full retirement of Moon compat when Luna planner reaches full parity.
- Permanent `docs/` workspace (see Critical tech debt above).
