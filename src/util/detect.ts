import { existsSync } from "node:fs";
import { join } from "node:path";
import which from "which";

/** Check if a binary is on PATH. */
export function isOnPath(name: string): boolean {
  try {
    which.sync(name);
    return true;
  } catch {
    return false;
  }
}

/** Find a binary on PATH, returning its full path. */
export function findBinary(name: string): string | null {
  try {
    return which.sync(name);
  } catch {
    return null;
  }
}

/** Find a binary, also checking extra directories. */
export function findBinaryIn(name: string, extraDirs: string[]): string | null {
  // Check PATH first
  const onPath = findBinary(name);
  if (onPath) return onPath;

  // Check extra dirs
  for (const dir of extraDirs) {
    const candidate = join(dir, name);
    if (existsSync(candidate)) return candidate;

    // Windows: try with .exe
    if (process.platform === "win32") {
      const exe = join(dir, `${name}.exe`);
      if (existsSync(exe)) return exe;
    }
  }
  return null;
}
