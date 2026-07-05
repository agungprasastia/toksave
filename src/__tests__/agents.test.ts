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
