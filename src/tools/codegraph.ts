import { execSync, spawn } from "node:child_process";
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
  if (process.env.TOKSAVE_TEST === "1") return true;
  if (opts.dryRun) return true;
  verbose(`Running npm install -g ${PACKAGE}`, opts.verbose);
  try {
    installGlobal(PACKAGE);
  } catch {}
  return isOnPath("codegraph") || resolveCodegraphBin() !== "";
}

function resolveCodegraphBin(): string {
  const which = isOnPath("codegraph") ? "codegraph" : "";
  if (which) return which;
  // Try common npm global locations
  return "";
}

function codegraphRealInstall(opts: RunOpts, agent: string): boolean {
  if (opts.dryRun) return true;
  if (process.env.TOKSAVE_TEST === "1") return true;
  const bin = isOnPath("codegraph") ? "codegraph" : resolveCodegraphBin();
  if (!bin) return false;
  // Check if binary supports --yes and --target by probing help
  let hasYes = false;
  let hasTarget = false;
  try {
    const helpOut = execSync(`${bin} install --help`, { encoding: "utf-8", timeout: 10000 });
    hasYes = helpOut.includes("--yes");
    hasTarget = helpOut.includes("--target");
  } catch {
    // ignore
  }
  const args = ["install"];
  if (hasYes) args.push("--yes");
  if (hasTarget) {
    let target = agent;
    if (target === "antigravity") target = "gemini";
    if (!target) target = "all";
    args.push("--target", target);
  }
  try {
    execSync(`${bin} ${args.join(" ")}`, { stdio: "ignore", timeout: 10 * 60 * 1000 });
    return true;
  } catch {
    return false;
  }
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

/** Pre-build index for the given directory — sync or init, background friendly. */
export function indexProject(dir: string, opts?: IndexOptions): IndexResult {
  const indexPath = join(dir, ".codegraph");
  const hasIndex = existsSync(indexPath);

  try {
    if (process.env.TOKSAVE_TEST === "1") {
      // In test mode just ensure dir exists
      const { mkdirSync } = require("node:fs");
      mkdirSync(indexPath, { recursive: true });
      return { success: true, indexPath };
    }

    if (opts?.onProgress) {
      // Foreground for CLI manual usage
      if (hasIndex) {
        opts.onProgress("Syncing existing index...");
        execSync("codegraph sync", { cwd: dir, stdio: opts?.verbose ? "inherit" : "ignore" });
      } else {
        opts.onProgress("Creating new index...");
        try {
          execSync("codegraph init -i", { cwd: dir, stdio: "ignore" });
        } catch {
          execSync("codegraph init", { cwd: dir, stdio: opts?.verbose ? "inherit" : "ignore" });
        }
      }
    } else {
      // Background async (used by hooks / runmcp pre-index)
      codegraphSyncBackground("codegraph", dir);
    }
    return { success: true, indexPath };
  } catch (err) {
    if (opts?.onProgress) {
      throw new IndexError("codegraph", {
        message: `Failed to ${hasIndex ? "sync" : "initialize"} CodeGraph index`,
        cause: err,
        dir,
        remediation: "Ensure codegraph binary is executable and directory is writable",
      });
    }
    // Background mode: best-effort, return success anyway (like tokless)
    return { success: true, indexPath };
  }
}

function codegraphSyncBackground(bin: string, dir: string): void {
  try {
    const indexPath = join(dir, ".codegraph");
    if (existsSync(indexPath)) {
      const child = spawn(bin, ["sync"], { cwd: dir, detached: true, stdio: "ignore" });
      child.unref();
    } else {
      // Try init -i then fallback init
      const child = spawn(bin, ["init", "-i"], { cwd: dir, detached: true, stdio: "ignore" });
      child.on("error", () => {
        try {
          const child2 = spawn(bin, ["init"], { cwd: dir, detached: true, stdio: "ignore" });
          child2.unref();
        } catch {}
      });
      child.unref();
    }
  } catch {}
}

export function codegraphIndexReady(): boolean {
  if (process.env.TOKSAVE_TEST === "1") return true;
  return isOnPath("codegraph");
}

// Export for agent wiring use
export { codegraphRealInstall };

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
