import { Command } from "commander";
import type { AgentId, RunOpts, ToolId } from "./registry.js";
import { parseAgentId, parseToolId } from "./registry.js";
import { toksaveVersion } from "./util/version.js";

export type CommandType =
  | "init"
  | "doctor"
  | "update"
  | "uninstall"
  | "self-update"
  | "codex-perm-hook"
  | "rtk-hook"
  | "context-mode-hook"
  | "runmcp"
  | "index";

export interface ParsedCli {
  command: CommandType;
  agents: AgentId[];
  tools: ToolId[];
  opts: RunOpts;
  offline: boolean;
  fix: boolean;
}

export function parseCli(argv: string[]): ParsedCli {
  const result: ParsedCli = {
    command: "init",
    agents: [],
    tools: [],
    opts: { dryRun: false, upgrade: false, verbose: false, yes: false },
    offline: false,
    fix: false,
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
    .option("--fix", "repair unhealthy tool installations", false)
    .action((options) => {
      result.command = "doctor";
      result.offline = options.offline ?? false;
      result.fix = options.fix ?? false;
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

  program
    .command("codex-perm-hook")
    .description("Internal hook for Codex permissions")
    .action(() => {
      result.command = "codex-perm-hook";
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("rtk-hook")
    .description("Internal hook for RTK command prefixing")
    .argument("[args...]")
    .allowUnknownOption()
    .action(() => {
      result.command = "rtk-hook";
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("context-mode-hook")
    .description("Internal hook for Context-Mode integration")
    .argument("[args...]")
    .allowUnknownOption()
    .action(() => {
      result.command = "context-mode-hook";
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("runmcp")
    .description("Internal hook to proxy MCP execution securely")
    .allowUnknownOption()
    .action(() => {
      result.command = "runmcp";
      applyGlobalOpts(result, program.opts());
    });

  program
    .command("index")
    .description("Build per-project indexes (codegraph) in the current dir")
    .action(() => {
      result.command = "index";
      applyGlobalOpts(result, program.opts());
    });

  program.parse(argv);

  return result;
}

function applyGlobalOpts(result: ParsedCli, opts: Record<string, unknown>): void {
  result.opts.dryRun = (opts.dryRun as boolean) ?? false;
  result.opts.verbose = (opts.verbose as boolean) ?? false;
  result.opts.yes = (opts.yes as boolean) ?? false;

  if (opts.agents) {
    for (const raw of opts.agents as string[]) {
      for (const s of raw.split(",")) {
        const id = parseAgentId(s.trim());
        if (id) result.agents.push(id);
      }
    }
  }

  if (opts.tools) {
    for (const raw of opts.tools as string[]) {
      for (const s of raw.split(",")) {
        const id = parseToolId(s.trim());
        if (id) result.tools.push(id);
      }
    }
  }
}
