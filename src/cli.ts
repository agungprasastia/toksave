import { Command } from "commander";
import type { AgentId, RunOpts, ToolId } from "./registry.js";
import { parseAgentId, parseToolId } from "./registry.js";
import { toksaveVersion } from "./util/version.js";

export interface ParsedCli {
  command: "init" | "doctor" | "update" | "uninstall" | "self-update";
  agents: AgentId[];
  tools: ToolId[];
  opts: RunOpts;
  offline: boolean;
}

export function parseCli(argv: string[]): ParsedCli {
  const result: ParsedCli = {
    command: "init",
    agents: [],
    tools: [],
    opts: { dryRun: false, upgrade: false, verbose: false, yes: false },
    offline: false,
  };

  const program = new Command();

  program
    .name("toksave")
    .version(toksaveVersion(), "-V, --version")
    .description(
      "Zero-config token-saver for AI coding agents.\n\n" +
        "Installs and wires RTK, Caveman, CodeGraph, and Context-Mode\n" +
        "into Claude Code, OpenCode, Codex, and Antigravity.",
    )
    .option("-a, --agents <ids...>", "target specific agents (claude,opencode,codex,antigravity)")
    .option("-t, --tools <ids...>", "target specific tools (rtk,caveman,codegraph,context-mode)")
    .option("-n, --dry-run", "show what would happen without making changes", false)
    .option("-v, --verbose", "print detailed output", false)
    .option("-y, --yes", "skip interactive prompts, auto-select detected agents", false)
    .action((options) => {
      result.command = "init";
      applyGlobalOpts(result, options);
    });

  program
    .command("doctor")
    .description("Health check — show what is wired and what is broken")
    .option("--offline", "skip remote version checks", false)
    .action((options) => {
      result.command = "doctor";
      result.offline = options.offline ?? false;
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("update")
    .description("Update all token-saving tools to latest versions")
    .action(() => {
      result.command = "update";
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("uninstall")
    .description("Remove toksave wiring from agents")
    .action(() => {
      result.command = "uninstall";
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("self-update")
    .description("Update the toksave CLI itself")
    .action(() => {
      result.command = "self-update";
      applyGlobalOpts(result, program.opts());
    });

  program.parse(argv);

  return result;
}

function applyGlobalOpts(result: ParsedCli, opts: any): void {
  result.opts.dryRun = opts.dryRun ?? false;
  result.opts.verbose = opts.verbose ?? false;
  result.opts.yes = opts.yes ?? false;

  if (opts.agents) {
    for (const raw of opts.agents) {
      for (const s of raw.split(",")) {
        const id = parseAgentId(s.trim());
        if (id) result.agents.push(id);
      }
    }
  }

  if (opts.tools) {
    for (const raw of opts.tools) {
      for (const s of raw.split(",")) {
        const id = parseToolId(s.trim());
        if (id) result.tools.push(id);
      }
    }
  }
}
