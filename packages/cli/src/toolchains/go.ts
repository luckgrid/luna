import { existsSync, readFileSync } from "node:fs";
import { spawnExit, spawnText } from "../lib/process";

function must0(code: number, step: string): void {
  if (code !== 0) {
    console.error(`error: ${step} failed (exit ${code})`);
    process.exit(code);
  }
}

const goModRequireLineRe = /^\s*([^/\s][^\s]+)\s+(v[0-9].*)/;

/** Module paths from go.mod require blocks (direct + indirect), one per line. */
export function goModRequiredModulePathsFromFile(gomodPath: string): string[] {
  let raw: string;
  try {
    raw = readFileSync(gomodPath, "utf8");
  } catch {
    return [];
  }
  const paths = new Set<string>();
  for (const line of raw.split("\n")) {
    const m = goModRequireLineRe.exec(line);
    if (m) paths.add(m[1]);
  }
  return [...paths].toSorted();
}

/** Subset of `go list -u -m all` lines for modules listed in go.mod (with `[newer]`). */
export function goFilterGoOutLinesModfileUpdates(gomodPath: string, fullList: string): string {
  const allowed = new Set(goModRequiredModulePathsFromFile(gomodPath));
  const out: string[] = [];
  for (const line of fullList.split("\n")) {
    const t = line.trim();
    if (!t || t.startsWith("go:")) continue;
    if (!t.includes("[")) continue;
    const path = t.split(/\s+/)[0];
    if (path && allowed.has(path)) out.push(t);
  }
  return out.join("\n");
}

export function goModUListHasTableRows(gomodPath: string, fullList: string): boolean {
  return goFilterGoOutLinesModfileUpdates(gomodPath, fullList).trim().length > 0;
}

export function captureGoListUModule(goRoot: string): string {
  return spawnText(["go", "list", "-u", "-m", "all"], { cwd: goRoot });
}

export function goEnvGomod(goRoot: string): string {
  const r = Bun.spawnSync(["go", "env", "GOMOD"], {
    cwd: goRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  const p = new TextDecoder().decode(r.stdout).trim();
  if (!p || !existsSync(p)) return "";
  return p;
}

export function goModHasUpgrades(goRoot: string): boolean {
  const gomod = goEnvGomod(goRoot);
  if (!gomod) return false;
  const required = goModRequiredModulePathsFromFile(gomod);
  if (required.length === 0) return false;
  const reqSet = new Set(required);
  const listOut = spawnText(
    ["go", "list", "-u", "-m", "-f", "{{if .Update}}{{println .Path}}{{end}}", "all"],
    { cwd: goRoot },
  );
  const updates = new Set(
    listOut
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean),
  );
  for (const p of reqSet) {
    if (updates.has(p)) return true;
  }
  return false;
}

export function goApplyModfileModuleUpdates(goRoot: string): void {
  const gomod = goEnvGomod(goRoot);
  if (!gomod) return;

  const listPkgs = spawnText(
    ["go", "list", "-u", "-m", "-f", "{{if .Update}}{{.Path}} {{.Update.Version}}{{end}}", "all"],
    { cwd: goRoot },
  );
  const reqSet = new Set(goModRequiredModulePathsFromFile(gomod));
  const pkgs: string[] = [];
  for (const line of listPkgs.split("\n")) {
    const t = line.trim();
    if (!t) continue;
    const [path, ver] = t.split(/\s+/);
    if (path && ver && reqSet.has(path)) pkgs.push(`${path}@${ver}`);
  }
  if (pkgs.length > 0) {
    must0(spawnExit(["go", "get", ...pkgs], { cwd: goRoot }), "go get (modfile modules)");
  }
  must0(spawnExit(["go", "get", "-u", "./..."], { cwd: goRoot }), "go get -u ./...");
  must0(
    spawnExit(["go", "get", "-u", "github.com/a-h/templ/cmd/templ"], { cwd: goRoot }),
    "go get -u templ",
  );
  must0(spawnExit(["go", "mod", "tidy"], { cwd: goRoot }), "go mod tidy");
}
