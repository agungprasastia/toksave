import type { RunOpts } from "../registry.js";
import { verbose } from "../util/colors.js";
import { isOnPath } from "../util/detect.js";
import type { HealthIssue, HealthStatus, RepairResult } from "../util/health.js";
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

/** Check if Context-Mode is installed and working. */
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
          message: "Context-Mode is not installed",
          remediation: "Run: toksave install context-mode",
        },
      ],
    };
  }

  if (!isOnPath("context-mode")) {
    issues.push({
      severity: "error",
      message: "Context-Mode is installed but not in PATH",
      remediation: "Add npm global bin directory to PATH",
    });
  }

  return {
    healthy: issues.length === 0,
    version,
    issues,
  };
}

/** Attempt to repair a broken Context-Mode installation. */
export async function repair(opts: RunOpts): Promise<RepairResult> {
  try {
    const beforeHealth = healthCheck();

    if (beforeHealth.healthy) {
      return {
        success: true,
        message: "Context-Mode is already healthy, no repair needed",
        healthAfterRepair: beforeHealth,
      };
    }

    await install({ ...opts, upgrade: true });

    const afterHealth = healthCheck();

    if (afterHealth.healthy) {
      return {
        success: true,
        message: "Context-Mode successfully repaired",
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
