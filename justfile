# Luna-owned local recipes (also available via `pixi run` from pixi.toml)

# Watch and restart dev servers via Moon
dev-watch:
    watchexec --watch . -- moon run :dev --query projectLayer=application

# Apply lint fixes across stacks
lint-fix:
    luna fix

# Run Rust tests with nextest when available
test-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo-nextest >/dev/null 2>&1 || cargo nextest --version >/dev/null 2>&1; then
        cargo nextest run
    else
        cargo test
    fi

# Full quality gate
check-all:
    luna check

# Deep clean: Luna clean + pixi cache hint
clean-deep:
    luna clean
    @echo "Tip: remove .pixi/ manually if you need a full Pixi reset"

# Sync Pixi root environment
env-sync:
    luna env sync
