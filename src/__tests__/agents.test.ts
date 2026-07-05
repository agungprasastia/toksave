import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import * as antigravity from "../agents/antigravity.js";
import * as claude from "../agents/claude.js";
import * as codex from "../agents/codex.js";
import * as opencode from "../agents/opencode.js";
import { readJsonFile } from "../config/json.js";
import type { RunOpts, ToolId } from "../registry.js";
import * as paths from "../util/paths.js";

const opts: RunOpts = { dryRun: false, upgrade: false, verbose: false, yes: true };
const tool: ToolId = "codegraph";
const agents = [claude, opencode, codex, antigravity] as const;

let tmp = "";
let oldHome: string | undefined;
let oldUserProfile: string | undefined;
let oldAppData: string | undefined;
let oldLocalAppData: string | undefined;

beforeEach(() => {
  oldHome = process.env.HOME;
  oldUserProfile = process.env.USERPROFILE;
  oldAppData = process.env.APPDATA;
  oldLocalAppData = process.env.LOCALAPPDATA;

  tmp = mkdtempSync(join(tmpdir(), "toksave-agents-test-"));
  process.env.HOME = join(tmp, "home");
  process.env.USERPROFILE = join(tmp, "home");
  process.env.APPDATA = join(tmp, "AppData", "Roaming");
  process.env.LOCALAPPDATA = join(tmp, "AppData", "Local");
});

afterEach(() => {
  restoreEnv("HOME", oldHome);
  restoreEnv("USERPROFILE", oldUserProfile);
  restoreEnv("APPDATA", oldAppData);
  restoreEnv("LOCALAPPDATA", oldLocalAppData);
  rmSync(tmp, { recursive: true, force: true });
});

describe("agent RTK enforcement", () => {
  test("Claude wires RTK through a PreToolUse hook and keeps instruction rules", async () => {
    await claude.wire("rtk", opts);

    const settings = readJsonFile(paths.claudePaths().settings) as Record<string, unknown>;
    const hooks = settings.hooks as Record<string, unknown>;
    const preToolUse = hooks.PreToolUse as { matcher?: string; hooks?: { command?: string }[] }[];
    expect(preToolUse.some((group) => group.hooks?.[0]?.command?.includes("rtk-hook claude"))).toBe(
      true,
    );
    expect(paths.readFile(paths.claudePaths().agentsMd)).toContain("RTK_START");
    expect(claude.verify("rtk")).toBe(true);
  });

  test("OpenCode wires RTK through a plugin file and keeps instruction rules", async () => {
    await opencode.wire("rtk", opts);

    const plugin = readOpenCodeRtkPlugin();
    expect(plugin).toContain("tool.execute.before");
    expect(plugin).toContain('input.tool !== "bash"');
    expect(plugin).toContain("rtk ");
    expect(paths.readFile(paths.opencodePaths().agentsMd)).toContain("RTK_START");
    expect(opencode.verify("rtk")).toBe(true);
  });

  test("OpenCode RTK plugin wire is idempotent and unwire removes only enforcement", async () => {
    await opencode.wire("rtk", opts);
    await opencode.wire("rtk", opts);
    expect(count(readOpenCodeRtkPlugin(), "tool.execute.before")).toBe(1);

    await opencode.unwire("rtk", opts);
    expect(existsSync(opencodeRtkPluginPath())).toBe(false);
    expect(paths.readFile(paths.opencodePaths().agentsMd) ?? "").not.toContain("RTK_START");
    expect(opencode.verify("rtk")).toBe(false);
  });

  test("Codex unwire(rtk) handles missing hooks.json without writing null", async () => {
    expect(existsSync(paths.codexPaths().hooks)).toBe(false);

    await codex.unwire("rtk", opts);

    // If hooks.json gets created by getOrCreateObject logic, it should contain valid JSON `{}`
    // or simply not exist, but it should NOT contain the literal string "null".
    if (existsSync(paths.codexPaths().hooks)) {
      const content = readFileSync(paths.codexPaths().hooks, "utf-8");
      expect(content).not.toBe("null");
    }
  });
});

