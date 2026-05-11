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

  update
      Refreshes those toolsets in order:
        Proto (\`proto outdated --update\`, then per-pin \`proto install\` from \`.proto/logs/\` so failure logs stay out of the repo root; Python may fall back to \`--build\` if no pre-built exists);
        Bun (\`bun update --recursive\` and per-workspace manifests);
        uv (\`uv lock --upgrade\` + \`uv sync\`) per Python project;
        Go (\`go get -u all\` + \`go mod tidy\`) per module root;
        then \`bun run setup\` (root \`package.json\` script: proto, workspaces, \`moon run web:setup\`, api build).

  update --major
      Same pipeline with Proto \`--latest\`, Bun \`update --latest\`, plus Bun prerelease catch-up where needed;
      for Go also \`go get go@latest\` and each \`tool\` line in go.mod at \`@latest\` before \`go get -u all\`.

Options:
  -h, --help          Show this help
  -v, -V, --version   Print CLI version

Python project discovery:
  • Primary: \`moon query projects --language python\` (see each project's \`moon.yml\`).
  • Fallback: scan \`apps/*\` and \`packages/*\` for \`moon.yml\` with \`language: python\` plus
    \`pyproject.toml\`.
  • Multiple projects: every match gets \`uv lock\` / \`uv sync\` during \`luna update\`.

Go module discovery:
  • Primary: \`moon query projects --language go\` (see each project's \`moon.yml\`).
  • Fallback: scan \`apps/*\` and \`packages/*\` for \`moon.yml\` with \`language: go\` plus \`go.mod\`.
  • Multiple modules: each gets \`go get\` / \`go mod tidy\` during \`luna update\`.

Optional env (add a path Moon does not list, e.g. outside apps/packages):
  UV_PROJECT_ROOT   Extra uv project dir (absolute or relative to repo root)
  GO_MODULE_ROOT    Extra Go module dir (absolute or relative to repo root)
`);
}
