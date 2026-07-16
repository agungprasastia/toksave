import pc from "picocolors";
import type { RunOpts } from "../registry.js";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  detectAgent,
  toolHealthCheck,
  toolInstalledVersion,
  toolLatestVersion,
  toolRepair,
  verifyTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import type { HealthStatus } from "../util/health.js";
import { formatPathFixResult, selfHealPath } from "../util/pathfix.js";
import { isUpToDate } from "../util/version.js";

/** Run the doctor command: health check. */
export async function run(offline: boolean, fix: boolean, opts: RunOpts): Promise<number> {
  colors.banner("toksave doctor", "quick health check");

  // ── Per-agent wiring status ─────────────────────────────
  for (const agent of ALL_AGENTS) {
    const det = detectAgent(agent.id);
    if (!det.installed) {
      colors.raw(
        `  ${pc.dim(colors.BULLET)} ${colors.pad(agent.label, 14)}${pc.dim("not installed")}`,
      );
      continue;
    }

    const missing: string[] = [];
    for (const tool of ALL_TOOLS) {
      if (verifyTool(agent.id, tool.id) === false) {
        missing.push(tool.label);
      }
    }

    if (missing.length === 0) {
      colors.raw(
        `  ${pc.green(colors.CHECK)} ${colors.pad(agent.label, 14)}${pc.dim("all tools wired")}`,
      );
    } else {
      colors.raw(
        `  ${pc.yellow(colors.WARN)} ${colors.pad(agent.label, 14)}${pc.yellow(`missing: ${missing.join(", ")}`)}`,
      );
    }
  }

  // ── Tool versions ───────────────────────────────────────
  if (!offline) {
    console.log();
    let outdated = 0;

    for (const tool of ALL_TOOLS) {
      const installed = toolInstalledVersion(tool.id);
      const latest = await toolLatestVersion(tool.id);
      const label = colors.pad(tool.label, 14);

      if (installed && latest) {
        const instStr = installed.startsWith("v") ? installed : `v${installed}`;
        const latestStr = latest.startsWith("v") ? latest : `v${latest}`;
        if (isUpToDate(installed, latest)) {
          colors.raw(`  ${pc.green(colors.CHECK)} ${pc.dim(label)}${pc.dim(instStr)}`);
        } else {
          outdated++;
          colors.raw(
            `  ${pc.yellow(colors.ARROW_UP)} ${pc.dim(`${label}${instStr}`)}${pc.green(` → ${latestStr}`)}`,
          );
        }
      } else if (installed) {
        const instStr =
          installed === "skill"
            ? installed
            : installed.startsWith("v")
              ? installed
              : `v${installed}`;
        colors.raw(`  ${pc.green(colors.CHECK)} ${pc.dim(label)}${pc.dim(instStr)}`);
      } else {
        colors.raw(`  ${pc.dim(colors.BULLET)} ${pc.dim(label)}${pc.dim("not installed")}`);
      }
    }

    console.log();
    if (outdated > 0) {
      colors.warn(`${outdated} update(s) available — run \`toksave update\``);
    } else {
      colors.ok("All up to date.");
    }
  }

  // ── PATH auto-fix ────────────────────────────────────────
  if (fix) {
    const msg = formatPathFixResult(selfHealPath());
    if (msg) colors.ok(msg);
  }

  // ── Tool health ─────────────────────────────────────────
  const unhealthy = ALL_TOOLS.map((tool) => ({ tool, health: toolHealthCheck(tool.id) })).filter(
    ({ health }) => !health.healthy,
  );

  if (unhealthy.length > 0) {
    console.log();
    for (const { tool, health } of unhealthy) {
      const label = colors.pad(tool.label, 14);
      colors.raw(`  ${pc.yellow(colors.WARN)} ${label}${pc.yellow("unhealthy")}`);
      printHealthIssues(health);

      if (fix) {
        const result = await toolRepair(tool.id, opts);
        const icon = result.success ? pc.green(colors.CHECK) : pc.red(colors.CROSS);
        colors.raw(`  ${icon} ${colors.pad(tool.label, 14)}${result.message}`);
        if (result.healthAfterRepair) {
          const status = result.healthAfterRepair.healthy
            ? pc.green("healthy")
            : pc.yellow("unhealthy");
          colors.raw(
            `  ${pc.dim(colors.BULLET)} ${colors.pad(tool.label, 14)}after repair: ${status}`,
          );
          printHealthIssues(result.healthAfterRepair);
        }
      }
    }

    if (!fix) {
      console.log();
      colors.info("Run `toksave doctor --fix` to repair unhealthy tools.");
    }
  }

  // ── Fix suggestion ──────────────────────────────────────
  const broken = ALL_AGENTS.some((a) => {
    const det = detectAgent(a.id);
    if (!det.installed) return false;
    return ALL_TOOLS.some((t) => verifyTool(a.id, t.id) === false);
  });

  if (broken) {
    console.log();
    colors.info("Run `toksave` to fix.");
  }

  printRepoFooter();
  console.log();
  return 0;
}

function printRepoFooter(): void {
  if (process.env.TOKSAVE_TEST === "1") return;
  const repoURL = "https://github.com/agungprasastia/toksave";
  const issuesURL = `${repoURL}/issues`;
  console.log();
  console.log(`  ${pc.dim("─".repeat(52))}`);
  console.log(`  ${pc.dim("If toksave helps, please star it here: ")}${pc.cyan(repoURL)}`);
  console.log(
    `  ${pc.dim("If you hit any issue or have ideas, please raise it here: ")}${pc.cyan(issuesURL)}`,
  );
}

function printHealthIssues(health: HealthStatus): void {
  for (const issue of health.issues) {
    const icon = issue.severity === "error" ? pc.red(colors.CROSS) : pc.yellow(colors.WARN);
    colors.raw(`    ${icon} ${issue.message}`);
    if (issue.remediation) colors.raw(`      ${pc.dim(issue.remediation)}`);
  }
}