describe("agent MCP wiring", () => {
  test("wire creates missing config and verify sees the MCP entry", async () => {
    expect(configFilesExist()).toEqual({
      claude: false,
      opencode: false,
      codex: false,
      antigravity: false,
    });

    for (const agent of agents) {
      await agent.wire(tool, opts);
      expect(agent.verify(tool)).toBe(true);
    }

    expect(readClaudeMcpKeys()).toEqual(["codegraph"]);
    expect(readOpenCodeMcpKeys()).toEqual(["codegraph"]);
    expect(readCodexConfig()).toContain("[mcp_servers.codegraph]");
    expect(readAntigravityMcpKeys()).toEqual(["codegraph"]);
  });

  test("wire is idempotent when called twice", async () => {
    for (const agent of agents) {
      await agent.wire(tool, opts);
      await agent.wire(tool, opts);
      expect(agent.verify(tool)).toBe(true);
    }

    expect(readClaudeMcpKeys()).toEqual(["codegraph"]);
    expect(readOpenCodeMcpKeys()).toEqual(["codegraph"]);
    expect(count(readCodexConfig(), "[mcp_servers.codegraph]")).toBe(1);
    expect(readAntigravityMcpKeys()).toEqual(["codegraph"]);
  });

  test("unwire before wire does not crash and verify stays false", async () => {
    for (const agent of agents) {
      await agent.unwire(tool, opts);
      expect(agent.verify(tool)).toBe(false);
    }
  });

  test("Antigravity atomic write rolls back all files on failure", async () => {
    // Phase 1: wire codegraph successfully to create initial state
    await antigravity.wire("codegraph", opts);
    const mcpFiles = paths.antigravityMcpFiles();
    expect(mcpFiles.length).toBeGreaterThan(0);

    // Capture initial configs
    const initialConfigs = mcpFiles.map((f) => ({
      file: f,
      content: paths.readFile(f) ?? "",
    }));

    // Phase 2: make one target file read-only to force write failure
    const targetFile = mcpFiles[0];
    if (targetFile) {
      const { chmodSync } = await import("node:fs");
      chmodSync(targetFile, 0o444); // read-only

      // Phase 3: try to wire context-mode (should fail and rollback)
      try {
        await antigravity.wire("context-mode", opts);
        expect(true).toBe(false); // Should not reach here
      } catch (err) {
        expect(err).toBeDefined();
      }

      // Phase 4: restore permissions for cleanup
      chmodSync(targetFile, 0o644);

      // Phase 5: verify rollback - codegraph config still intact, context-mode not added
      for (const { file, content } of initialConfigs) {
        expect(paths.readFile(file)).toBe(content);
      }
      expect(antigravity.verify("codegraph")).toBe(true);
      expect(antigravity.verify("context-mode")).toBe(false);
    }
  });
});

function restoreEnv(key: string, value: string | undefined): void {
  if (value === undefined) {
    delete process.env[key];
    return;
  }
  process.env[key] = value;
}

function configFilesExist(): Record<string, boolean> {
  return {
    claude: existsSync(paths.claudePaths().globalJson),
    opencode: existsSync(paths.opencodePaths().config),
    codex: existsSync(paths.codexPaths().config),
    antigravity: paths.antigravityMcpFiles().every((file) => existsSync(file)),
  };
}

function readClaudeMcpKeys(): string[] {
  const cfg = readJsonFile(paths.claudePaths().globalJson) as Record<string, unknown>;
  return Object.keys((cfg.mcpServers as Record<string, unknown>) ?? {});
}

function readOpenCodeMcpKeys(): string[] {
  const cfg = readJsonFile(paths.opencodePaths().config) as Record<string, unknown>;
  return Object.keys((cfg.mcp as Record<string, unknown>) ?? {});
}

function opencodeRtkPluginPath(): string {
  return join(paths.opencodePaths().dir, "plugins", "toksave-rtk.js");
}

function readOpenCodeRtkPlugin(): string {
  return readFileSync(opencodeRtkPluginPath(), "utf-8");
}

function readCodexConfig(): string {
  return readFileSync(paths.codexPaths().config, "utf-8");
}

function readAntigravityMcpKeys(): string[] {
  const keySets = paths.antigravityMcpFiles().map((file) => {
    const cfg = readJsonFile(file) as Record<string, unknown>;
    return Object.keys((cfg.mcpServers as Record<string, unknown>) ?? {});
  });
  expect(keySets.every((keys) => keys.join("\0") === keySets[0]?.join("\0"))).toBe(true);
  return keySets[0] ?? [];
}

function count(value: string, needle: string): number {
  return value.split(needle).length - 1;
}
