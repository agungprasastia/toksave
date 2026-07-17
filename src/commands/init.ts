import boxen from "boxen";
import Table from "cli-table3";
import pc from "picocolors";
import {
  type AgentId,
  ALL_AGENTS,
  ALL_TOOLS,
  agentInfo,
  detectAgent,
  installTool,
  type RunOpts,
  type ToolId,
  toolInstalledVersion,
  verifyTool,
  wireTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import { ensureDeps } from "../util/deps.js";
import { recordWire } from "../util/manifest.js";
import { formatPathFixResult, selfHealPath } from "../util/pathfix.js";
import { Progress } from "../util/progress.js";
import { isInteractive, multiSelect, type SelectOption } from "../util/prompt.js";

/** Run the default install flow: install tools → detect agents → wire. */
export async function run(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts,
): Promise<number> {
  colors.banner("toksave", "global token-saver for AI agents");

  // ── Step 1: Check dependencies ──────────────────────────
  const tools = ALL_TOOLS.filter((t) => toolsFilter.length === 0 || toolsFilter.includes(t.id));

  const minNode = Math.max(0, ...tools.map((t) => t.minNodeMajor));
  const hasNpmTools = tools.some((t) => t.channel === "npm");
  const deps = ensureDeps(hasNpmTools, false, minNode);

  // ── Step 2: Install tools ───────────────────────────────
  const s = new Progress();
  const installedTools = new Set<ToolId>();
  for (const tool of tools) {
    s.start(`Installing ${tool.label}`);
    if (tool.channel === "npm" && !deps.ok) {
      s.stop(`${pc.yellow(colors.WARN)} ${tool.label} — needs Node.js`);
      continue;
    }
    try {
      const ok = await installTool(tool.id, opts);
      await new Promise((r) => setTimeout(r, 200)); // UX delay so progress bar is visible
      if (ok) installedTools.add(tool.id);
      s.stop(
        ok
          ? `${pc.green(colors.CHECK)} ${tool.label}`
          : `${pc.yellow(colors.WARN)} ${tool.label} — skipped`,
      );
    } catch (err: unknown) {
      const e = err as Error;
      s.stop(`${pc.red(colors.CROSS)} ${tool.label} — ${firstLine(e.message || String(e))}`);
    }
  }

  if (!opts.dryRun) {
    const msg = formatPathFixResult(selfHealPath());
    if (msg) colors.ok(msg);
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
        selected: false,
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
      if (!installedTools.has(tool.id)) {
        failedTools.push(tool.label);
        continue;
      }
      try {
        const ok = await wireTool(agentId, tool.id, opts);
        await new Promise((r) => setTimeout(r, 200)); // UX delay so progress bar is visible
        if (ok && !opts.dryRun) {
          recordWire(agentId, tool.id, toolInstalledVersion(tool.id) ?? undefined);
          if (verifyTool(agentId, tool.id) === false) {
            failedTools.push(tool.label);
            continue;
          }
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
    console.log(
      boxen(pc.green(`Equipped ${pc.bold(wired.join(", "))}.`), {
        padding: 1,
        margin: { bottom: 1 },
        borderStyle: "round",
        borderColor: "green",
      }),
    );
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
  const table = new Table({
    chars: {
      top: "",
      "top-mid": "",
      "top-left": "",
      "top-right": "",
      bottom: "",
      "bottom-mid": "",
      "bottom-left": "",
      "bottom-right": "",
      left: "",
      "left-mid": "",
      mid: "",
      "mid-mid": "",
      right: "",
      "right-mid": "",
      middle: "   ",
    },
    style: { "padding-left": 0, "padding-right": 0, border: [] },
  });

  for (const tool of tools) {
    const installed = toolInstalledVersion(tool.id);
    if (installed) {
      table.push([`  ${pc.green(colors.CHECK)}`, tool.label, installed]);
    } else {
      table.push([`  ${colors.BULLET}`, pc.dim(tool.label), pc.dim("not installed")]);
    }
  }
  console.log(table.toString());
}

function firstLine(s: string): string {
  return s.split("\n")[0] ?? s;
}
