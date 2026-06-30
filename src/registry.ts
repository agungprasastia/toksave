// ─── Enums ───────────────────────────────────────────────────

export type AgentId = "claude" | "opencode" | "codex" | "antigravity";
export type ToolId = "rtk" | "caveman" | "codegraph" | "context-mode";
export type Channel = "github" | "npm" | "skill";

// ─── Info types ──────────────────────────────────────────────

export interface AgentInfo {
  id: AgentId;
  label: string;
  homepage: string;
  cliBin: string;
}

export interface ToolInfo {
  id: ToolId;
  label: string;
  homepage: string;
  channel: Channel;
  minNodeMajor: number;
}

export interface Detection {
  installed: boolean;
  source: string;
}

export interface RunOpts {
  dryRun: boolean;
  upgrade: boolean;
  verbose: boolean;
  yes: boolean;
}

// ─── Static registries ──────────────────────────────────────

export const ALL_AGENTS: AgentInfo[] = [
  {
    id: "claude",
    label: "Claude Code",
    homepage: "https://github.com/anthropics/claude-code",
    cliBin: "claude",
  },
  {
    id: "opencode",
    label: "OpenCode",
    homepage: "https://github.com/anomalyco/opencode",
    cliBin: "opencode",
  },
  {
    id: "codex",
    label: "Codex",
    homepage: "https://github.com/openai/codex",
    cliBin: "codex",
  },
  {
    id: "antigravity",
    label: "Antigravity",
    homepage: "https://antigravity.google",
    cliBin: "agy",
  },
];

export const ALL_TOOLS: ToolInfo[] = [
  {
    id: "rtk",
    label: "RTK",
    homepage: "https://github.com/rtk-ai/rtk",
    channel: "github",
    minNodeMajor: 0,
  },
  {
    id: "caveman",
    label: "Caveman",
    homepage: "https://github.com/JuliusBrussee/caveman",
    channel: "skill",
    minNodeMajor: 0,
  },
  {
    id: "codegraph",
    label: "CodeGraph",
    homepage: "https://github.com/colbymchenry/codegraph",
    channel: "npm",
    minNodeMajor: 18,
  },
  {
    id: "context-mode",
    label: "Context-Mode",
    homepage: "https://github.com/mksglu/context-mode",
    channel: "npm",
    minNodeMajor: 22,
  },
];

// ─── Lookups ─────────────────────────────────────────────────

export function agentInfo(id: AgentId): AgentInfo {
  return ALL_AGENTS.find((a) => a.id === id)!;
}

export function toolInfo(id: ToolId): ToolInfo {
  return ALL_TOOLS.find((t) => t.id === id)!;
}

export function parseAgentId(s: string): AgentId | null {
  const lower = s.toLowerCase().trim();
  if (["claude", "opencode", "codex", "antigravity"].includes(lower)) {
    return lower as AgentId;
  }
  return null;
}

export function parseToolId(s: string): ToolId | null {
  const lower = s.toLowerCase().trim();
  const map: Record<string, ToolId> = {
    rtk: "rtk",
    caveman: "caveman",
    codegraph: "codegraph",
    "context-mode": "context-mode",
    contextmode: "context-mode",
  };
  return map[lower] ?? null;
}

// ─── Dispatch functions ─────────────────────────────────────

import * as antigravity from "./agents/antigravity.js";
import * as claude from "./agents/claude.js";
import * as codex from "./agents/codex.js";
import * as opencode from "./agents/opencode.js";
import * as cavemanTool from "./tools/caveman.js";
import * as codegraphTool from "./tools/codegraph.js";
import * as contextModeTool from "./tools/context-mode.js";
import * as rtkTool from "./tools/rtk.js";

const agentModules = { claude, opencode, codex, antigravity };
const toolModules = {
  rtk: rtkTool,
  caveman: cavemanTool,
  codegraph: codegraphTool,
  "context-mode": contextModeTool,
};

export function detectAgent(id: AgentId): Detection {
  return agentModules[id].detect();
}

export async function installTool(id: ToolId, opts: RunOpts): Promise<boolean> {
  return toolModules[id].install(opts);
}

export async function wireTool(agent: AgentId, tool: ToolId, opts: RunOpts): Promise<boolean> {
  return agentModules[agent].wire(tool, opts);
}

export async function unwireTool(agent: AgentId, tool: ToolId, opts: RunOpts): Promise<boolean> {
  return agentModules[agent].unwire(tool, opts);
}

export function verifyTool(agent: AgentId, tool: ToolId): boolean | null {
  return agentModules[agent].verify(tool);
}

export function toolInstalledVersion(id: ToolId): string | null {
  return toolModules[id].installedVersion();
}

export function toolLatestVersion(id: ToolId): string | null {
  return toolModules[id].latestVersion();
}
