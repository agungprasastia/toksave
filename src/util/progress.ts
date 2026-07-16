import pc from "picocolors";

import { C } from "./colors.js";

const BAR_WIDTH = 20;
const LABEL_WIDTH = 26;

export class Progress {
  private timer: ReturnType<typeof setInterval> | null = null;
  private progress = 0;
  private label = "";
  private tty = process.stdout.isTTY;

  start(label: string) {
    this.label = label;
    this.progress = 0;

    if (this.tty) {
      this.render();
      this.timer = setInterval(() => {
        this.progress += (95 - this.progress) * 0.05;
        this.render();
      }, 50);
    } else {
      console.log(label);
    }
  }

  stop(message: string, isError = false) {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }

    if (this.tty) {
      this.progress = isError ? this.progress : 100;

      const p = Math.floor(this.progress);
      const blocks = Math.floor((p / 100) * BAR_WIDTH);
      const color = isError ? pc.red : pc.green;
      const bar = color("█".repeat(blocks)) + pc.dim("█".repeat(BAR_WIDTH - blocks));
      const percent = isError ? pc.red(`${p}%`.padStart(4)) : pc.green(`${p}%`.padStart(4));

      // biome-ignore lint/suspicious/noControlCharactersInRegex: Ansi color codes require control characters
      const plainMessage = message.replace(/\x1B\[\d+m/g, "");
      const padding = Math.max(0, LABEL_WIDTH - plainMessage.length);
      const spaces = " ".repeat(padding);

      process.stdout.write(`\x1b[2K\r  ${message}${spaces}[${bar}] ${percent}\n`);
    } else {
      console.log(`  ${message}`);
    }
  }

  update(text: string) {
    this.label = text;
    if (this.tty) this.render();
  }

  private render() {
    const p = Math.floor(this.progress);
    const blocks = Math.floor((p / 100) * BAR_WIDTH);
    const bar = pc.cyan("█".repeat(blocks)) + pc.dim("█".repeat(BAR_WIDTH - blocks));
    const percent = pc.cyan(`${p}%`.padStart(4));

    // biome-ignore lint/suspicious/noControlCharactersInRegex: Ansi color codes require control characters
    const plainLabel = this.label.replace(/\x1B\[\d+m/g, "");
    const padding = Math.max(0, LABEL_WIDTH - plainLabel.length);
    const spaces = " ".repeat(padding);

    process.stdout.write(`\x1b[2K\r  ${this.label}${spaces}[${bar}] ${percent}`);
  }
}

// ─── Symbols ─────────────────────────────────────────────────

export const Sym = {
  Check: "✔",
  Warn: "⚠",
  Bullet: "•",
};

// ─── Tree Helpers ────────────────────────────────────────────

export function TreeLeaf(msg: string): void {
  console.log(`  ${msg}`);
}

export function TreeCorner(msg: string): void {
  console.log(`  └─ ${msg}`);
}

export function TreeCornerStyled(msg: string): void {
  console.log(`  └─ ${msg}`);
}

export function TreeFooter(width: number): void {
  console.log(`  ${"─".repeat(width)}`);
}

export function Rule(width: number): string {
  return "─".repeat(width);
}

export function VisibleLen(s: string): number {
  let n = 0;
  for (let i = 0; i < s.length; i++) {
    if (s.charCodeAt(i) === 0x1b) {
      for (i++; i < s.length; i++) {
        const c = s.charCodeAt(i);
        if ((c >= 0x61 && c <= 0x7a) || (c >= 0x41 && c <= 0x5a)) break;
      }
      continue;
    }
    n++;
  }
  return n;
}

export function EraseStyledLine(_s: string): string {
  return "\r\x1b[2K";
}

// ─── Section Progress ────────────────────────────────────────

const spinnerFrames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

export class SectionProgress {
  protected label: string;
  protected total = 0;
  protected current = "";
  protected phase = "";
  protected frac = 0;
  protected startTime = 0;
  protected frame = 0;
  protected active = false;
  protected tty: boolean;
  protected rows = 0;
  protected lastNonTTY = "";
  protected lastWidth = 0;
  protected timer: ReturnType<typeof setInterval> | null = null;
  protected treeRoot = false;

  constructor(label: string) {
    this.label = label;
    this.tty = process.stdout.isTTY === true;
  }

