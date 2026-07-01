import type { RunOpts } from "../registry.js";
import { verbose } from "../util/colors.js";
import { isOnPath } from "../util/detect.js";
import {
  installGlobal,
  installedVersion as npmInstalledVersion,
  latestVersion as npmLatestVersion,
} from "../util/npm.js";

const PACKAGE = "@colbymchenry/codegraph";

/** Install CodeGraph globally via npm. */
export async function install(opts: RunOpts): Promise<boolean> {
  if (isOnPath("codegraph") && !opts.upgrade) return true;
  if (opts.dryRun) return true;
  verbose(`Running npm install -g ${PACKAGE}`, opts.verbose);
  installGlobal(PACKAGE);
  return true;
}

/** Get installed CodeGraph version. */
export function installedVersion(): string | null {
  return npmInstalledVersion(PACKAGE);
}

/** Get latest CodeGraph version from npm registry. */
export async function latestVersion(): Promise<string | null> {
  return npmLatestVersion(PACKAGE);
}

/** Pre-build index for the given directory. */
export function indexProject(dir: string): boolean {
  try {
    const { execSync } = require("node:child_process");
    const hasIndex = require("node:fs").existsSync(require("node:path").join(dir, ".codegraph"));
    if (hasIndex) {
      execSync("codegraph sync", { cwd: dir, stdio: "ignore" });
      return true;
    }
    try {
      execSync("codegraph init -i", { cwd: dir, stdio: "ignore" });
    } catch {
      execSync("codegraph init", { cwd: dir, stdio: "ignore" });
    }
    return true;
  } catch {
    return false;
  }
}
