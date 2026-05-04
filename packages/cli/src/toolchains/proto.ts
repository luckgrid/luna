import { die } from "../lib/term";

export function protoHasOutdatedPins(): boolean {
  const r = Bun.spawnSync(["proto", "outdated", "--json"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const text = new TextDecoder().decode(r.stdout).trim();
  if (!text) die("proto outdated --json returned empty output (is proto in PATH?)");
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    die("proto outdated --json returned invalid JSON");
  }
  if (typeof data !== "object" || data === null) die("proto outdated --json: unexpected shape");
  const entries = Object.values(data);
  return entries.some((x) => {
    if (typeof x !== "object" || x === null) return false;
    if (!("is_outdated" in x)) return false;
    return Reflect.get(x, "is_outdated") === true;
  });
}

export function printProtoOutdated(): void {
  Bun.spawnSync(["proto", "outdated"], { stdout: "inherit", stderr: "inherit", stdin: "ignore" });
}
