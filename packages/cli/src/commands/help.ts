/** Keep in sync with `main.ts` command dispatch. */
export function printRootHelp(): void {
  console.log(`luna — Luna monorepo CLI (Bun)

Usage:
  luna [options] <command>

Commands:
  outdated   Report outdated proto, bun, uv; exit 1 if any tier has upgrades (CI)
  update     Apply upgrades (proto → bun → uv), then bun run setup

Options:
  -h, --help          Show this help
  -v, -V, --version   Print CLI version

Python project discovery:
  • Primary: \`moon query projects --language python\` (see each project's \`moon.yml\`).
  • Fallback: scan \`apps/*\` and \`packages/*\` for \`moon.yml\` with \`language: python\` plus
    \`pyproject.toml\`.
  • Multiple projects: every match gets \`uv lock\` / \`uv sync\` during \`luna update\`.

Optional env (add a path Moon does not list, e.g. outside apps/packages):
  UV_PROJECT_ROOT   Extra uv project dir (absolute or relative to repo root)
`);
}
