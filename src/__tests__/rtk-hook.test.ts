import { describe, expect, test } from "bun:test";

const hookInput = JSON.stringify({
  tool_name: "Bash",
  tool_input: { command: "bun test" },
});

describe("rtk-hook", () => {
  test("Claude Code output uses hookSpecificOutput.updatedInput", () => {
    const result = runHook("claude");
    expect(result.exitCode).toBe(0);

    const output = JSON.parse(result.stdout.toString()) as {
      hookSpecificOutput: { updatedInput?: { command?: string }; modifiedToolInput?: unknown };
    };
    expect(output.hookSpecificOutput.updatedInput?.command).toBe("rtk bun test");
    expect(output.hookSpecificOutput.modifiedToolInput).toBeUndefined();
  });

  test("Codex output keeps modifiedToolInput shape", () => {
    const result = runHook("codex");
    expect(result.exitCode).toBe(0);

    const output = JSON.parse(result.stdout.toString()) as {
      hookSpecificOutput: { modifiedToolInput?: { command?: string }; updatedInput?: unknown };
    };
    expect(output.hookSpecificOutput.modifiedToolInput?.command).toBe("rtk bun test");
    expect(output.hookSpecificOutput.updatedInput).toBeUndefined();
  });
});

function runHook(agent: string): Bun.SpawnSyncReturns<Buffer> {
  return Bun.spawnSync({
    cmd: [process.execPath, "src/index.ts", "rtk-hook", agent],
    stdin: Buffer.from(hookInput),
    stdout: "pipe",
    stderr: "pipe",
  });
}
