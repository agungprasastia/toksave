import { existsSync } from "node:fs";
import { getOrCreateObject, readJsonFile, writeJsonFile } from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { warn } from "../util/colors.js";
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

/** Detect if Warp / Oz Agent is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("warp", paths.warpKnownBinDirs());
  const hasDesktop = paths.warpDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (process.env.NODE_ENV === "test" && existsSync(paths.warpPaths().dir)) {
    return { installed: true, source: "config" };
  }
  const dir = paths.warpPaths().dir;
  if (existsSync(dir) || existsSync(paths.warpPaths().mcpConfig)) {
    return { installed: true, source: "config" };
  }
  return { installed: false, source: "" };
}

/** Wire a tool into Warp / Oz Agent. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "rtk":
      if (!opts.dryRun) installWarpRtkHook();
      return true;
    case "codegraph":
      if (!opts.dryRun) {
        configureWarpMcp("codegraph");
        writeOwner("warp", "codegraph");
      }
      return true;
    case "context-mode":
      if (!opts.dryRun) {
        configureWarpMcp("context-mode");
        writeOwner("warp", "context-mode");
      }
      return true;
    case "caveman":
      if (!opts.dryRun) writeOwner("warp", "caveman");
      return true;
    case "ponytail":
      if (!opts.dryRun) writeOwner("warp", "ponytail");
      return true;
    case "principles":
      if (!opts.dryRun) writeOwner("warp", "principles");
      return true;
    default:
      return false;
  }
}

/** Unwire a tool from Warp / Oz Agent. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "rtk":
      removeWarpRtkHook();
      return true;
    case "codegraph":
      removeWarpMcp("codegraph");
      removeOwner("warp", "codegraph");
      return true;
    case "context-mode":
      removeWarpMcp("context-mode");
      removeOwner("warp", "context-mode");
      return true;
    case "caveman":
      removeOwner("warp", "caveman");
      return true;
    case "ponytail":
      removeOwner("warp", "ponytail");
      return true;
    case "principles":
      removeOwner("warp", "principles");
      return true;
    default:
      return false;
  }
}

/** Verify a tool is wired into Warp / Oz Agent. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "rtk":
      return hasWarpRtkHook();
    case "codegraph":
      return warpMcpHas("codegraph");
    case "context-mode":
      return warpMcpHas("context-mode");
    case "caveman":
      return hasOwner("warp", "caveman");
    case "ponytail":
      return hasOwner("warp", "ponytail");
    case "principles":
      return hasOwner("warp", "principles");
    default:
      return null;
  }
}

// ─── Hooks ───────────────────────────────────────────────────

type HookCfg = Record<string, unknown>;

function loadHooks(): { cfg: Record<string, unknown>; raw: string } | null {
  const p = paths.warpPaths();
  const raw = paths.readFile(p.hooksFile) ?? "";
  if (!raw) return { cfg: {}, raw: "" };
  try {
    const cfg = JSON.parse(raw) as Record<string, unknown>;
    if (cfg && typeof cfg === "object" && !Array.isArray(cfg)) return { cfg, raw };
  } catch {
    // Intentional: fall through, hooks.json exists but is unparseable -> return null so
    // callers never overwrite the file with a partially-rebuilt structure.
  }
  return null;
}

function saveHooks(cfg: Record<string, unknown>): void {
  const p = paths.warpPaths();
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
  // Remove only the inner hook entries that match `substr`. Empty groups are dropped;
  // groups that still have user hooks are kept. The event key is deleted only when
  // no groups remain.
  const kept = arr.reduce<Record<string, unknown>[]>((acc, g) => {
    const hooks = (g as Record<string, unknown>).hooks as unknown[] | undefined;
    if (!Array.isArray(hooks)) {
      acc.push(g as Record<string, unknown>);
      return acc;
    }
    const keptHooks = hooks.filter((h) => {
      const c = (h as Record<string, unknown>).command as string | undefined;
      return !c?.includes(substr);
    });
    if (keptHooks.length > 0) {
      acc.push({ ...(g as Record<string, unknown>), hooks: keptHooks });
    }
    return acc;
  }, []);
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

function installWarpRtkHook(): void {
  const p = paths.warpPaths();
  paths.ensureDir(p.dir);
  const res = loadHooks();
  if (!res) {
    warn(
      `warp: skipping hook install; could not parse ${p.hooksFile}. Restore it or delete it and re-run 'toksave wire warp'.`,
    );
    return;
  }
  const tok = paths.toksaveAbs();
  const command = `${tok} rtk-hook warp`;
  addHookGroup(res.cfg, "PreToolUse", "Execute", { type: "command", command, timeout: 10 });
  saveHooks(res.cfg);
}

function removeWarpRtkHook(): void {
  const res = loadHooks();
  if (!res) return; // hooks.json unreadable; leave the file untouched.
  removeHookGroup(res.cfg, "PreToolUse", "rtk-hook");
  saveHooks(res.cfg);
}

function hasWarpRtkHook(): boolean {
  const res = loadHooks();
  if (!res) return false;
  return hasHook(res.cfg, "PreToolUse", "rtk-hook");
}

// ─── MCP ─────────────────────────────────────────────────────

function configureWarpMcp(toolId: string): void {
  const p = paths.warpPaths();
  paths.ensureDir(p.dir);
  const abs = paths.toksaveAbs();
  const cfg = (readJsonFile(p.mcpConfig) as Record<string, unknown>) ?? {};
  const servers = getOrCreateObject(cfg, "mcpServers");

  const args =
    toolId === "codegraph"
      ? ["runmcp", "--agent", "warp", "codegraph", "serve", "--mcp"]
      : ["runmcp", "--agent", "warp", toolId];
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

function removeWarpMcp(toolId: string): void {
  const p = paths.warpPaths();
  const cfg = readJsonFile(p.mcpConfig) as Record<string, unknown> | null;
  if (!cfg) return;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  if (!mcp?.[toolId]) return;
  delete mcp[toolId];
  writeJsonFile(p.mcpConfig, cfg);
}

function warpMcpHas(toolId: string): boolean {
  const p = paths.warpPaths();
  const cfg = readJsonFile(p.mcpConfig) as Record<string, unknown> | null;
  if (!cfg) return false;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  return !!mcp?.[toolId];
}
