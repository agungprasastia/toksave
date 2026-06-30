import pc from "picocolors";
import * as clack from "@clack/prompts";
import {
  ALL_AGENTS, ALL_TOOLS, type AgentId, type ToolId,
  parseAgentId, parseToolId, agentInfo, toolInfo,
  detectAgent, unwireTool, type RunOpts,
} from "../registry.js";
import { isInteractive, multiSelect, type SelectOption } from "../util/prompt.js";
import * as colors from "../util/colors.js";
import { removeWire, clearManifest } from "../util/manifest.js";
import { cacheDir } from "../util/paths.js";
import { existsSync, rmSync } from "fs";

/** Run the uninstall command: unwire tools from agents. */
export async function run(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts
): Promise<number> {
  colors.banner("toksave", "uninstall");

  // ── Detect installed agents ──────────────────────────────
  const detected = ALL_AGENTS
    .filter((a) => detectAgent(a.id).installed)
    .map((a) => a.id);

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
  const tools: ToolId[] =
    toolsFilter.length > 0 ? toolsFilter : ALL_TOOLS.map((t) => t.id);

  // ── Unwire ──────────────────────────────────────────────
  const s = clack.spinner();
  for (const agentId of agentIds) {
    const info = agentInfo(agentId);
    s.start(`Unwiring ${info.label}`);
    for (const toolId of tools) {
      if (!opts.dryRun) {
        await unwireTool(agentId, toolId, opts);
        removeWire(agentId, toolId);
      }
    }
    s.stop(`${pc.green(colors.CHECK)} ${info.label}`);
  }

  // ── Cleanup cache if full removal ───────────────────────
  if (!opts.dryRun && tools.length === ALL_TOOLS.length && agentIds.length === detected.length) {
    const cache = cacheDir();
    if (existsSync(cache)) {
      try { rmSync(cache, { recursive: true, force: true }); } catch { /* ignore */ }
    }
  }

  // ── Summary ─────────────────────────────────────────────
  console.log();
  const agentLabels = agentIds.map((id) => agentInfo(id).label);
  const toolLabels = tools.map((id) => toolInfo(id).label);
  colors.ok(`Uninstalled ${toolLabels.join(", ")} from ${agentLabels.join(", ")}.`);
  console.log();
  return 0;
}
