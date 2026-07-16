import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import { getOrCreateObject, hasKey, readJsonFile, writeJsonFile } from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import {
  opencodePluginInstalled,
  registerCavemanOpencode,
  unregisterCavemanOpencode,
} from "../tools/caveman.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn, isOnPath } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner, removeOwner, writeOwner } from "../util/unified-block.js";

/** Detect if OpenCode is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("opencode", paths.opencodeKnownBinDirs());
  const hasDesktop = paths.opencodeDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (process.env.NODE_ENV === "test" && existsSync(paths.opencodePaths().dir)) {
    return { installed: true, source: "config" };
  }
  return { installed: false, source: "" };
}

/** Wire a tool into OpenCode. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      wireMcp("codegraph", [paths.toksaveAbs(), "runmcp", "codegraph", "serve", "--mcp"], opts);
      if (!opts.dryRun) {
        writeOwner("opencode", "codegraph");
        unwireAutoIndexOpenCode();
      }
      return true;
    case "context-mode":
      wireMcpContextMode(opts);
      if (!opts.dryRun) writeOwner("opencode", "context-mode");
      return true;
    case "caveman":
      if (!opts.dryRun) {
        registerCavemanOpencode();
        writeOwner("opencode", "caveman");
      }
      return true;
    case "rtk":
      if (!opts.dryRun) {
        wireRtkPlugin(opts);
      }
      return true;
    case "ponytail":
      if (!opts.dryRun) {
        registerPonytailPlugin();
        writeOwner("opencode", "ponytail");
      }
      return true;
    case "principles":
      if (!opts.dryRun) writeOwner("opencode", "principles");
      return true;
    default:
      return false;
  }
}

/** Unwire a tool from OpenCode. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      removeOwner("opencode", "codegraph");
      return true;
    case "context-mode":
      removeContextModePlugin();
      removeOwner("opencode", "context-mode");
      return true;
    case "caveman":
      unregisterCavemanOpencode();
      removeOwner("opencode", "caveman");
      return true;
    case "rtk":
      removeRtkPlugin();
      return true;
    case "ponytail":
      unregisterPonytailPlugin();
      removeOwner("opencode", "ponytail");
      removePonytailArtifacts();
      return true;
    case "principles":
      removeOwner("opencode", "principles");
      return true;
    default:
      return false;
  }
}

/** Verify a tool is wired into OpenCode. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasContextModePlugin();
    case "caveman":
      return opencodePluginInstalled() || hasOwner("opencode", "caveman");
    case "rtk":
      return isOnPath("rtk") && hasRtkPlugin();
    case "ponytail":
      return hasOwner("opencode", "ponytail") || ponytailPluginInstalled();
    case "principles":
      return hasOwner("opencode", "principles");
    default:
      return null;
  }
}

// ─── MCP wiring ──────────────────────────────────────────────

function wireMcp(toolId: string, command: string[], opts: RunOpts): boolean {
  if (opts.dryRun) return true;

  verbose(`Wiring MCP ${toolId} into OpenCode`, opts.verbose);

  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};

  if (!hasKey(cfg, "$schema")) {
    cfg.$schema = "https://opencode.ai/config.json";
  }

  const mcp = getOrCreateObject(cfg, "mcp");
  const entry = { type: "local", command, enabled: true };
  const existing = mcp[toolId] as
    | { type?: unknown; command?: unknown; enabled?: unknown }
    | undefined;

  if (
    existing?.type === entry.type &&
    Array.isArray(existing.command) &&
    existing.command.length === command.length &&
    existing.command.every((arg, i) => arg === command[i]) &&
    existing.enabled === entry.enabled
  ) {
    return true;
  }

  mcp[toolId] = entry;
  writeJsonFile(p.config, cfg);
  return true;
}

function wireMcpContextMode(opts: RunOpts): boolean {
  if (opts.dryRun) return true;
  const p = paths.opencodePaths();
  paths.ensureDir(p.dir);
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  if (!hasKey(cfg, "$schema")) cfg.$schema = "https://opencode.ai/config.json";
  // Ensure bare context-mode plugin entry, remove versioned
  setContextModePluginBare(cfg);
  // Also ensure mcp entry for context-mode via local runmcp (if using plugin bare, mcp comes from plugin)
  // For safety, ensure mcp entry not duplicated as old style
  const mcp = getOrCreateObject(cfg, "mcp");
  if (mcp["context-mode"]) {
    delete mcp["context-mode"];
  }
  writeJsonFile(p.config, cfg);
  return true;
}

function setContextModePluginBare(cfg: Record<string, unknown>): void {
  const plugins = (cfg.plugin as unknown[]) ?? [];
  const kept: unknown[] = [];
  for (const p of plugins) {
    if (typeof p === "string" && (p === "context-mode" || p.startsWith("context-mode@"))) continue;
    kept.push(p);
  }
  kept.push("context-mode");
  cfg.plugin = kept;
  // Remove old mcp.context-mode if exists
  const mcp = (cfg.mcp as Record<string, unknown>) ?? {};
  if (mcp["context-mode"]) {
    delete mcp["context-mode"];
    if (Object.keys(mcp).length === 0) delete (cfg as Record<string, unknown>).mcp;
  }
}

function removeMcp(toolId: string): void {
  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  const mcp = cfg.mcp as Record<string, unknown> | undefined;
  if (mcp?.[toolId]) {
    delete mcp[toolId];
    writeJsonFile(p.config, cfg);
  }
}

function hasMcp(toolId: string): boolean {
  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  const mcp = cfg.mcp as Record<string, unknown> | undefined;
  return !!mcp?.[toolId];
}

function hasContextModePlugin(): boolean {
  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return false;
  return plugins.some(
    (pl) => typeof pl === "string" && (pl === "context-mode" || pl.startsWith("context-mode@")),
  );
}

function removeContextModePlugin(): void {
  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return;
  const kept = plugins.filter(
    (pl) => !(typeof pl === "string" && (pl === "context-mode" || pl.startsWith("context-mode@"))),
  );
  cfg.plugin = kept;
  writeJsonFile(p.config, cfg);
}

// Legacy auto-index cleanup (opencode/plugins/tokless-codegraph-init.js)
function unwireAutoIndexOpenCode(): void {
  const legacyPath = `${paths.opencodePaths().dir}/plugins/tokless-codegraph-init.js`;
  try {
    rmSync(legacyPath, { force: true });
  } catch {}
}

// ─── RTK plugin ──────────────────────────────────────────────

function wireRtkPlugin(opts: RunOpts): void {
  const pluginFile = rtkPluginPath();
  verbose("Installing RTK plugin for OpenCode", opts.verbose);
  paths.writeFile(pluginFile, RTK_PLUGIN);
}

function removeRtkPlugin(): void {
  try {
    rmSync(rtkPluginPath(), { force: true });
  } catch {}
}

function hasRtkPlugin(): boolean {
  return existsSync(rtkPluginPath());
}

function rtkPluginPath(): string {
  return `${paths.opencodePaths().dir}/plugins/toksave-rtk.js`;
}

const RTK_PLUGIN = `export const Plugin = async () => ({
  "tool.execute.before": async (input, output) => {
    if (input.tool !== "bash") return;
    const command = String(output.args.command ?? "").trim();
    if (!command || command === "rtk" || command.startsWith("rtk ")) return;
    output.args.command = \`rtk \${command}\`;
  },
});
`;

// ─── Ponytail plugin ─────────────────────────────────────────

const PONYTAIL_PKG = "@dietrichgebert/ponytail";

function registerPonytailPlugin(): void {
  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  if (!hasKey(cfg, "$schema")) cfg.$schema = "https://opencode.ai/config.json";
  const plugins = (cfg.plugin as unknown[]) ?? [];
  // Check already present
  if (
    plugins.some((pl) => typeof pl === "string" && pl.toLowerCase() === PONYTAIL_PKG.toLowerCase())
  ) {
    return;
  }
  // Insert before context-mode if present
  let inserted = false;
  const out: unknown[] = [];
  for (const pl of plugins) {
    if (
      !inserted &&
      typeof pl === "string" &&
      (pl === "context-mode" || pl.startsWith("context-mode@"))
    ) {
      out.push(PONYTAIL_PKG);
      inserted = true;
    }
    out.push(pl);
  }
  if (!inserted) out.push(PONYTAIL_PKG);
  cfg.plugin = out;
  writeJsonFile(p.config, cfg);
}

function unregisterPonytailPlugin(): void {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) as Record<string, unknown> | null;
  if (!cfg) return;
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return;
  const kept = plugins.filter(
    (pl) => !(typeof pl === "string" && pl.toLowerCase() === PONYTAIL_PKG.toLowerCase()),
  );
  cfg.plugin = kept;
  writeJsonFile(p.config, cfg);
}

function ponytailPluginInstalled(): boolean {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) as Record<string, unknown> | null;
  if (!cfg) return false;
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return false;
  return plugins.some(
    (pl) => typeof pl === "string" && pl.toLowerCase() === PONYTAIL_PKG.toLowerCase(),
  );
}

function removePonytailArtifacts(): void {
  const dir = paths.opencodePaths().dir;
  try {
    rmSync(`${dir}/plugins/ponytail`, { recursive: true, force: true });
  } catch {}
  try {
    rmSync(`${dir}/.ponytail-active`, { force: true });
  } catch {}
}

// ─── Auto-index plugin ───────────────────────────────────────

const AUTO_INDEX_PLUGIN = `let indexed = false;
export const Plugin = async () => ({
  "tool.execute.before": async () => {
    if (indexed) return;
    indexed = true;
    const { execSync } = require("node:child_process");
    try { execSync("toksave index --auto", { timeout: 120000 }); } catch {}
  },
});
`;

function autoIndexPluginPath(): string {
  return join(paths.opencodePaths().dir, "plugins", "toksave-autoindex.js");
}

export function installOpencodeAutoIndexPlugin(): void {
  paths.writeFile(autoIndexPluginPath(), AUTO_INDEX_PLUGIN);
}

export function removeOpencodeAutoIndexPlugin(): void {
  try { rmSync(autoIndexPluginPath(), { force: true }); } catch {}
}

export function hasOpencodeAutoIndexPlugin(): boolean {
  return existsSync(autoIndexPluginPath());
}
