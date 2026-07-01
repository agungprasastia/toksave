import type { RunOpts } from "../registry.js";
import { verbose } from "../util/colors.js";
import { isOnPath } from "../util/detect.js";
import {
  installGlobal,
  installedVersion as npmInstalledVersion,
  latestVersion as npmLatestVersion,
} from "../util/npm.js";

const PACKAGE = "context-mode";

/** Install Context-Mode globally via npm. */
export async function install(opts: RunOpts): Promise<boolean> {
  if (isOnPath("context-mode") && !opts.upgrade) return true;
  if (opts.dryRun) return true;
  verbose(`Running npm install -g ${PACKAGE}`, opts.verbose);
  installGlobal(PACKAGE);
  return true;
}

/** Get installed Context-Mode version. */
export function installedVersion(): string | null {
  return npmInstalledVersion(PACKAGE);
}

/** Get latest Context-Mode version from npm registry. */
export async function latestVersion(): Promise<string | null> {
  return npmLatestVersion(PACKAGE);
}
