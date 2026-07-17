import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import { getOrCreateObject, readJsonFile, writeJsonFile } from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner, removeOwner, writeOwner } from "../util/unified-block.js";

let ideProjectRoot = "";

export function setIdeProjectRoot(p: string): void {
  ideProjectRoot = p;
}

function ideRoot(): string {
  return ideProjectRoot || process.cwd();
}

// IDE (VS Code) project-scoped paths
function copilotIdeHooksDir(): string {
  return join(ideRoot(), ".github", "hooks");
}
function copilotIdeHooksFile(name: string): string {
  return join(copilotIdeHooksDir(), name);
}
function copilotIdeMcpFile(): string {
  return join(ideRoot(), ".vscode", "mcp.json");
}
function copilotIdeInstructionsFile(): string {
  return join(ideRoot(), ".github", "copilot-instructions.md");
}

function copilotHooksFile(name: string): string {
  return join(paths.copilotPaths().hooksDir, name);
}

// ─── Detection ───────────────────────────────────────────────

export function detect(): Detection {
  const hasCli = !!findBinaryIn("copilot", paths.copilotKnownBinDirs());
  if (hasCli) return { installed: true, source: "cli" };
  if (process.env.NODE_ENV !== "test") return { installed: false, source: "" };
  const dir = paths.copilotPaths().dir;
  if (existsSync(dir)) return { installed: true, source: "config" };
  // Also check for project-level copilot IDE markers
  if (
    existsSync(join(ideRoot(), ".github", "copilot-instructions.md")) ||
    existsSync(join(ideRoot(), ".vscode", "mcp.json"))
  ) {
    return { installed: true, source: "config" };
  }
  return { installed: false, source: "" };
}

// ─── Wire / Unwire / Verify ──────────────────────────────────

export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "rtk":
      if (!opts.dryRun) {
        installCopilotRtkHook();
        installCopilotIdeRtkHook();
      }
      return true;
    case "codegraph":
      if (!opts.dryRun) {
        configureCopilotMcp("codegraph");
        configureCopilotIdeMcp("codegraph");
        writeOwner("copilot", "codegraph");
        installCopilotCodegraphIndexHook();
        installCopilotIdeCodegraphIndexHook();
        syncCopilotIdeInstructions();
      }
      return true;
    case "context-mode":
      if (!opts.dryRun) {
        configureCopilotMcp("context-mode");
        configureCopilotIdeMcp("context-mode");
        writeOwner("copilot", "context-mode");
        installCopilotContextModeHook();
        installCopilotIdeContextModeHook();
        syncCopilotIdeInstructions();
      }
      return true;
    case "caveman":
      if (!opts.dryRun) {
        writeOwner("copilot", "caveman");
        syncCopilotIdeInstructions();
      }
      return true;
    case "ponytail":
      if (!opts.dryRun) {
        writeOwner("copilot", "ponytail");
        syncCopilotIdeInstructions();
      }
      return true;
    case "principles":
      if (!opts.dryRun) {
        writeOwner("copilot", "principles");
        syncCopilotIdeInstructions();
      }
      return true;
    default:
      return false;
  }
}

export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "rtk":
      removeCopilotRtkHook();
      removeCopilotIdeRtkHook();
      return true;
    case "codegraph":
      removeCopilotMcp("codegraph");
      removeCopilotIdeMcp("codegraph");
      removeCopilotCodegraphIndexHook();
      removeCopilotIdeCodegraphIndexHook();
      removeOwner("copilot", "codegraph");
      return true;
    case "context-mode":
      removeCopilotMcp("context-mode");
      removeCopilotIdeMcp("context-mode");
      removeCopilotContextModeHook();
      removeCopilotIdeContextModeHook();
      removeOwner("copilot", "context-mode");
      return true;
    case "caveman":
      removeOwner("copilot", "caveman");
      return true;
    case "ponytail":
      removeOwner("copilot", "ponytail");
      return true;
    case "principles":
      removeOwner("copilot", "principles");
      return true;
    default:
      return false;
  }
}

export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "rtk":
      return hasCopilotRtkHook();
    case "codegraph":
      return copilotMcpHas("codegraph") && hasCopilotCodegraphIndexHook();
    case "context-mode":
      return copilotMcpHas("context-mode");
    case "caveman":
      return hasOwner("copilot", "caveman");
    case "ponytail":
      return hasOwner("copilot", "ponytail");
    case "principles":
      return hasOwner("copilot", "principles");
    default:
      return null;
  }
}

// ─── RTK hook (flat format) ──────────────────────────────────

function copilotRtkHookCommand(): string {
  const tok = paths.toksaveAbs();
  return `${tok} rtk-hook copilot`;
}

