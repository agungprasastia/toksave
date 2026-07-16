import * as paths from "./paths.js";

export interface McpSpawn {
  command: string;
  args: string[];
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
