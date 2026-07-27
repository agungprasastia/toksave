import { existsSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { findBinary, findBinaryIn, resolveNode } from "./detect.js";
import * as paths from "./paths.js";

export interface McpSpawn {
  command: string;
  args: string[];
}

const MCP_TOOL_BINS: Record<string, string[]> = {
  codegraph: ["codegraph"],
  "context-mode": ["context-mode"],
};

function knownUserBinDirs(): string[] {
  const dirs = [paths.localBin(), join(paths.localBin(), "..", "share", "mise", "shims")];
  // localBin is ~/.local/bin — also check sibling cargo/bun
  try {
    const home = require("node:os").homedir() as string;
    dirs.push(join(home, ".bun", "bin"), join(home, ".cargo", "bin"));
  } catch {}
  return dirs;
}

function isToksaveCommand(command: string): boolean {
  if (!command) return false;
  if (command === "toksave" || command === "toksave.exe") return true;
  if (command.endsWith("/toksave") || command.endsWith("\\toksave")) return true;
  if (command.endsWith("toksave.exe")) return true;
  try {
    return command === paths.toksaveAbs();
  } catch {
    return false;
  }
}

/** Resolve a bare MCP tool name or path to an executable on disk/PATH. */
export function resolveMcpExecutable(toolOrPath: string): string | null {
  if (!toolOrPath) return null;
  if (isAbsolute(toolOrPath) || toolOrPath.startsWith("./") || toolOrPath.startsWith("../")) {
    return existsSync(toolOrPath) ? toolOrPath : null;
  }

  const candidates = MCP_TOOL_BINS[toolOrPath] ?? [toolOrPath];
  for (const name of candidates) {
    const found = findBinary(name);
    if (found) return found;
  }

  // Last resort: known user bin dirs (GUI agents often lack mise on PATH)
  for (const name of candidates) {
    const found = findBinaryIn(name, knownUserBinDirs());
    if (found) return found;
  }
  return null;
}

/** Extract the MCP tool binary name from a toksave runmcp argv list. */
export function toolIdFromRunmcpArgs(args: string[]): string | null {
  if (!args.length) return null;
  let i = 0;
  if (args[0] === "runmcp") i = 1;
  if (args[i] === "--agent") i += 2;
  return args[i] ?? null;
}

/**
 * Validate that an MCP config entry can actually start.
 * Config presence alone is not enough — runmcp historically accepted wiring
 * while the target binary was missing / unresolvable.
 */
export function mcpSpawnHealthy(
  command: string,
  args: string[] = [],
): {
  healthy: boolean;
  reason?: string;
} {
  if (!command) return { healthy: false, reason: "missing command" };

  const runmcpArgs =
    args[0] === "runmcp" ? args : isToksaveCommand(command) ? ["runmcp", ...args] : null;

  if (runmcpArgs) {
    const tool = toolIdFromRunmcpArgs(runmcpArgs);
    if (!tool) return { healthy: false, reason: "runmcp missing tool name" };
    if (tool === "serve" || tool.startsWith("-")) {
      return { healthy: false, reason: `runmcp args look malformed: ${args.join(" ")}` };
    }
    const resolved = resolveMcpExecutable(tool);
    if (!resolved) {
      return {
        healthy: false,
        reason: `MCP tool binary not found on PATH: ${tool}`,
      };
    }
    if (!resolveNode()) {
      return { healthy: false, reason: "node not found on PATH (required for MCP npm shims)" };
    }
    return { healthy: true };
  }

  if (!existsSync(command) && !findBinary(command)) {
    return { healthy: false, reason: `MCP command not found: ${command}` };
  }
  return { healthy: true };
}

/** Pick best way to spawn MCP server for tool — via toksave runmcp proxy for consistent pre-index + shebang handling. */
export function pickMcpSpawn(toolId: string, ...extra: string[]): McpSpawn {
  const abs = paths.toksaveAbs();
  if (toolId === "codegraph") {
    return {
      command: abs,
      args: ["runmcp", "codegraph", "serve", "--mcp", ...extra].filter(
        (a, i, arr) => i === 0 || !arr.slice(0, i).includes(a),
      ),
    };
  }
  // If extra already includes serve --mcp etc, keep
  const base = ["runmcp", toolId];
  return { command: abs, args: [...base, ...extra] };
}

/** Wrap spawn with auto-index for specific agent — inject --agent flag so runmcp knows to pre-index. */
export function wrapAutoIndex(agent: string, spawn: McpSpawn): McpSpawn {
  if (spawn.args[0] === "runmcp") {
    // Avoid double --agent
    if (spawn.args.includes("--agent")) return spawn;
    return { command: spawn.command, args: ["runmcp", "--agent", agent, ...spawn.args.slice(1)] };
  }
  return spawn;
}
