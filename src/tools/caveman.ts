import type { RunOpts } from "../registry.js";

// Caveman is a "skill" — pure markdown files, no binary to install.
// The actual wiring is done per-agent in agents/*.ts.

/** Install caveman (no-op — skill files are written during wire step). */
export async function install(_opts: RunOpts): Promise<boolean> {
  return true;
}

/** Not version-trackable. */
export function installedVersion(): string | null {
  return null;
}

/** Not version-trackable. */
export function latestVersion(): string | null {
  return null;
}
