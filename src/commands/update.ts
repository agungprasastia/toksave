import boxen from "boxen";
import pc from "picocolors";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  detectAgent,
  installTool,
  type RunOpts,
  toolInfo,
  toolInstalledVersion,
  toolLatestVersion,
  verifyTool,
  wireTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import { Progress } from "../util/progress.js";
import { isUpToDate } from "../util/version.js";

/** Run the update command: check for tool updates and upgrade. */
export async function run(opts: RunOpts): Promise<number> {
  colors.banner("toksave update", "refresh tools to latest");

  // ── Probe versions ──────────────────────────────────────
  const changed: (typeof ALL_TOOLS)[number]["id"][] = [];

  for (const tool of ALL_TOOLS) {
    const installed = toolInstalledVersion(tool.id);
    const latest = await toolLatestVersion(tool.id);
    const label = colors.pad(tool.label, 14);

    const instStr = installed ? `v${installed}` : "not on PATH";
    const latStr = latest ? `v${latest}` : "?";

    if (installed && latest && !isUpToDate(installed, latest)) {
      changed.push(tool.id);
      colors.raw(
        `  ${pc.yellow(colors.ARROW_UP)} ${label} ${instStr} → ${latStr} ${pc.yellow("→ upgrade")}`,
      );
    } else if (!installed && latest) {
      changed.push(tool.id);
      colors.raw(`  ${pc.yellow("+")} ${label} ${instStr} → ${latStr} ${pc.yellow("→ install")}`);
    } else {
      colors.raw(
        `  ${pc.green(colors.CHECK)} ${label} ${instStr} → ${latStr} ${pc.dim("(up to date)")}`,
      );
    }
  }
  console.log();

  if (changed.length === 0) {
    colors.ok("Everything up to date.");
    console.log();
    return 0;
  }

  if (opts.dryRun) {
    const names = changed.map((id) => toolInfo(id).label);
    colors.info(`Would upgrade: ${names.join(", ")}`);
    console.log();
    return 0;
  }

  // ── Upgrade changed tools ───────────────────────────────
  const upgradeOpts: RunOpts = { ...opts, dryRun: false, upgrade: true };
  const s = new Progress();

  for (const id of changed) {
    const info = toolInfo(id);
    s.start(`Upgrading ${info.label}`);
    try {
      await installTool(id, upgradeOpts);
      s.stop(`${pc.green(colors.CHECK)} ${info.label}`);
    } catch (err: unknown) {
      const e = err as Error;
      s.stop(`${pc.red(colors.CROSS)} ${info.label} — ${String(e.message || e)}`);
    }
  }

  // ── Re-sync wiring ──────────────────────────────────────
  for (const toolId of changed) {
    for (const agent of ALL_AGENTS) {
      const det = detectAgent(agent.id);
      if (!det.installed) continue;
      if (verifyTool(agent.id, toolId) === true) {
        await wireTool(agent.id, toolId, upgradeOpts);
      }
    }
  }

  console.log();
  const names = changed.map((id) => toolInfo(id).label);
  console.log(
    boxen(pc.green(`Updated ${pc.bold(names.join(", "))}.`), {
      padding: 1,
      margin: { bottom: 1 },
      borderStyle: "round",
      borderColor: "green",
    }),
  );
  console.log();
  return 0;
}