function installCopilotRtkHook(): void {
  const p = paths.copilotPaths();
  paths.ensureDir(p.hooksDir);
  const cmd = copilotRtkHookCommand();
  const flat = { type: "command", command: cmd, timeout: 10 };
  const hooks: Record<string, unknown> = {
    PreToolUse: [flat],
    preToolUse: [flat],
    PostToolUse: [flat],
    postToolUse: [flat],
  };
  writeJsonFile(copilotHooksFile("tokless-rtk.json"), { version: 1, hooks });
  ensureCopilotRtkCommandApproval();
}

function removeCopilotRtkHook(): void {
  try {
    rmSync(copilotHooksFile("tokless-rtk.json"), { force: true });
  } catch {}
}

function hasCopilotRtkHook(): boolean {
  const raw = paths.readFile(copilotHooksFile("tokless-rtk.json"));
  return !!raw?.includes("rtk-hook copilot");
}

function installCopilotIdeRtkHook(): void {
  paths.ensureDir(copilotIdeHooksDir());
  const cmd = copilotRtkHookCommand();
  const entry = { type: "command", command: cmd, timeout: 10 };
  const hooks: Record<string, unknown> = {
    PreToolUse: [entry],
    PostToolUse: [entry],
  };
  writeJsonFile(copilotIdeHooksFile("tokless-rtk.json"), { version: 1, hooks });
}

function removeCopilotIdeRtkHook(): void {
  try {
    rmSync(copilotIdeHooksFile("tokless-rtk.json"), { force: true });
  } catch {}
}

function _hasCopilotIdeRtkHook(): boolean {
  const raw = paths.readFile(copilotIdeHooksFile("tokless-rtk.json"));
  return !!raw?.includes("rtk-hook copilot");
}

function ensureCopilotRtkCommandApproval(): void {
  const file = join(paths.copilotPaths().dir, "permissions-config.json");
  const raw = paths.readFile(file);
  if (!raw) return;
  try {
    const cfg = JSON.parse(raw) as Record<string, unknown>;
    const locs = cfg.locations as Record<string, unknown> | undefined;
    if (!locs) return;
    let changed = false;
    for (const key of Object.keys(locs)) {
      const loc = locs[key] as Record<string, unknown> | undefined;
      if (!loc) continue;
      const approvals = (loc.tool_approvals as unknown[]) ?? [];
      if (
        approvals.some((a) => {
          const m = a as Record<string, unknown>;
          return (
            m.kind === "commands" &&
            Array.isArray(m.commandIdentifiers) &&
            (m.commandIdentifiers as string[]).includes("rtk")
          );
        })
      )
        continue;
      approvals.push({ kind: "commands", commandIdentifiers: ["rtk"] });
      loc.tool_approvals = approvals as never;
      changed = true;
    }
    if (changed) {
      writeJsonFile(file, cfg);
    }
  } catch {}
}

// ─── Codegraph index hooks ───────────────────────────────────

function installCopilotCodegraphIndexHook(): void {
  const p = paths.copilotPaths();
  paths.ensureDir(p.hooksDir);
  const tok = paths.toksaveAbs();
  const cmd = `${tok} copilot-hook codegraph-index`;
  const hook = { type: "command", command: cmd };
  const hooks: Record<string, unknown> = { sessionStart: [hook] };
  writeJsonFile(copilotHooksFile("tokless-codegraph-index.json"), { version: 1, hooks });
}

function removeCopilotCodegraphIndexHook(): void {
  try {
    rmSync(copilotHooksFile("tokless-codegraph-index.json"), { force: true });
  } catch {}
}

function hasCopilotCodegraphIndexHook(): boolean {
  const raw = paths.readFile(copilotHooksFile("tokless-codegraph-index.json"));
  return !!raw?.includes("copilot-hook codegraph-index");
}

function installCopilotIdeCodegraphIndexHook(): void {
  paths.ensureDir(copilotIdeHooksDir());
  const tok = paths.toksaveAbs();
  const cmd = `${tok} copilot-hook codegraph-index`;
  const entry = { type: "command", command: cmd, timeout: 120 };
  const hooks: Record<string, unknown> = { PostToolUse: [entry] };
  writeJsonFile(copilotIdeHooksFile("tokless-codegraph-index.json"), { version: 1, hooks });
}

function removeCopilotIdeCodegraphIndexHook(): void {
  try {
    rmSync(copilotIdeHooksFile("tokless-codegraph-index.json"), { force: true });
  } catch {}
}

// ─── Context-mode hooks ──────────────────────────────────────

