import { spawnSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import pc from "picocolors";
import {
  type AgentId,
  ALL_AGENTS,
  ALL_TOOLS,
  agentInfo,
  detectAgent,
  type RunOpts,
  type ToolId,
  toolInfo,
  unwireTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import { removeWire } from "../util/manifest.js";
import * as paths from "../util/paths.js";
import { cacheDir } from "../util/paths.js";
import { Progress } from "../util/progress.js";
import { confirm, isInteractive, multiSelect, type SelectOption } from "../util/prompt.js";

/** Run the uninstall command: unwire tools from agents. */
export async function run(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts,
): Promise<number> {
  colors.banner("toksave", "uninstall");

  // ── Detect installed agents ──────────────────────────────
  const detected = ALL_AGENTS.filter((a) => detectAgent(a.id).installed).map((a) => a.id);

  if (detected.length === 0) {
    colors.raw("  Nothing wired.");
    console.log();
    return 0;
  }

  // ── Pick agents ─────────────────────────────────────────
  let agentIds: AgentId[];

  if (agentsFilter.length > 0) {
    agentIds = agentsFilter.filter((id) => detected.includes(id));
  } else if (opts.yes || !isInteractive()) {
    agentIds = [...detected];
  } else {
    const options: SelectOption[] = detected.map((id) => ({
      value: id,
      label: agentInfo(id).label,
      disabled: false,
      selected: true,
    }));
    const picked = await multiSelect("Select agents to uninstall toksave from", options);
    agentIds = picked as AgentId[];
  }

  if (agentIds.length === 0) {
    colors.raw("  Nothing selected.");
    console.log();
    return 0;
  }

  // ── Pick tools ──────────────────────────────────────────
  const tools: ToolId[] = toolsFilter.length > 0 ? toolsFilter : ALL_TOOLS.map((t) => t.id);

  // ── Unwire ──────────────────────────────────────────────
  const s = new Progress();
  s.start("Removing TokSave tools");
  for (const agentId of agentIds) {
    const info = agentInfo(agentId);
    s.update(`Unwiring ${info.label}`);
    for (const toolId of tools) {
      if (!opts.dryRun) {
        await unwireTool(agentId, toolId, opts);
        removeWire(agentId, toolId);
      }
      await new Promise((r) => setTimeout(r, 150)); // UX delay so progress bar is visible
    }
    s.stop(`${pc.green(colors.CHECK)} ${info.label}`);
  }

  // ── Cleanup cache + purge binaries if full removal ──────
  if (!opts.dryRun && tools.length === ALL_TOOLS.length && agentIds.length === detected.length) {
    const cache = cacheDir();
    if (existsSync(cache)) {
      try {
        rmSync(cache, { recursive: true, force: true });
      } catch {}
    }
    await purgeBinariesIfConfirmed(opts);
  }

  // ── Summary ─────────────────────────────────────────────
  console.log();
  const agentLabels = agentIds.map((id) => agentInfo(id).label);
  const toolLabels = tools.map((id) => toolInfo(id).label);
  colors.ok(`Uninstalled ${toolLabels.join(", ")} from ${agentLabels.join(", ")}.`);
  console.log();
  return 0;
}

async function purgeBinariesIfConfirmed(opts: RunOpts): Promise<void> {
  if (opts.dryRun || process.env.TOKSAVE_TEST === "1") return;
  let doPurge = opts.yes;
  if (!doPurge && isInteractive()) {
    doPurge = await confirm(
      "Also remove binaries/packages toksave installed (rtk, npm globals)?",
      false,
    );
  }
  if (!doPurge) return;

  // rtk
  try {
    const localBin = paths.localBin();
    const rtkPath = process.platform === "win32" ? `${localBin}/rtk.exe` : `${localBin}/rtk`;
    if (existsSync(rtkPath)) {
      spawnSync(rtkPath, ["init", "--uninstall"], { timeout: 5000 });
      try {
        rmSync(rtkPath, { force: true });
      } catch {}
    }
  } catch {}

  // npm globals
  for (const pkg of ["context-mode", "@colbymchenry/codegraph", "@dietrichgebert/ponytail"]) {
    try {
      spawnSync("npm", ["uninstall", "-g", pkg], { timeout: 10000 });
    } catch {}
  }
}
