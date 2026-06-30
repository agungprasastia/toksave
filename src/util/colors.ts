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

export { pc };
