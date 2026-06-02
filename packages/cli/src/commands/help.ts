/** Keep in sync with `main.ts` command dispatch. */
export function printRootHelp(): void {
  console.log(`luna — Luna monorepo CLI (Bun)

Usage:
  luna [options] <command>

Commands:
  outdated
      Toolsets: Proto-managed pins (.prototools: proto, moon, bun, python, go);
                Bun workspaces (root + apps/* + packages/*/package.json);
                Python deps via uv lockfiles (Moon \`language: python\` projects + UV_PROJECT_ROOT);
                Go modules (Moon \`language: go\` + go.mod, + GO_MODULE_ROOT): tool-only modules use fast
                \`go list -m -u\` on \`tool\` lines; code modules use \`go get -n -u all\` dry-run. Set
                \`LUNA_GO_FULL_GRAPH=1\` to force the full-graph dry-run on tool-only modules.
      Exits 1 if anything is outdated (CI).
      Writes a best-effort snapshot to \`.cache/outdated-snapshot.json\` for \`luna update\` (cache-first precheck).
      On an interactive terminal, shows a live status line while probes run, then per-tier ✓/✗ lines before tables.
      \`outdated --use-cache\` reuses the cache when the fingerprint still matches (fast; not for CI).
      Set \`LUNA_OUTDATED_PROGRESS=1\` for per-probe timing lines (when live UI is off or non-TTY).
      Set \`LUNA_OUTDATED_NO_LIVE=1\` to disable the live spinner block (CI-friendly logs).

  update
      Refreshes toolsets that have actionable upgrades under the current policy (skips tiers already up to date):
        Proto (\`proto outdated --update\`, then per-pin \`proto install\` from \`.proto/logs/\` so failure logs stay out of the repo root; Python may fall back to \`--build\` if no pre-built exists);
        Bun (\`bun update --recursive\` and per-workspace manifests, then \`bun add pkg@newest\` for the \`Update\` column and optional range-widen adds);
        uv (\`uv lock --upgrade\` + \`uv sync\`) per Python project;
        Go per module root: tool-only (\`go get -tool @newest\` from \`go list -m -u\`, or \`@latest\` with \`--major\`) +
          \`go mod tidy\` + \`go tool\` smoke-test; code modules (\`go get -u all\` + tidy + \`go build ./...\` when packages exist);
        then \`bun run setup\` (root \`package.json\` script: proto, workspaces, \`moon run web:setup\`, api build).
      Only true major bumps (leading non-zero version digit changing, e.g. 1.x → 2.x) are blocked — use \`--major\` to apply them.
      Precheck reuses \`.cache/outdated-snapshot.json\` when the fingerprint matches and \`writtenAt\` is under 12 hours old (otherwise rescans automatically). Pass \`--refresh-outdated\` (or \`LUNA_UPDATE_REFRESH_OUTDATED=1\`) to force a live rescan. Exits early when nothing is actionable (e.g. only major bumps remain).

  update --major
      Same pipeline with Proto \`--latest\`, Bun \`update --latest\`, plus Bun prerelease catch-up where needed;
      for Go code modules also \`go get go@latest\` and each \`tool\` at \`@latest\` before \`go get -u all\`; tool-only
      modules use \`go get -tool …@latest\` only (no \`go get -u all\`).

Options:
  -h, --help          Show this help
  -v, -V, --version   Print CLI version

Python project discovery:
  • Primary: one \`moon query projects\` (all languages), then filter projects with \`language: python\` and \`pyproject.toml\`.
  • Fallback: scan \`apps/*\` and \`packages/*\` for \`moon.yml\` with \`language: python\` plus
    \`pyproject.toml\`.
  • Multiple projects: every match gets \`uv lock\` / \`uv sync\` during \`luna update\`.

Go module discovery:
  • Primary: same \`moon query projects\` JSON as Python, filtered by \`language: go\` and \`go.mod\`.
  • Fallback: scan \`apps/*\` and \`packages/*\` for \`moon.yml\` with \`language: go\` plus \`go.mod\`.
  • Multiple modules: each gets \`go get\` / \`go mod tidy\` / post-bump verify during \`luna update\`.
  • Tool-only modules (e.g. \`apps/web\` with \`tool github.com/gohugoio/hugo\` and no local \`.go\` files): only
    \`tool\` lines are checked/updated — not every Hugo transitive indirect.

Optional env (add a path Moon does not list, e.g. outside apps/packages):
  UV_PROJECT_ROOT              Extra uv project dir (absolute or relative to repo root)
  GO_MODULE_ROOT               Extra Go module dir (absolute or relative to repo root)
  LUNA_OUTDATED_PROGRESS       Set to \`1\` to print per-toolchain probe timings during \`luna outdated\` (non-TTY or with \`LUNA_OUTDATED_NO_LIVE=1\`)
  LUNA_OUTDATED_NO_LIVE        Set to \`1\` to disable the TTY live spinner + early ✓/✗ block (cleaner CI logs)
  LUNA_UPDATE_REFRESH_OUTDATED   Set to \`1\` to force a live outdated precheck during \`luna update\` (same as \`--refresh-outdated\`)
  LUNA_GO_FULL_GRAPH               Set to \`1\` to use \`go get -n -u all\` / \`go get -u all\` on tool-only modules (slow; legacy)
`);
}
