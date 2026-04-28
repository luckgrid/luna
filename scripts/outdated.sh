#!/usr/bin/env bash
# Report outdated toolchain pins (.prototools), Bun workspaces, Python (uv) lockfiles, and Go modules.
#
# Usage:
#   ./scripts/outdated.sh           # print reports only (exit 0)
#   ./scripts/outdated.sh --strict  # exit 1 if any tier reports updates (CI)
#
# Environment:
#   STRICT=1  same as --strict

set -euo pipefail

_SCRIPTS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/helpers.sh
source "$_SCRIPTS/helpers.sh"

STRICT="${STRICT:-0}"
[[ "${1:-}" == "--strict" ]] && STRICT=1

FAILED=0
ST_FAIL_PROTO=0
ST_FAIL_BUN=0
ST_FAIL_UV=0
ST_FAIL_GO=0

section "proto (.prototools — moon, bun, python, go, proto)"
require_cmd proto
proto outdated

section "Bun (workspaces — root package.json + apps/* + packages/*)"
require_cmd bun
(cd "$ROOT" && bun outdated) || true

section "Python / uv (uv-managed projects — pyproject.toml + uv.lock)"
require_cmd uv
(cd "$UV_PROJECT_ROOT" && uv lock --upgrade --dry-run)

section "Go (modules — go.mod / go.sum; [version] can include transitive deps)"
require_cmd go
(cd "$GO_MODULE_ROOT" && go list -u -m all)

if [[ "$STRICT" -eq 1 ]]; then
	section "strict check"
	require_cmd bun
	if proto_has_outdated_pins; then
		strict_need "strict: proto — outdated tool pin(s) (.prototools)"
		ST_FAIL_PROTO=1
		FAILED=1
	else
		strict_ok "strict: proto — OK (.prototools)"
	fi
	if bun_workspace_outdated; then
		strict_need "strict: Bun — outdated direct dependencies in workspaces"
		ST_FAIL_BUN=1
		FAILED=1
	else
		strict_ok "strict: Bun — OK (workspaces)"
	fi
	if uv_lock_has_upgrades; then
		strict_need "strict: Python / uv — lockfile(s) can update (see bun run update)"
		ST_FAIL_UV=1
		FAILED=1
	else
		strict_ok "strict: Python / uv — OK"
	fi
	if go_mod_has_upgrades; then
		strict_need "strict: Go — go.mod require(s) have a newer version"
		ST_FAIL_GO=1
		FAILED=1
	else
		strict_ok "strict: Go — OK (go.mod requires)"
	fi
	section "strict summary"
	if [[ "$FAILED" -eq 0 ]]; then
		strict_all_passed "All strict checks passed (nothing reported as outdated)."
	else
		echo "" >&2
		strict_summary_fail_title "Strict mode failed — upgrades reported in:"
		[[ "$ST_FAIL_PROTO" -eq 1 ]] && strict_summary_bullet "proto (.prototools)"
		[[ "$ST_FAIL_BUN" -eq 1 ]] && strict_summary_bullet "Bun workspaces"
		[[ "$ST_FAIL_UV" -eq 1 ]] && strict_summary_bullet "Python / uv lockfile(s)"
		[[ "$ST_FAIL_GO" -eq 1 ]] && strict_summary_bullet "Go (go.mod requires)"
		echo "" >&2
		strict_hint "Exit code 1 is intentional (use in CI). To refresh everything: bun run update"
		strict_hint "For a non-gated report only: bun run outdated:check"
	fi
	exit "$FAILED"
fi
