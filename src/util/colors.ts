import pc from "picocolors";

// ─── Symbols ─────────────────────────────────────────────────
export const CHECK = "✔ ";
export const CROSS = "✖ ";
export const WARN = "⚠ ";
export const BULLET = "• ";
export const ARROW_UP = "↑ ";
export const INFO = "ℹ ";

// ─── Printers ────────────────────────────────────────────────

export function raw(msg: string) {
  console.log(msg);
}

export function ok(msg: string) {
  console.log(`  ${pc.green(CHECK)} ${msg}`);
}

export function err(msg: string) {
  console.error(`  ${pc.red(CROSS)} ${msg}`);
}

export function warn(msg: string) {
  console.log(`  ${pc.yellow(WARN)} ${msg}`);
}

export function info(msg: string) {
  console.log(`  ${pc.cyan(INFO)} ${msg}`);
}

export function banner(title: string, subtitle: string) {
  console.log();
  console.log(`  ${pc.bold(pc.cyan(title))}${pc.dim(`  ${subtitle}`)}`);
  console.log();
}

export function pad(s: string, width: number): string {
  return s.padEnd(width);
}

export function verbose(msg: string, isVerbose: boolean) {
  if (isVerbose) {
    console.log(`  ${pc.dim(`[v] ${msg}`)}`);
  }
}

export const C = {
  Bold: (s: string) => pc.bold(s),
  Cyan: (s: string) => pc.cyan(s),
  Gray: (s: string) => pc.gray(s),
  Yellow: (s: string) => pc.yellow(s),
  Green: (s: string) => pc.green(s),
  Red: (s: string) => pc.red(s),
  Magenta: (s: string) => pc.magenta(s),
  Dim: (s: string) => pc.dim(s),
};

export function gray(s: string): string {
  return pc.gray(s);
}
export function cyan(s: string): string {
  return pc.cyan(s);
}
export function bold(s: string): string {
  return pc.bold(s);
}
export function green(s: string): string {
  return pc.green(s);
}
export function yellow(s: string): string {
  return pc.yellow(s);
}

export function rule(width: number): string {
  return "─".repeat(width);
}

export function StdoutIsTTY(): boolean {
  return process.stdout.isTTY === true;
}

export function StdoutANSI(): boolean {
  return StdoutIsTTY() && process.env.NO_COLOR === undefined && process.env.TERM !== "dumb";
}

export const L = {
  Raw: (msg: string) => console.log(msg),
  Ok: (msg: string) => ok(msg),
  Warn: (msg: string) => warn(msg),
  Err: (msg: string) => err(msg),
  Info: (msg: string) => info(msg),
  Sub: (msg: string) => console.log(`  ${pc.dim(msg)}`),
  Debug: (msg: string) => {
    if (typeof process !== "undefined" && process.env?.DEBUG) {
      console.log(`  ${pc.dim(`[debug] ${msg}`)}`);
    }
  },
};

export { pc };
