import type { RunOpts } from "../registry.js";
import { isOnPath } from "../util/detect.js";
import { installGlobal, installedVersion as npmInstalledVersion, latestVersion as npmLatestVersion } from "../util/npm.js";
import { verbose } from "../util/colors.js";

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
export function latestVersion(): string | null {
  return npmLatestVersion(PACKAGE);
}
