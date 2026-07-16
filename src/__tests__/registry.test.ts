import { describe, expect, test } from "bun:test";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  agentInfo,
  parseAgentId,
  parseToolId,
  toolInfo,
} from "../registry.js";

describe("Registry", () => {
  test("ALL_AGENTS has 6 entries (including copilot, droid)", () => {
    expect(ALL_AGENTS).toHaveLength(6);
    expect(ALL_AGENTS.map((a) => a.id)).toContain("copilot");
    expect(ALL_AGENTS.map((a) => a.id)).toContain("droid");
  });

  test("ALL_TOOLS has 6 entries (including ponytail, principles)", () => {
    expect(ALL_TOOLS).toHaveLength(6);
    expect(ALL_TOOLS.map((t) => t.id)).toContain("ponytail");
    expect(ALL_TOOLS.map((t) => t.id)).toContain("principles");
  });

  test("agentInfo returns correct data", () => {
    const info = agentInfo("claude");
    expect(info.label).toBe("Claude Code");
    expect(info.cliBin).toBe("claude");
  });

  test("toolInfo returns correct data", () => {
    const info = toolInfo("rtk");
    expect(info.label).toBe("RTK");
    expect(info.channel).toBe("github");
  });

  test("parseAgentId valid", () => {
    expect(parseAgentId("claude")).toBe("claude");
    expect(parseAgentId("OPENCODE")).toBe("opencode");
    expect(parseAgentId("Codex")).toBe("codex");
    expect(parseAgentId("antigravity")).toBe("antigravity");
    expect(parseAgentId("copilot")).toBe("copilot");
    expect(parseAgentId("droid")).toBe("droid");
  });

  test("parseAgentId invalid", () => {
    expect(parseAgentId("invalid")).toBeNull();
    expect(parseAgentId("")).toBeNull();
  });

  test("parseToolId valid", () => {
    expect(parseToolId("rtk")).toBe("rtk");
    expect(parseToolId("caveman")).toBe("caveman");
    expect(parseToolId("codegraph")).toBe("codegraph");
    expect(parseToolId("context-mode")).toBe("context-mode");
    expect(parseToolId("contextmode")).toBe("context-mode");
    expect(parseToolId("ponytail")).toBe("ponytail");
    expect(parseToolId("principles")).toBe("principles");
    expect(parseToolId("karpathy-skills")).toBe("principles");
    expect(parseToolId("karpathy")).toBe("principles");
  });

  test("parseToolId invalid", () => {
    expect(parseToolId("invalid")).toBeNull();
  });

  test("context-mode requires Node 22", () => {
    const info = toolInfo("context-mode");
    expect(info.minNodeMajor).toBe(22);
  });

  test("rtk has no Node requirement", () => {
    const info = toolInfo("rtk");
    expect(info.minNodeMajor).toBe(0);
  });
});
