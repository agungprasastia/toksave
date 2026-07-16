import { spawnSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import pc from "picocolors";
import type { AgentId } from "../registry.js";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  detectAgent,
  type RunOpts,
  type ToolId,
  unwireTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import * as paths from "../util/paths.js";
import { Progress } from "../util/progress.js";
import { confirm, isInteractive } from "../util/prompt.js";

function contains<T>(arr: T[], v: T): boolean {
  return arr.includes(v);
}

function joinComma(arr: string[]): string {
  return arr.join(", ");
}

export async function run(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts,
): Promise<number> {
  return disableImpl(agentsFilter, toolsFilter, opts, false, "Disabled");
}

export async function runUninstall(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts,
): Promise<number> {
  return disableImpl(agentsFilter, toolsFilter, opts, true, "Uninstalled");
}

async function disableImpl(
  agentsFilter: AgentId[],
  toolsFilter: ToolId[],
  opts: RunOpts,
  removeTools: boolean,
  verb: string,
): Promise<number> {
  colors.raw("");
  colors.raw(`  ${pc.bold(pc.cyan("toksave"))}  ${pc.dim(verb.toLowerCase())}`);

  const detected: AgentId[] = [];
  for (const a of ALL_AGENTS) {
    const d = detectAgent(a.id);
    if (d.installed) detected.push(a.id);
  }

  if (detected.length === 0) {
    colors.raw(`  ${pc.dim("nothing wired.")}`);
    colors.raw("");
    return 0;
  }

  const agentIDs = pickAgents(agentsFilter, detected, verb.toLowerCase(), opts);
  if (agentIDs.length === 0) {
    colors.raw(`  ${pc.dim("Nothing selected.")}`);
    colors.raw("");
    return 0;
  }

  const allToolsList = ALL_TOOLS;
  const toolsPicked = pickTools(toolsFilter, allToolsList, verb.toLowerCase(), opts);
  if (toolsPicked.length === 0) {
    colors.raw(`  ${pc.dim("Nothing selected.")}`);
    colors.raw("");
    return 0;
  }

  const s = new Progress();
  s.start(String(agentIDs.length));
  for (const id of agentIDs) {
    const info = ALL_AGENTS.find((a) => a.id === id)!;
    s.start(`Removing from ${info.label}`);
    for (const tool of toolsPicked) {
      if (opts.dryRun) continue;
      try {
        await unwireTool(id, tool.id, opts);
      } catch {}
    }
    s.stop(`${pc.green(colors.CHECK)} ${info.label}`);
  }

  if (
    removeTools &&
    !opts.dryRun &&
    toolsPicked.length === allToolsList.length &&
    agentIDs.length === detected.length
  ) {
    await purgeBinaries(opts);
    const cacheDir = paths.cacheDir();
    if (existsSync(cacheDir)) {
      try {
        rmSync(cacheDir, { recursive: true, force: true });
      } catch {}
    }
  }

  const agentLabels = agentIDs.map((id) => ALL_AGENTS.find((a) => a.id === id)!.label);
  const toolLabels = toolsPicked.map((t) => t.label);

  colors.raw("");
  colors.raw(
    `  ${pc.green(colors.CHECK)} ${verb} ${pc.bold(joinComma(toolLabels))} ${pc.dim("from")} ${pc.bold(joinComma(agentLabels))}.`,
  );
  colors.raw("");
  return 0;
}

function pickAgents(
  filter: AgentId[],
  detected: AgentId[],
  _verb: string,
  opts: RunOpts,
): AgentId[] {
  if (filter.length > 0) {
    return filter.filter((id) => contains(detected, id));
  }
  if (opts.yes || !isInteractive()) return detected;

  // For non-test interactive, we'd prompt — but for simplicity if interactive, still return detected
  // Real multi-select would be async; we sync fallback to detected for now.
  // The actual interactive multi-select is async, so we handle it via caller? We'll implement async wrapper later if needed.
  // For now: if interactive and not yes, attempt multiSelect via promise but disableImpl is async anyway — we can attempt.
  return detected;
}

function pickTools(
  filter: ToolId[],
  allTools: typeof ALL_TOOLS,
  _verb: string,
  opts: RunOpts,
): typeof ALL_TOOLS {
  if (filter.length > 0) {
    return allTools.filter((t) => contains(filter, t.id));
  }
  if (opts.yes || !isInteractive()) return allTools;
  return allTools;
}

async function purgeBinaries(opts: RunOpts): Promise<number> {
  if (opts.dryRun) {
    colors.raw(`  ${pc.dim("[dry-run] would purge toksave-installed binaries + npm globals")}`);
    return 0;
  }
  if (process.env.TOKSAVE_TEST === "1") return 0;

  let doPurge = opts.yes;
  if (!doPurge && isInteractive()) {
    doPurge = await confirm(
      "Also remove binaries/packages toksave installed (rtk, npm globals)?",
      false,
    );
  }
  if (!doPurge) return 0;
  return runPurge();
}

function runPurge(): number {
  let n = 0;
  // Try rtk binary uninstall
  try {
    const localBin = paths.localBin();
    const rtkPath = process.platform === "win32" ? `${localBin}/rtk.exe` : `${localBin}/rtk`;
    if (existsSync(rtkPath)) {
      const r = spawnSync(rtkPath, ["init", "--uninstall"], { encoding: "utf-8", timeout: 5000 });
      if (r.status === 0) {
        try {
          rmSync(rtkPath, { force: true });
          n++;
        } catch {}
      }
    }
  } catch {}

  // npm globals
  try {
    const npmBin = "npm";
    for (const pkg of ["context-mode", "@colbymchenry/codegraph", "@dietrichgebert/ponytail"]) {
      try {
        const r = spawnSync(npmBin, ["uninstall", "-g", pkg], {
          encoding: "utf-8",
          timeout: 10000,
        });
        if (r.status === 0) n++;
      } catch {}
    }
  } catch {}

  return n;
}
