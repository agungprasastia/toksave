import * as clack from "@clack/prompts";
import pc from "picocolors";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  type AgentId,
  type RunOpts,
  type ToolId,
  agentInfo,
  detectAgent,
  installTool,
  toolInfo,
  toolInstalledVersion,
  wireTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import { recordWire } from "../util/manifest.js";
import { checkNode } from "../util/npm.js";
import { type SelectOption, isInteractive, multiSelect } from "../util/prompt.js";

/** Run the default install flow: install tools → detect agents → wire. */
export async function run(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts,
): Promise<number> {
  colors.banner("toksave", "global token-saver for AI agents");

  // ── Step 1: Check dependencies ──────────────────────────
  const tools = ALL_TOOLS.filter((t) => toolsFilter.length === 0 || toolsFilter.includes(t.id));

  const maxNode = Math.max(0, ...tools.map((t) => t.minNodeMajor));
  const nodeOk = maxNode === 0 || checkNode(maxNode);
  if (!nodeOk && maxNode > 0) {
    colors.warn(`Node.js >= ${maxNode} required for some tools. https://nodejs.org/en/download`);
  }

  // ── Step 2: Install tools ───────────────────────────────
  const s = clack.spinner();
  for (const tool of tools) {
    s.start(`Installing ${tool.label}`);
    if (tool.channel === "npm" && !nodeOk) {
      s.stop(`${pc.yellow(colors.WARN)} ${tool.label} — needs Node.js`);
      continue;
    }
    try {
      const ok = await installTool(tool.id, opts);
      s.stop(
        ok
          ? `${pc.green(colors.CHECK)} ${tool.label}`
          : `${pc.yellow(colors.WARN)} ${tool.label} — skipped`,
      );
    } catch (e: any) {
      s.stop(`${pc.red(colors.CROSS)} ${tool.label} — ${firstLine(e.message)}`);
    }
  }

  // ── Step 3: Detect agents ───────────────────────────────
  const detected: { id: AgentId; source: string }[] = [];
  for (const a of ALL_AGENTS) {
    const d = detectAgent(a.id);
    if (d.installed) detected.push({ id: a.id, source: d.source });
  }

  // ── Step 4: Pick agents ─────────────────────────────────
  let requested: AgentId[];

  if (agentsFilter.length > 0) {
    requested = agentsFilter;
  } else if (opts.yes || !isInteractive()) {
    // Auto-select detected agents
    requested = detected.map((d) => d.id);
  } else {
    const options: SelectOption[] = ALL_AGENTS.map((a) => {
      const det = detected.find((d) => d.id === a.id);
      return {
        value: a.id,
        label: a.label,
        disabled: !det,
        hint: det ? det.source : a.homepage,
        selected: !!det,
      };
    });

    const picked = await multiSelect("Select agents to wire toksave into", options);
    requested = picked as AgentId[];
  }

  if (requested.length === 0) {
    colors.raw("  Nothing selected.");
    console.log();
    return 0;
  }

  // ── Step 5: Wire tools into agents ──────────────────────
  const failures: { id: AgentId; failed: string[] }[] = [];

  for (const agentId of requested) {
    const det = detected.find((d) => d.id === agentId);
    if (!det) {
      const info = agentInfo(agentId);
      colors.warn(`${info.label} not installed — install it first: ${info.homepage}`);
      continue;
    }

    const info = agentInfo(agentId);
    s.start(`Wiring ${info.label}`);
    const failedTools: string[] = [];

    for (const tool of tools) {
      try {
        const ok = await wireTool(agentId, tool.id, opts);
        if (ok && !opts.dryRun) {
          recordWire(agentId, tool.id, toolInstalledVersion(tool.id) ?? undefined);
        }
        if (!ok) failedTools.push(tool.label);
      } catch {
        failedTools.push(tool.label);
      }
    }

    if (failedTools.length === 0) {
      s.stop(`${pc.green(colors.CHECK)} ${info.label}`);
    } else {
      s.stop(`${pc.yellow(colors.WARN)} ${info.label} — ${failedTools.join(", ")} not wired`);
      failures.push({ id: agentId, failed: failedTools });
    }
  }

  // ── Step 6: Summary ─────────────────────────────────────
  console.log();
  const wired = requested
    .filter((id) => !failures.some((f) => f.id === id))
    .filter((id) => detected.some((d) => d.id === id))
    .map((id) => agentInfo(id).label);

  if (wired.length > 0) {
    colors.ok(`Equipped ${wired.join(", ")}.`);
  }

  for (const f of failures) {
    colors.warn(
      `${agentInfo(f.id).label}: ${f.failed.join(", ")} not wired. Run \`toksave doctor\` for details.`,
    );
  }

  // Version table
  console.log();
  printVersionTable(tools);
  console.log();

  return failures.length > 0 ? 1 : 0;
}

function printVersionTable(tools: typeof ALL_TOOLS): void {
  for (const tool of tools) {
    const installed = toolInstalledVersion(tool.id);
    const label = colors.pad(tool.label, 14);
    if (installed) {
      colors.ok(`${label}${installed}`);
    } else {
      colors.raw(`  ${colors.BULLET} ${label}${pc.dim("not installed")}`);
    }
  }
}

function firstLine(s: string): string {
  return s.split("\n")[0] ?? s;
}
