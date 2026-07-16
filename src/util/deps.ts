import { runStdout } from "./exec.js";

export interface DepsResult {
  ok: boolean;
  nodeVersion: string | null;
  gitAvailable: boolean;
}

export function ensureDeps(needNode?: boolean, needGit?: boolean, minNode?: number): DepsResult {
  let nodeVersion: string | null = null;
  let gitAvailable = false;

  if (needNode) {
    const out = runStdout("node", ["--version"]);
    nodeVersion = out?.replace(/^v/, "").trim() ?? null;
    if (nodeVersion && minNode !== undefined) {
      const major = Number.parseInt(nodeVersion.split(".")[0], 10);
      if (!Number.isNaN(major) && major < minNode) {
        console.warn(
          `Node.js ${nodeVersion} detected but >= v${minNode}.x required. ` +
            "Upgrade Node.js at https://nodejs.org",
        );
        return { ok: false, nodeVersion, gitAvailable };
      }
    }
  }

  if (needGit) {
    const out = runStdout("git", ["--version"]);
    gitAvailable = out !== null;
    if (!gitAvailable) {
      console.warn("git not found on PATH. Install git: https://git-scm.com");
      return { ok: false, nodeVersion, gitAvailable: false };
    }
  }

  return { ok: true, nodeVersion, gitAvailable };
}
