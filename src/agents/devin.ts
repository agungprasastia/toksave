import { existsSync } from "node:fs";
import { getOrCreateObject, readJsonFile, writeJsonFile } from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner, removeOwner, writeOwner } from "../util/unified-block.js";

const CONTEXT_MODE_TOOLS = [
  "ctx_search",
  "ctx_execute",
  "ctx_execute_file",
  "ctx_batch_execute",
  "ctx_index",
  "ctx_fetch_and_index",
];
const CODEGRAPH_TOOLS = ["codegraph_explore"];

/** Detect if Devin / Cascade is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("devin", paths.devinKnownBinDirs());
  const hasDesktop = paths.devinDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (process.env.NODE_ENV === "test" && existsSync(paths.devinPaths().dir)) {
    return { installed: true, source: "config" };
  }
  const dir = paths.devinPaths().dir;
  if (existsSync(dir) || existsSync(paths.devinPaths().mcpConfig)) {
    return { installed: true, source: "config" };
  }
  return { installed: false, source: "" };
}

/** Wire a tool into Devin. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "rtk":
      if (!opts.dryRun) installDevinRtkHook();
      return true;
    case "codegraph":
      if (!opts.dryRun) {
        configureDevinMcp("codegraph");
        writeOwner("devin", "codegraph");
      }
      return true;
    case "context-mode":
      if (!opts.dryRun) {
        configureDevinMcp("context-mode");
        writeOwner("devin", "context-mode");
      }
      return true;
    case "caveman":
      if (!opts.dryRun) writeOwner("devin", "caveman");
      return true;
    case "ponytail":
      if (!opts.dryRun) writeOwner("devin", "ponytail");
      return true;
    case "principles":
      if (!opts.dryRun) writeOwner("devin", "principles");
      return true;
    default:
      return false;
  }
}

/** Unwire a tool from Devin. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "rtk":
      removeDevinRtkHook();
      return true;
    case "codegraph":
      removeDevinMcp("codegraph");
      removeOwner("devin", "codegraph");
      return true;
    case "context-mode":
      removeDevinMcp("context-mode");
      removeOwner("devin", "context-mode");
      return true;
    case "caveman":
      removeOwner("devin", "caveman");
      return true;
    case "ponytail":
      removeOwner("devin", "ponytail");
      return true;
    case "principles":
      removeOwner("devin", "principles");
      return true;
    default:
      return false;
  }
}

/** Verify a tool is wired into Devin. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "rtk":
      return hasDevinRtkHook();
    case "codegraph":
      return devinMcpHas("codegraph");
    case "context-mode":
      return devinMcpHas("context-mode");
    case "caveman":
      return hasOwner("devin", "caveman");
    case "ponytail":
      return hasOwner("devin", "ponytail");
    case "principles":
      return hasOwner("devin", "principles");
    default:
      return null;
  }
}

// ─── Hooks ───────────────────────────────────────────────────

type HookCfg = Record<string, unknown>;

function loadHooks(): { cfg: Record<string, unknown>; raw: string } {
  const p = paths.devinPaths();
  const raw = paths.readFile(p.hooksFile) ?? "";
  try {
    if (raw) {
      const cfg = JSON.parse(raw) as Record<string, unknown>;
      return { cfg, raw };
    }
  } catch {}
  const cfgFromJson = readJsonFile(p.hooksFile) as Record<string, unknown> | null;
  if (cfgFromJson) return { cfg: cfgFromJson, raw };
  return { cfg: {}, raw: "" };
}

function saveHooks(cfg: Record<string, unknown>): void {
  const p = paths.devinPaths();
  writeJsonFile(p.hooksFile, cfg);
}

function addHookGroup(
  cfg: Record<string, unknown>,
  event: string,
  matcher: string,
  hookEntry: HookCfg,
): void {
  const arr = (cfg[event] as unknown[]) ?? [];
  const existing = arr as Record<string, unknown>[];
  const cmd = hookEntry.command as string;
  for (const g of existing) {
    if (g.matcher !== matcher) continue;
    const hooks = g.hooks as unknown[] | undefined;
    if (!Array.isArray(hooks)) continue;
    for (const h of hooks) {
      const hc = (h as Record<string, unknown>).command as string | undefined;
      if (hc === cmd) return;
    }
  }
  existing.push({ matcher, hooks: [hookEntry] });
  cfg[event] = existing as never;
}

function removeHookGroup(cfg: Record<string, unknown>, event: string, substr: string): void {
  const arr = cfg[event] as unknown[] | undefined;
  if (!Array.isArray(arr)) return;
  const kept = arr.filter((g) => {
    const hooks = (g as Record<string, unknown>).hooks as unknown[] | undefined;
    if (!Array.isArray(hooks)) return true;
    for (const h of hooks) {
      const c = (h as Record<string, unknown>).command as string | undefined;
      if (c?.includes(substr)) return false;
    }
    return true;
  });
  if (kept.length === 0) delete cfg[event];
  else cfg[event] = kept as never;
}

function hasHook(cfg: Record<string, unknown>, event: string, substr: string): boolean {
  const arr = cfg[event] as unknown[] | undefined;
  if (!Array.isArray(arr)) return false;
  for (const g of arr) {
    const hooks = (g as Record<string, unknown>).hooks as unknown[] | undefined;
    if (!Array.isArray(hooks)) continue;
    for (const h of hooks) {
      const c = (h as Record<string, unknown>).command as string | undefined;
      if (c?.includes(substr)) return true;
    }
  }
  return false;
}

function installDevinRtkHook(): void {
  const p = paths.devinPaths();
  paths.ensureDir(p.dir);
  const { cfg } = loadHooks();
  const tok = paths.toksaveAbs();
  const command = `${tok} rtk-hook devin`;
  addHookGroup(cfg, "PreToolUse", "Execute", { type: "command", command, timeout: 10 });
  saveHooks(cfg);
}

function removeDevinRtkHook(): void {
  const { cfg } = loadHooks();
  removeHookGroup(cfg, "PreToolUse", "rtk-hook devin");
  saveHooks(cfg);
}

function hasDevinRtkHook(): boolean {
  const { cfg } = loadHooks();
  return hasHook(cfg, "PreToolUse", "rtk-hook devin");
}

// ─── MCP ─────────────────────────────────────────────────────

function configureDevinMcp(toolId: string): void {
  const p = paths.devinPaths();
  paths.ensureDir(p.dir);
  const abs = paths.toksaveAbs();
  const cfg = (readJsonFile(p.mcpConfig) as Record<string, unknown>) ?? {};
  const servers = getOrCreateObject(cfg, "mcpServers");

  const args =
    toolId === "codegraph"
      ? ["runmcp", "--agent", "devin", "codegraph", "serve", "--mcp"]
      : ["runmcp", "--agent", "devin", toolId];
  const entry: Record<string, unknown> = {
    command: abs,
    args,
  };
  if (toolId === "codegraph") {
    entry.enabledTools = CODEGRAPH_TOOLS;
  } else if (toolId === "context-mode") {
    entry.enabledTools = CONTEXT_MODE_TOOLS;
  }

  (servers as Record<string, unknown>)[toolId] = entry;
  writeJsonFile(p.mcpConfig, cfg);
}

function removeDevinMcp(toolId: string): void {
  const p = paths.devinPaths();
  const cfg = readJsonFile(p.mcpConfig) as Record<string, unknown> | null;
  if (!cfg) return;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  if (!mcp?.[toolId]) return;
  delete mcp[toolId];
  writeJsonFile(p.mcpConfig, cfg);
}

function devinMcpHas(toolId: string): boolean {
  const p = paths.devinPaths();
  const cfg = readJsonFile(p.mcpConfig) as Record<string, unknown> | null;
  if (!cfg) return false;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  return !!mcp?.[toolId];
}
