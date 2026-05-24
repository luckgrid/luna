import { envFlagEnabled, terminalAnsiStderr } from "../../lib/utils";
import type { OutdatedTierId } from "./types";

const SPIN = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"] as const;

/** Live single-line status on stderr while probes run (TTY only). */
export function isOutdatedLiveStatusEnabled(): boolean {
  if (envFlagEnabled("LUNA_OUTDATED_NO_LIVE")) return false;
  if (process.env.CI === "true" || process.env.CI === "1") return false;
  return process.stderr.isTTY && process.stderr.writable;
}

type TierCell = "pending" | "running" | { ok: boolean; ms: number };

const TIER_ORDER: OutdatedTierId[] = ["proto", "bun", "uv", "go"];

function sectionErr(title: string): void {
  const bold = terminalAnsiStderr() ? "\x1b[1m" : "";
  const reset = terminalAnsiStderr() ? "\x1b[0m" : "";
  process.stderr.write(`\n${bold}== ${title} ==${reset}\n`);
}

export class OutdatedLiveStatus {
  private readonly cells: Record<OutdatedTierId, TierCell> = {
    proto: "pending",
    bun: "pending",
    uv: "pending",
    go: "pending",
  };
  private spinIdx = 0;
  private timer: ReturnType<typeof setInterval> | null = null;

  begin(): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    process.stderr.write("\n[luna] scanning toolchains (parallel)…\n");
    for (const t of TIER_ORDER) this.cells[t] = "running";
    this.timer = setInterval(() => this.draw(), 110);
    this.draw();
  }

  markDone(tier: OutdatedTierId, ok: boolean, ms: number): void {
    this.cells[tier] = { ok, ms };
    if (isOutdatedLiveStatusEnabled()) this.draw();
  }

  private draw(): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    const spin = SPIN[this.spinIdx % SPIN.length] ?? "⠋";
    this.spinIdx += 1;
    const parts = TIER_ORDER.map((id) => {
      const c = this.cells[id];
      if (c === "pending") return `○ ${id}`;
      if (c === "running") return `${spin} ${id}`;
      const mark = c.ok ? "✓" : "✗";
      return `${mark} ${id} (${c.ms}ms)`;
    });
    const line = `[luna] outdated  ${parts.join("  │  ")}`;
    const w = Math.max(20, process.stderr.columns ?? 80);
    const padded = line.length >= w ? `${line.slice(0, w - 4)} …` : line.padEnd(w, " ");
    process.stderr.write(`\r\x1b[2K${padded}`);
  }

  printSummaryBlock(
    lines: { ok: boolean; text: string }[],
    printLine: (ok: boolean, text: string) => void,
  ): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    sectionErr("check results");
    for (const { ok, text } of lines) printLine(ok, text);
    process.stderr.write("\n");
  }

  finishAfterGather(
    lines: { ok: boolean; text: string }[],
    printLine: (ok: boolean, text: string) => void,
  ): void {
    if (!isOutdatedLiveStatusEnabled()) return;
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    process.stderr.write("\r\x1b[2K");
    this.printSummaryBlock(lines, printLine);
  }
}
