// ─── Enums ───────────────────────────────────────────────────

export type AgentId = "claude" | "opencode" | "codex" | "antigravity" | "copilot" | "droid";
export type ToolId = "rtk" | "caveman" | "codegraph" | "context-mode" | "ponytail" | "principles";
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
  notTrackable?: boolean;
  instructionOnly?: boolean;
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
  {
    id: "copilot",
    label: "GitHub Copilot",
    homepage: "https://github.com/github/copilot-cli",
    cliBin: "copilot",
  },
  {
    id: "droid",
    label: "Factory Droid",
    homepage: "https://factory.ai",
    cliBin: "droid",
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
  {
    id: "ponytail",
    label: "Ponytail",
    homepage: "https://github.com/DietrichGebert/ponytail",
    channel: "npm",
    minNodeMajor: 0,
  },
  {
    id: "principles",
    label: "Principles",
    homepage: "https://github.com/multica-ai/andrej-karpathy-skills",
    channel: "skill",
    minNodeMajor: 0,
    notTrackable: true,
    instructionOnly: true,
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
  if (["claude", "opencode", "codex", "antigravity", "copilot", "droid"].includes(lower)) {
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
    ponytail: "ponytail",
    principles: "principles",
    "karpathy-skills": "principles",
    karpathy: "principles",
    karpathyskills: "principles",
  };
  return map[lower] ?? null;
}

// ─── Dispatch functions ─────────────────────────────────────

import * as antigravity from "./agents/antigravity.js";
import * as claude from "./agents/claude.js";
import * as codex from "./agents/codex.js";
import * as copilot from "./agents/copilot.js";
import * as droid from "./agents/droid.js";
import * as opencode from "./agents/opencode.js";
import * as cavemanTool from "./tools/caveman.js";
import * as codegraphTool from "./tools/codegraph.js";
import * as contextModeTool from "./tools/context-mode.js";
import * as ponytailTool from "./tools/ponytail.js";
import * as principlesTool from "./tools/principles.js";
import * as rtkTool from "./tools/rtk.js";
import type { HealthStatus, RepairResult } from "./util/health.js";
import { getCachedLatest, getStaleFallback, setCachedLatest } from "./util/versioncache.js";

const agentModules = { claude, opencode, codex, antigravity, copilot, droid };
const toolModules = {
  rtk: rtkTool,
  caveman: cavemanTool,
  codegraph: codegraphTool,
  "context-mode": contextModeTool,
  ponytail: ponytailTool,
  principles: principlesTool,
} as const;

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

export async function toolLatestVersion(id: ToolId): Promise<string | null> {
  // Check cache first (6h TTL)
  const cached = getCachedLatest(id);
  if (cached.hit) return cached.version;

  // Fetch from network
  const version = await toolModules[id].latestVersion();
  if (version) {
    setCachedLatest(id, version);
    return version;
  }

  // Fetch failed — fall back to stale cache if available
  return getStaleFallback(id);
}

export function toolHealthCheck(id: ToolId): HealthStatus {
  return toolModules[id].healthCheck();
}

export async function toolRepair(id: ToolId, opts: RunOpts): Promise<RepairResult> {
  return toolModules[id].repair(opts);
}
