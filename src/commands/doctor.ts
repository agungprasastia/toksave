import pc from "picocolors";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  detectAgent,
  toolInstalledVersion,
  toolLatestVersion,
  verifyTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import { isUpToDate } from "../util/version.js";

/** Run the doctor command: health check. */
export async function run(offline: boolean): Promise<number> {
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
      const latest = toolLatestVersion(tool.id);
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
  console.log();
  return 0;
}
