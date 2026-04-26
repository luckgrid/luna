#!/usr/bin/env bash
# Shared helpers for monorepo dependency scripts (sourced by outdated.sh / upgrade.sh).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
export ROOT

# Default roots for uv- and Go-based projects in this template. Override with
# UV_PROJECT_ROOT / GO_MODULE_ROOT when you add more apps or move modules.
UV_PROJECT_ROOT="${UV_PROJECT_ROOT:-$ROOT/apps/api}"
GO_MODULE_ROOT="${GO_MODULE_ROOT:-$ROOT/apps/web}"
export UV_PROJECT_ROOT GO_MODULE_ROOT

# ANSI styling (disabled for NO_COLOR, non-tty stderr, or TERM=dumb).
setup_term_colors() {
	if [[ -n "${NO_COLOR:-}" ]] || [[ "${TERM:-}" == "dumb" ]] || ! [[ -t 2 ]]; then
		C_RED=""
		C_GREEN=""
		C_BOLD=""
		C_DIM=""
		C_RESET=""
	else
		C_RED=$'\033[0;31m'
		C_GREEN=$'\033[0;32m'
		C_BOLD=$'\033[1m'
		C_DIM=$'\033[2m'
		C_RESET=$'\033[0m'
	fi
}

setup_term_colors

section() {
	printf '\n%s== %s ==%s\n' "$C_BOLD" "$1" "$C_RESET"
}

strict_ok() {
	printf '%s✓%s %s\n' "$C_GREEN" "$C_RESET" "$1" >&2
}

strict_need() {
	printf '%s✗%s %s\n' "$C_RED" "$C_RESET" "$1" >&2
}

strict_summary_fail_title() {
	printf '%s%s%s\n' "${C_BOLD}${C_RED}" "$1" "$C_RESET" >&2
}

strict_summary_bullet() {
	printf '  %s•%s %s\n' "$C_RED" "$C_RESET" "$1" >&2
}

strict_hint() {
	printf '%s%s%s\n' "$C_DIM" "$1" "$C_RESET" >&2
}

strict_all_passed() {
	printf '%s%s%s\n' "$C_GREEN" "$1" "$C_RESET"
}

die() {
	echo "error: $*" >&2
	exit 1
}

require_cmd() {
	command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

proto_has_outdated_pins() {
	local json ec
	# Capture JSON first: when all pins are fresh, the Bun step exits 1 on purpose
	# ("no outdated tools"). That must not trigger `|| die` on a pipeline.
	json="$(proto outdated --json 2>/dev/null)" || die "proto outdated --json failed (is proto in PATH?)"
	[[ -n "$json" ]] || die "proto outdated --json returned empty output"
	printf '%s' "$json" | bun -e '
const text = await new Response(Bun.stdin).text();
const data = JSON.parse(text);
if (typeof data !== "object" || data === null) process.exit(2);
const any = Object.values(data).some((x) => x && x.is_outdated === true);
process.exit(any ? 0 : 1);
' 2>/dev/null
	ec=$?
	[[ "$ec" -eq 0 ]] && return 0
	[[ "$ec" -eq 1 ]] && return 1
	die "proto pin check failed (bun exit $ec; is bun in PATH? invalid JSON from proto?)"
}

bun_workspace_outdated() {
	bun outdated 2>&1 | grep -q "| Package  | Current |" || return 1
	return 0
}

uv_lock_has_upgrades() {
	(cd "$UV_PROJECT_ROOT" && uv lock --upgrade --dry-run 2>&1 | grep -q '^Update ') && return 0
	return 1
}

go_mod_has_upgrades() {
	local out
	# Only direct requires in go.mod: transitive deps often show [newer] in
	# `go list -u -m all` forever because MVS does not pull them without an
	# explicit get — that is not what `go get -u ./...` / upgrade.sh fixes.
	out="$(cd "$GO_MODULE_ROOT" && go list -u -m -f '{{if and (not .Main) (not .Indirect) .Update}}{{println .Path}}{{end}}' all 2>/dev/null || true)"
	echo "$out" | grep -q .
}

sync_root_package_manager_bun() {
	local ver line
	line="$(grep -E '^[[:space:]]*bun[[:space:]]*=' "$ROOT/.prototools" | head -1 || true)"
	[[ -n "$line" ]] || {
		echo "warning: no bun= line in .prototools; skip packageManager sync" >&2
		return 0
	}
	ver="$(echo "$line" | sed -E 's/^[[:space:]]*bun[[:space:]]*=[[:space:]]*"?([^"]+)"?.*/\1/')"
	[[ -n "$ver" ]] || return 0
	if command -v jq >/dev/null 2>&1; then
		local tmp
		tmp="$(mktemp)"
		jq --arg pm "bun@${ver}" '.packageManager = $pm' "$ROOT/package.json" >"$tmp"
		mv "$tmp" "$ROOT/package.json"
		echo "Synced root package.json packageManager -> bun@${ver}"
	else
		echo "warning: jq not installed; align package.json packageManager with bun@${ver}" >&2
	fi
}
