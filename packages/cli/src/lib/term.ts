/** ANSI styling (disabled for NO_COLOR, non-tty stderr, or TERM=dumb). */
function setupColors(): { red: string; green: string; bold: string; dim: string; reset: string } {
  if (process.env.NO_COLOR || process.env.TERM === "dumb" || !process.stderr.isTTY) {
    return { red: "", green: "", bold: "", dim: "", reset: "" };
  }
  return {
    red: "\x1b[0;31m",
    green: "\x1b[0;32m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    reset: "\x1b[0m",
  };
}

const C = setupColors();

export function section(title: string): void {
  console.log(`\n${C.bold}== ${title} ==${C.reset}`);
}

export function strictOk(msg: string): void {
  console.error(`${C.green}✓${C.reset} ${msg}`);
}

export function strictNeed(msg: string): void {
  console.error(`${C.red}✗${C.reset} ${msg}`);
}

export function strictSummaryFailTitle(msg: string): void {
  console.error(`${C.bold}${C.red}${msg}${C.reset}`);
}

export function strictSummaryBullet(msg: string): void {
  console.error(`  ${C.red}•${C.reset} ${msg}`);
}

export function strictHint(msg: string): void {
  console.error(`${C.dim}${msg}${C.reset}`);
}

export function strictAllPassed(msg: string): void {
  console.log(`${C.green}${msg}${C.reset}`);
}

export function die(msg: string): never {
  console.error(`error: ${msg}`);
  process.exit(1);
}