  Start(n: number): void {
    this.total = n;
    this.rows = 0;
    if (this.tty) {
      console.log(`  ${C.Magenta(C.Bold("●"))} ${C.Magenta(C.Bold(this.label))}`);
    } else if (this.treeRoot) {
      TreeTop(this.label);
    } else {
      TreeCorner(this.label);
    }
    if (this.tty) {
      this.timer = setInterval(() => this.tick(), 80);
    }
  }

  private tick(): void {
    if (!this.active) return;
    this.frame = (this.frame + 1) % spinnerFrames.length;
    this.render();
  }

  Begin(label: string): void {
    this.current = label;
    this.phase = "";
    this.frac = 0;
    this.lastNonTTY = "";
    this.startTime = Date.now();
    this.active = true;
    if (this.tty) this.render();
  }

  Step(phase: string, frac: number): void {
    if (frac < 0) frac = 0;
    if (frac > 1) frac = 1;
    this.phase = phase;
    this.frac = frac;
    if (this.tty) {
      this.render();
      return;
    }
    if (!phase || phase === this.lastNonTTY) return;
    this.lastNonTTY = phase;
    const lbl = this.current || this.label;
    console.log(`  ${C.Dim("·")} ${lbl} — ${phase}`);
  }

  Complete(note?: string): void {
    this.active = false;
    this.clearLine();
    const noteStr = note ? ` ${C.Dim(note)}` : "";
    console.log(`${this.indent()}${C.Green(Sym.Check)} ${this.current}${noteStr}`);
    this.rows++;
  }

  Fail(reason: string): void {
    this.active = false;
    this.clearLine();
    console.log(`${this.indent()}${C.Red("✖")} ${this.current} ${C.Red(reason)}`);
    this.rows++;
  }

  Done(_msg: string): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.active = false;
    this.clearLine();
    if (this.tty && this.label) {
      const up = this.rows + 1;
      process.stdout.write(`\x1b[${up}A\r\x1b[2K${C.Dim("└─")} ${C.Bold(this.label)}\x1b[${up}B\r`);
    }
    this.closeBranch();
    this.rows = 0;
  }

  protected indent(): string {
    return C.Dim("│   ");
  }

  private clearLine(): void {
    if (this.tty) {
      this.lastWidth = 0;
      process.stdout.write("\r\x1b[2K");
    }
  }

  private render(): void {
    if (!this.tty || !this.current || !this.active) return;
    const phaseStr = this.phase ? `  ${C.Dim(this.phase)}` : "";
    const elp = this.startTime
      ? ` ${C.Dim(`${Math.floor((Date.now() - this.startTime) / 1000)}s`)}`
      : "";
    const frame = spinnerFrames[this.frame]!;
    const line = `${this.indent()}${C.Cyan(frame)} ${this.current}${phaseStr}${elp}`;
    const w = VisibleLen(line);
    const pad = this.lastWidth > w ? this.lastWidth - w : 0;
    this.lastWidth = w;
    process.stdout.write(`\r\x1b[2K${line}${" ".repeat(pad)}`);
  }

  private closeBranch(): void {
    console.log(`${C.Dim("│")}`);
  }
}

export class RootSectionProgress extends SectionProgress {
  constructor(label: string) {
    super(label);
    this.treeRoot = true;
  }
}

// ─── Tree Helpers (internal; SectionProgress.Done non-TTY uses TreeTop/TreeCorner) ──

function TreeTop(title: string): void {
  console.log(`${C.Dim("┌─")} ${C.Bold(title)}`);
}

// ─── runStatus — spinner + async fn ──────────────────────────

export async function runStatus<T>(label: string, fn: () => Promise<T>): Promise<T> {
  const tty = process.stdout.isTTY === true;
  const f = spinnerFrames;
  let i = 0;
  let interval: ReturnType<typeof setInterval> | null = null;

  if (tty) {
    interval = setInterval(() => {
      const ch = f[i]!;
      process.stdout.write(`\r\x1b[2K  ${C.Cyan(ch)} ${label}`);
      i = (i + 1) % f.length;
    }, 80);
  } else {
    console.log(`  ${label}`);
  }

  try {
    const result = await fn();
    if (interval) {
      clearInterval(interval);
      process.stdout.write(`\r\x1b[2K  ${C.Green(Sym.Check)} ${label}\n`);
    }
    return result;
  } catch (e) {
    if (interval) {
      clearInterval(interval);
      process.stdout.write(`\r\x1b[2K  ${C.Red("✖")} ${label}\n`);
    }
    throw e;
  }
}
