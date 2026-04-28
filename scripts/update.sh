#!/usr/bin/env bash
# Update toolchain pins, JS deps, Python (uv) lockfiles, and Go modules across the monorepo.
# Order: proto + Bun first (template primary stack), then uv, then Go.
#
# Usage:
#   ./scripts/update.sh
#
# Afterward: review diffs, run tests / bun run check, and commit.

set -euo pipefail

_SCRIPTS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/helpers.sh
source "$_SCRIPTS/helpers.sh"

section "current outdated snapshot"
"$_SCRIPTS/outdated.sh" || true

section "proto — write latest tool versions to .prototools"
require_cmd proto
proto outdated --update --latest -y

section "proto — install pins from .prototools"
proto install

section "sync root packageManager with .prototools bun pin"
sync_root_package_manager_bun

section "Bun — bump workspace deps to latest semver"
require_cmd bun
(cd "$ROOT" && bun update --latest --recursive)

section "Python / uv — refresh lockfile(s) and sync venv(s)"
require_cmd uv
(cd "$UV_PROJECT_ROOT" && uv lock --upgrade && uv sync)

section "Go — upgrade module graph"
require_cmd go
(
	cd "$GO_MODULE_ROOT"
	go get -u ./...
	go get -u github.com/a-h/templ/cmd/templ
	go get -u all
	go mod tidy
)

section "done — verify with bun run outdated and bun run check"
echo "Update steps finished. Review changes before committing."
