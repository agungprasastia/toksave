import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { CAVEMAN_SKILL_VERSION } from "../content/caveman-skill.js";
import type { RunOpts } from "../registry.js";
import type { HealthIssue, HealthStatus, RepairResult } from "../util/health.js";
import * as paths from "../util/paths.js";

// Caveman is a "skill" — pure markdown files, no binary to install.
// The actual wiring is done per-agent in agents/*.ts.

/** Install caveman (no-op — skill files are written during wire step). */
export async function install(_opts: RunOpts): Promise<boolean> {
  return true;
}

/** Get installed Caveman skill version by reading skill file. */
export function installedVersion(): string | null {
  // Check all agents that use SKILL.md files (Claude Code, Antigravity)
  const skillPaths = [
    join(paths.claudePaths().skillsDir, "caveman/SKILL.md"),
    join(paths.antigravityPaths().dir, "config", "skills", "caveman", "SKILL.md"),
  ];

  for (const skillPath of skillPaths) {
    try {
      if (!existsSync(skillPath)) continue;

      const content = readFileSync(skillPath, "utf-8");
      const versionMatch = content.match(/^version:\s*(.+)$/m);
      if (versionMatch) return versionMatch[1].trim();

      // Fallback: if no version field, assume it's an old version
      return "0.0.0";
    } catch {}
  }

  return null;
}

/** Get latest Caveman skill version. */
export async function latestVersion(): Promise<string | null> {
  return CAVEMAN_SKILL_VERSION;
}

/** Check if Caveman skill files are installed. */
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
          message: "Caveman skill not found",
          remediation: "Run: toksave install caveman",
        },
      ],
    };
  }

  // Check if version is outdated
  const latest = CAVEMAN_SKILL_VERSION;
  if (version !== latest) {
    issues.push({
      severity: "warning",
      message: `Caveman skill version ${version} is outdated (latest: ${latest})`,
      remediation: "Run: toksave update caveman",
    });
  }

  return {
    healthy: issues.filter((i) => i.severity === "error").length === 0,
    version,
    issues,
  };
}

/** Attempt to repair Caveman installation. */
export async function repair(_opts: RunOpts): Promise<RepairResult> {
  const beforeHealth = healthCheck();

  if (beforeHealth.healthy) {
    return {
      success: true,
      message: "Caveman is already healthy, no repair needed",
      healthAfterRepair: beforeHealth,
    };
  }

  // Caveman repair requires re-wiring to agents, which is handled by the wire command
  return {
    success: false,
    message: "Caveman repair requires running: toksave init caveman",
  };
}
