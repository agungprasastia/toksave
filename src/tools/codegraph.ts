import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";
import type { RunOpts } from "../registry.js";
import { verbose } from "../util/colors.js";
import { isOnPath } from "../util/detect.js";
import { IndexError } from "../util/errors.js";
import type { HealthIssue, HealthStatus, RepairResult } from "../util/health.js";
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

export interface IndexResult {
  success: boolean;
  indexPath: string;
}

export interface IndexOptions {
  verbose?: boolean;
  skipPatterns?: string[];
  onProgress?: (message: string) => void;
}

/** Pre-build index for the given directory. */
export function indexProject(dir: string, opts?: IndexOptions): IndexResult {
  const indexPath = join(dir, ".codegraph");
  const hasIndex = existsSync(indexPath);

  try {
    if (hasIndex) {
      opts?.onProgress?.("Syncing existing index...");
      execSync("codegraph sync", { cwd: dir, stdio: opts?.verbose ? "inherit" : "ignore" });
    } else {
      opts?.onProgress?.("Creating new index...");
      execSync("codegraph init", { cwd: dir, stdio: opts?.verbose ? "inherit" : "ignore" });
    }
    return { success: true, indexPath };
  } catch (err) {
    throw new IndexError("codegraph", {
      message: `Failed to ${hasIndex ? "sync" : "initialize"} CodeGraph index`,
      cause: err,
      dir,
      remediation: "Ensure codegraph binary is executable and directory is writable",
    });
  }
}

/** Check if CodeGraph is installed and working. */
export function healthCheck(): HealthStatus {
  const issues: HealthIssue[] = [];
  const version = installedVersion();

  if (!version) {
    return {
      healthy: false,
      version: null,
      issues: [
        {
          severity: "error",
          message: "CodeGraph is not installed",
          remediation: "Run: toksave install codegraph",
        },
      ],
    };
  }

  if (!isOnPath("codegraph")) {
    issues.push({
      severity: "error",
      message: "CodeGraph is installed but not in PATH",
      remediation: "Add npm global bin directory to PATH",
    });
  }

  return {
    healthy: issues.length === 0,
    version,
    issues,
  };
}

/** Attempt to repair a broken CodeGraph installation. */
export async function repair(opts: RunOpts): Promise<RepairResult> {
  try {
    const beforeHealth = healthCheck();

    if (beforeHealth.healthy) {
      return {
        success: true,
        message: "CodeGraph is already healthy, no repair needed",
        healthAfterRepair: beforeHealth,
      };
    }

    await install({ ...opts, upgrade: true });

    const afterHealth = healthCheck();

    if (afterHealth.healthy) {
      return {
        success: true,
        message: "CodeGraph successfully repaired",
        healthAfterRepair: afterHealth,
      };
    }

    return {
      success: false,
      message: "Repair attempted but health check still failing",
      healthAfterRepair: afterHealth,
    };
  } catch (err) {
    return {
      success: false,
      message: `Repair failed: ${err instanceof Error ? err.message : String(err)}`,
    };
  }
}
