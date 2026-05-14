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
                Go modules via \`go get -n -u all\` dry-run (Moon \`language: go\` + go.mod, + GO_MODULE_ROOT).
      Exits 1 if anything is outdated (CI).
      Writes a best-effort snapshot to \`.cache/outdated-snapshot.json\` for \`luna update --use-outdated-cache\`.
      On an interactive terminal, shows a live status line while probes run, then per-tier ✓/✗ lines before tables.
      \`outdated --use-cache\` reuses the cache when the fingerprint still matches (fast; not for CI).
      Set \`LUNA_OUTDATED_PROGRESS=1\` for per-probe timing lines (when live UI is off or non-TTY).
      Set \`LUNA_OUTDATED_NO_LIVE=1\` to disable the live spinner block (CI-friendly logs).

  update
      Refreshes those toolsets in order:
        Proto (\`proto outdated --update\`, then per-pin \`proto install\` from \`.proto/logs/\` so failure logs stay out of the repo root; Python may fall back to \`--build\` if no pre-built exists);
        Bun (\`bun update --recursive\` and per-workspace manifests, then \`bun add pkg@latest\` per workspace for non-major bumps that caret semver leaves stuck — e.g. 0.x → 0.x+1);
        uv (\`uv lock --upgrade\` + \`uv sync\`) per Python project;
        Go (\`go get -u all\` + \`go mod tidy\`, then \`go build ./...\` when packages exist + smoke-test each \`tool\` via \`go tool\`) per module root;
        then \`bun run setup\` (root \`package.json\` script: proto, workspaces, \`moon run web:setup\`, api build).
      Only true major bumps (leading non-zero version digit changing, e.g. 1.x → 2.x) are blocked — use \`--major\` to apply them.
      Precheck uses an in-process snapshot (no nested \`luna outdated\`). Optional \`--use-outdated-cache\` (or \`LUNA_UPDATE_USE_OUTDATED_CACHE=1\`) skips live precheck when \`.cache/outdated-snapshot.json\` matches the current fingerprint.

  update --major
      Same pipeline with Proto \`--latest\`, Bun \`update --latest\`, plus Bun prerelease catch-up where needed;
      for Go also \`go get go@latest\` and each \`tool\` line in go.mod at \`@latest\` before \`go get -u all\`.

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

Optional env (add a path Moon does not list, e.g. outside apps/packages):
  UV_PROJECT_ROOT              Extra uv project dir (absolute or relative to repo root)
  GO_MODULE_ROOT               Extra Go module dir (absolute or relative to repo root)
  LUNA_OUTDATED_PROGRESS       Set to \`1\` to print per-toolchain probe timings during \`luna outdated\` (non-TTY or with \`LUNA_OUTDATED_NO_LIVE=1\`)
  LUNA_OUTDATED_NO_LIVE        Set to \`1\` to disable the TTY live spinner + early ✓/✗ block (cleaner CI logs)
  LUNA_UPDATE_USE_OUTDATED_CACHE   Set to \`1\` to use \`.cache/outdated-snapshot.json\` during \`luna update\` precheck (same as \`--use-outdated-cache\`)
`);
}
