import { readFileSync } from "node:fs";

/**
 * RTK hook for Codex and Antigravity PreToolUse.
 * Receives tool invocation JSON on stdin, prefixes Bash commands with `rtk`.
 */
export function runRtkHook(): number {
  try {
    const input = readFileSync(0, "utf-8");
    if (!input) return 0;

    const req = JSON.parse(input) as {
      tool_name?: string;
      tool_input?: { command?: string };
    };

    const toolName = req.tool_name ?? "";
    const command = req.tool_input?.command ?? "";

    // Only intercept Bash-like tools
    if (!isBashTool(toolName) || !command.trim()) return 0;

    // Already uses rtk — pass through
    const trimmed = command.trim();
    if (trimmed.startsWith("rtk ") || trimmed === "rtk") return 0;

    // Prefix with rtk
    const modified = {
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        modifiedToolInput: { command: `rtk ${trimmed}` },
      },
    };

    console.log(JSON.stringify(modified));
    return 0;
  } catch {
    return 0; // On error, don't block the tool call
  }
}

function isBashTool(name: string): boolean {
  return /^(Bash|run_command|execute_command|cmd|sh|pwsh)$/i.test(name);
}
