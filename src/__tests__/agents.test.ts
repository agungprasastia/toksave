import { afterEach, beforeEach, describe, expect, type Mock, spyOn, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import * as antigravity from "../agents/antigravity.js";
import * as claude from "../agents/claude.js";
import * as codex from "../agents/codex.js";
import * as copilot from "../agents/copilot.js";
import * as droid from "../agents/droid.js";
import * as opencode from "../agents/opencode.js";
import { readJsonFile } from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import * as detect from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner } from "../util/unified-block.js";

interface AgentModule {
  wire(tool: ToolId, opts: RunOpts): Promise<boolean>;
  unwire(tool: ToolId, opts: RunOpts): Promise<boolean>;
  verify(tool: ToolId): boolean | null;
  detect(): Detection;
}

const opts: RunOpts = { dryRun: false, upgrade: false, verbose: false, yes: true };
const tool: ToolId = "codegraph";

let tmp = "";
let oldHome: string | undefined;
let oldUserProfile: string | undefined;
let oldAppData: string | undefined;
let oldLocalAppData: string | undefined;
let isOnPathSpy: Mock<typeof detect.isOnPath>;

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

  isOnPathSpy = spyOn(detect, "isOnPath").mockImplementation((name) => {
    if (name === "rtk") return true;
    return false;
  });
  // Ensure copilot IDE root stays inside temp
  copilot.setIdeProjectRoot(join(tmp, "proj"));
});

afterEach(() => {
  restoreEnv("HOME", oldHome);
  restoreEnv("USERPROFILE", oldUserProfile);
  restoreEnv("APPDATA", oldAppData);
  restoreEnv("LOCALAPPDATA", oldLocalAppData);
  rmSync(tmp, { recursive: true, force: true });

  if (isOnPathSpy) {
    isOnPathSpy.mockRestore();
  }
  copilot.setIdeProjectRoot("");
});

describe("agent RTK enforcement", () => {
  test("Claude wires RTK through a PreToolUse hook", async () => {
    await claude.wire("rtk", opts);

    const settings = readJsonFile(paths.claudePaths().settings) as Record<string, unknown>;
    const hooks = settings.hooks as Record<string, unknown>;
    const preToolUse = hooks.PreToolUse as { matcher?: string; hooks?: { command?: string }[] }[];
    expect(preToolUse.some((group) => group.hooks?.[0]?.command?.includes("rtk-hook claude"))).toBe(
      true,
    );
    expect(claude.verify("rtk")).toBe(true);
  });

  test("OpenCode wires RTK through a plugin file", async () => {
    await opencode.wire("rtk", opts);

    const plugin = readOpenCodeRtkPlugin();
    expect(plugin).toContain("tool.execute.before");
    expect(plugin).toContain('input.tool !== "bash"');
    expect(plugin).toContain("rtk ");
    expect(opencode.verify("rtk")).toBe(true);
  });

  test("OpenCode RTK plugin wire is idempotent and unwire removes only enforcement", async () => {
    await opencode.wire("rtk", opts);
    await opencode.wire("rtk", opts);
    expect(count(readOpenCodeRtkPlugin(), "tool.execute.before")).toBe(1);

    await opencode.unwire("rtk", opts);
    expect(existsSync(opencodeRtkPluginPath())).toBe(false);
    expect(opencode.verify("rtk")).toBe(false);
  });

  test("Codex unwire(rtk) handles missing hooks.json without writing null", async () => {
    expect(existsSync(paths.codexPaths().hooks)).toBe(false);

    await codex.unwire("rtk", opts);

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

    for (const agent of [claude, opencode, codex, antigravity, copilot, droid]) {
      await (agent as AgentModule).wire(tool, opts);
      expect((agent as AgentModule).verify(tool)).toBe(true);
    }

    expect(readClaudeMcpKeys()).toEqual(["codegraph"]);
    expect(readOpenCodeMcpKeys()).toEqual(["codegraph"]);
    expect(readCodexConfig()).toContain("[mcp_servers.codegraph]");
    expect(readAntigravityMcpKeys()).toEqual(["codegraph"]);
  });

  test("wire is idempotent when called twice", async () => {
    for (const agent of [claude, opencode, codex, antigravity]) {
      await (agent as AgentModule).wire(tool, opts);
      await (agent as AgentModule).wire(tool, opts);
      expect((agent as AgentModule).verify(tool)).toBe(true);
    }

    expect(readClaudeMcpKeys()).toEqual(["codegraph"]);
    expect(readOpenCodeMcpKeys()).toEqual(["codegraph"]);
    expect(count(readCodexConfig(), "[mcp_servers.codegraph]")).toBe(1);
    expect(readAntigravityMcpKeys()).toEqual(["codegraph"]);
  });

  test("unwire before wire does not crash and verify stays false", async () => {
    for (const agent of [claude, opencode, codex, antigravity, copilot, droid]) {
      await (agent as AgentModule).unwire(tool, opts);
      expect((agent as AgentModule).verify(tool)).toBe(false);
    }
  });

  test("Claude auto-index SessionStart hook is installed on wire", async () => {
    await claude.wire("codegraph", opts);
    const cfg = readJsonFile(paths.claudePaths().settings) as Record<string, unknown>;
    const hooks = cfg?.hooks as Record<string, unknown> | undefined;
    expect(hooks?.SessionStart).toBeDefined();
    const entry = (hooks!.SessionStart as unknown[])[0] as Record<string, unknown>;
    expect((entry!.hooks as unknown[])[0]).toMatchObject({
      command: "toksave index --auto",
      timeout: 120000,
    });
  });

  test("Claude auto-index hook is idempotent", async () => {
    await claude.wire("codegraph", opts);
    await claude.wire("codegraph", opts);
    const cfg = readJsonFile(paths.claudePaths().settings) as Record<string, unknown>;
    const hooks = cfg?.hooks as Record<string, unknown> | undefined;
    expect(hooks!.SessionStart).toBeDefined();
    expect((hooks!.SessionStart as unknown[]).length).toBe(1);
  });

  test("Claude auto-index hook is removed on unwire", async () => {
    await claude.wire("codegraph", opts);
    await claude.unwire("codegraph", opts);
    const cfg = readJsonFile(paths.claudePaths().settings) as Record<string, unknown>;
    const hooks = cfg?.hooks as Record<string, unknown> | undefined;
    if (hooks?.SessionStart) {
      const remaining = (hooks.SessionStart as unknown[]).filter(
        (g) =>
          (g as { hooks?: { command?: string }[] })?.hooks?.[0]?.command === "toksave index --auto",
      );
      expect(remaining.length).toBe(0);
    }
  });

  test("Antigravity atomic write rolls back all files on failure", async () => {
    await antigravity.wire("codegraph", opts);
    const mcpFiles = paths.antigravityMcpFiles();
    expect(mcpFiles.length).toBeGreaterThan(1);

    const initialConfigs = mcpFiles.map((f) => ({
      file: f,
      content: paths.readFile(f) ?? "",
    }));

    const targetFile = mcpFiles[mcpFiles.length - 1]!;
    const writeSpy = spyOn(paths, "writeFile").mockImplementation((p, content) => {
      if (p === targetFile) {
        throw new Error("Simulated write failure");
      }
      const fs = require("node:fs");
      const path = require("node:path");
      fs.mkdirSync(path.dirname(p), { recursive: true });
      const tmpf = `${p}.${process.pid}.tmp`;
      fs.writeFileSync(tmpf, content, "utf-8");
      fs.renameSync(tmpf, p);
    });

    try {
      await antigravity.wire("context-mode", opts);
      expect(true).toBe(false);
    } catch (err) {
      expect(err).toBeDefined();
    }

    writeSpy.mockRestore();

    for (const { file, content } of initialConfigs) {
      expect(paths.readFile(file)).toBe(content);
    }
    expect(antigravity.verify("codegraph")).toBe(true);
    expect(antigravity.verify("context-mode")).toBe(false);
  });
});

