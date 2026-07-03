import { readFileSync } from "node:fs";

/**
 * Context-Mode hook for Antigravity PreInvocation.
 * Proxies the hook event to the context-mode CLI.
 */
export function runContextModeHook(): number {
  try {
    const input = readFileSync(0, "utf-8");
    if (!input) return 0;

    // Pass through — context-mode handles the actual logic.
    // This hook exists so toksave can be the single entry point
    // without requiring context-mode to be directly on PATH.
    const { spawnSync } = require("node:child_process") as typeof import("node:child_process");

    const result = spawnSync("context-mode", ["hook", ...process.argv.slice(3)], {
      input,
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
      timeout: 8_000,
    });

    if (result.stdout) {
      process.stdout.write(result.stdout);
    }

    return result.status ?? 0;
  } catch {
    return 0; // On error, don't block the invocation
  }
}