function installCopilotContextModeHook(): void {
  const p = paths.copilotPaths();
  paths.ensureDir(p.hooksDir);
  const events = [
    { event: "preToolUse", token: "pretooluse" },
    { event: "postToolUse", token: "posttooluse" },
    { event: "sessionStart", token: "sessionstart" },
    { event: "userPromptSubmitted", token: "userpromptsubmit" },
    { event: "agentStop", token: "stop" },
    { event: "preCompact", token: "precompact" },
  ];
  const hooks: Record<string, unknown> = {};
  for (const e of events) {
    hooks[e.event] = [{ type: "command", command: `context-mode hook copilot-cli ${e.token}` }];
  }
  writeJsonFile(copilotHooksFile("context-mode.json"), { version: 1, hooks });
}

function removeCopilotContextModeHook(): void {
  try {
    rmSync(copilotHooksFile("context-mode.json"), { force: true });
  } catch {}
}

function installCopilotIdeContextModeHook(): void {
  paths.ensureDir(copilotIdeHooksDir());
  const events = ["PreToolUse", "PostToolUse", "SessionStart", "Stop"];
  const tokens = ["pretooluse", "posttooluse", "sessionstart", "stop"];
  const hooks: Record<string, unknown> = {};
  for (let i = 0; i < events.length; i++) {
    hooks[events[i]!] = [
      { type: "command", command: `context-mode hook copilot-vscode ${tokens[i]}`, timeout: 10 },
    ];
  }
  writeJsonFile(copilotIdeHooksFile("context-mode.json"), { version: 1, hooks });
}

function removeCopilotIdeContextModeHook(): void {
  try {
    rmSync(copilotIdeHooksFile("context-mode.json"), { force: true });
  } catch {}
}

// ─── MCP ─────────────────────────────────────────────────────

function configureCopilotMcp(toolId: string): void {
  const p = paths.copilotPaths();
  paths.ensureDir(p.dir);
  const cfg = (readJsonFile(p.mcpConfig) as Record<string, unknown>) ?? {};
  const servers = getOrCreateObject(cfg, "mcpServers");
  const abs = paths.toksaveAbs();
  const args =
    toolId === "codegraph" ? ["runmcp", "codegraph", "serve", "--mcp"] : ["runmcp", toolId];
  const desired: Record<string, unknown> = {
    type: "local",
    command: abs,
    args,
    tools: ["*"],
  };
  if (toolId === "context-mode") {
    desired.env = { CONTEXT_MODE_PLATFORM: "copilot-cli", CONTEXT_MODE_COPILOT_PLUGIN: "1" };
  }
  (servers as Record<string, unknown>)[toolId] = desired;
  writeJsonFile(p.mcpConfig, cfg);
}

function removeCopilotMcp(toolId: string): void {
  const p = paths.copilotPaths();
  const cfg = readJsonFile(p.mcpConfig) as Record<string, unknown> | null;
  if (!cfg) return;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  if (!mcp?.[toolId]) return;
  delete mcp[toolId];
  writeJsonFile(p.mcpConfig, cfg);
}

function copilotMcpHas(toolId: string): boolean {
  const p = paths.copilotPaths();
  const cfg = readJsonFile(p.mcpConfig) as Record<string, unknown> | null;
  if (!cfg) return false;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  return !!mcp?.[toolId];
}

function configureCopilotIdeMcp(toolId: string): void {
  const file = copilotIdeMcpFile();
  paths.ensureDir(join(ideRoot(), ".vscode"));
  const cfg = (readJsonFile(file) as Record<string, unknown>) ?? {};
  const servers = getOrCreateObject(cfg, "servers");
  const abs = paths.toksaveAbs();
  const args =
    toolId === "codegraph" ? ["runmcp", "codegraph", "serve", "--mcp"] : ["runmcp", toolId];
  const desired: Record<string, unknown> = {
    type: "stdio",
    command: abs,
    args,
  };
  if (toolId === "context-mode") {
    desired.env = { CONTEXT_MODE_PLATFORM: "copilot-vscode", CONTEXT_MODE_COPILOT_PLUGIN: "1" };
  }
  (servers as Record<string, unknown>)[toolId] = desired;
  writeJsonFile(file, cfg);
}

function removeCopilotIdeMcp(toolId: string): void {
  const file = copilotIdeMcpFile();
  const cfg = readJsonFile(file) as Record<string, unknown> | null;
  if (!cfg) return;
  const servers = cfg.servers as Record<string, unknown> | undefined;
  if (!servers?.[toolId]) return;
  delete servers[toolId];
  writeJsonFile(file, cfg);
}

function syncCopilotIdeInstructions(): void {
  const cliPath = paths.copilotPaths().instructions;
  const body = paths.readFile(cliPath);
  if (!body?.trim()) return;
  const ideFile = copilotIdeInstructionsFile();
  paths.ensureDir(join(ideRoot(), ".github"));
  paths.writeFile(ideFile, `${body.trimEnd()}\n`);
}