describe("agent unified block wiring", () => {
  test("caveman via unified block creates Principles + caveman section", async () => {
    await claude.wire("caveman", opts);
    const md = paths.readFile(paths.claudePaths().agentsMd) ?? "";
    expect(md).toContain("## Response Style (caveman)");
    expect(md).toContain("## Principles");
    expect(hasOwner("claude", "caveman")).toBe(true);
    expect(claude.verify("caveman")).toBe(true);
  });

  test("codegraph + caveman share file with index section when >=2 owners", async () => {
    await claude.wire("caveman", opts);
    await claude.wire("codegraph", opts);
    const md = paths.readFile(paths.claudePaths().agentsMd) ?? "";
    expect(md).toContain("## Response Style (caveman)");
    expect(md).toContain("## Code Index (codegraph)");
    expect(md).toContain("# Agent Instructions"); // index section when >=2
    expect(hasOwner("claude", "caveman")).toBe(true);
    expect(hasOwner("claude", "codegraph")).toBe(true);
  });

  test("remove caveman leaves codegraph", async () => {
    await claude.wire("caveman", opts);
    await claude.wire("codegraph", opts);
    await claude.unwire("caveman", opts);
    const md = paths.readFile(paths.claudePaths().agentsMd) ?? "";
    expect(md).not.toContain("## Response Style (caveman)");
    expect(md).toContain("## Code Index (codegraph)");
  });

  test("principles and ponytail wiring via unified block", async () => {
    await claude.wire("principles", opts);
    expect(hasOwner("claude", "principles")).toBe(true);
    await claude.wire("ponytail", opts);
    expect(hasOwner("claude", "ponytail")).toBe(true);
    const md = paths.readFile(paths.claudePaths().agentsMd) ?? "";
    expect(md).toContain("## Principles");
    expect(md).toContain("## Build Discipline (ponytail)");
  });

  test("legacy fence stripped on writeOwner", async () => {
    const p = paths.claudePaths().agentsMd;
    paths.ensureDir(paths.claudePaths().dir);
    paths.writeFile(p, "<!-- CAVEMAN_START -->\nold block\n<!-- CAVEMAN_END -->\n");
    await claude.wire("caveman", opts);
    const md = paths.readFile(p) ?? "";
    expect(md).not.toContain("CAVEMAN_START");
    expect(md).toContain("## Response Style (caveman)");
  });

  test("copilot and droid wiring via unified block", async () => {
    await copilot.wire("caveman", opts);
    expect(hasOwner("copilot", "caveman")).toBe(true);
    expect(copilot.verify("caveman")).toBe(true);

    await droid.wire("caveman", opts);
    expect(hasOwner("droid", "caveman")).toBe(true);
    expect(droid.verify("caveman")).toBe(true);
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

describe("agent × tool wiring matrix (all 36 combos)", () => {
  const agentMap: Record<string, AgentModule> = {
    claude,
    opencode,
    codex,
    antigravity,
    copilot,
    droid,
  };
  const tools: ToolId[] = ["rtk", "caveman", "codegraph", "context-mode", "ponytail", "principles"];

  for (const [aname, agent] of Object.entries(agentMap)) {
    for (const tl of tools) {
      test(`${aname} × ${tl} wire/unwire cycle`, async () => {
        const res1 = await agent.wire(tl, opts);
        expect(res1).toBe(true);
        const v1 = agent.verify(tl);
        expect(v1).toBe(true);

        const res2 = await agent.unwire(tl, opts);
        expect(res2).toBe(true);
        const v2 = agent.verify(tl);
        expect(v2).toBe(false);
      });
    }
  }

  for (const tl of ["rtk", "caveman", "codegraph", "context-mode", "ponytail", "principles"] as ToolId[]) {
    test(`dry-run wire does not pass verify for ${tl} (caught by verify-after-wire)`, async () => {
      const dryOpts: RunOpts = { ...opts, dryRun: true };
      for (const [aname, agent] of Object.entries(agentMap)) {
        const res = await agent.wire(tl, dryOpts);
        expect(res).toBe(true);
        const v = agent.verify(tl);
        expect(v).toBe(false);
      }
    });
  }
});

describe("agent detection config-dir fallback", () => {
  test("Codex detect() uses config-dir fallback only in test mode", () => {
    const spy = spyOn(detect, "findBinaryIn").mockReturnValue(null);
    paths.ensureDir(paths.codexPaths().dir);
    expect(codex.detect()).toEqual({ installed: true, source: "config" });
    spy.mockRestore();
  });

  test("OpenCode detect() uses config-dir fallback only in test mode", () => {
    const spy = spyOn(detect, "findBinaryIn").mockReturnValue(null);
    paths.ensureDir(paths.opencodePaths().dir);
    expect(opencode.detect()).toEqual({ installed: true, source: "config" });
    spy.mockRestore();
  });

  test("Codex detect() returns not-installed when no cli and no config dir", () => {
    const spy = spyOn(detect, "findBinaryIn").mockReturnValue(null);
    expect(codex.detect()).toEqual({ installed: false, source: "" });
    spy.mockRestore();
  });

  test("OpenCode detect() returns not-installed when no cli, desktop, or config dir", () => {
    const spy = spyOn(detect, "findBinaryIn").mockReturnValue(null);
    expect(opencode.detect()).toEqual({ installed: false, source: "" });
    spy.mockRestore();
  });
});

describe("OpenCode auto-index plugin", () => {
  test("plugin file is created on install", () => {
    opencode.installOpencodeAutoIndexPlugin();
    const pluginPath = `${paths.opencodePaths().dir}/plugins/toksave-autoindex.js`;
    expect(existsSync(pluginPath)).toBe(true);
    const content = readFileSync(pluginPath, "utf-8");
    expect(content).toContain("toksave index --auto");
  });

  test("plugin install is idempotent", () => {
    opencode.installOpencodeAutoIndexPlugin();
    opencode.installOpencodeAutoIndexPlugin();
    const pluginPath = `${paths.opencodePaths().dir}/plugins/toksave-autoindex.js`;
    expect(existsSync(pluginPath)).toBe(true);
  });

  test("plugin is removed on remove", () => {
    opencode.installOpencodeAutoIndexPlugin();
    opencode.removeOpencodeAutoIndexPlugin();
    const pluginPath = `${paths.opencodePaths().dir}/plugins/toksave-autoindex.js`;
    expect(existsSync(pluginPath)).toBe(false);
  });

  test("OpenCode codegraph wire installs auto-index plugin", async () => {
    await opencode.wire("codegraph", opts);
    expect(opencode.hasOpencodeAutoIndexPlugin()).toBe(true);
  });

  test("OpenCode codegraph unwire removes auto-index plugin", async () => {
    await opencode.wire("codegraph", opts);
    await opencode.unwire("codegraph", opts);
    expect(opencode.hasOpencodeAutoIndexPlugin()).toBe(false);
  });
});
