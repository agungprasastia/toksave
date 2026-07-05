import { existsSync, rmSync } from "node:fs";
import { getOrCreateObject, hasKey, readJsonFile, writeJsonFile } from "../config/json.js";
import {
  CTX_RULES_BLOCK,
  hasCtxRules,
  removeCtxRules as stripCtxRules,
} from "../content/ctx-rules.js";
import { hasRtkRules, RTK_RULES_BLOCK, removeRtkRules } from "../content/rtk-rules.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { getCavemanInstructionBlock } from "../tools/caveman.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn, isOnPath } from "../util/detect.js";
import * as paths from "../util/paths.js";

/** Detect if OpenCode is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("opencode", paths.opencodeKnownBinDirs());
  const hasDesktop = paths.opencodeDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (existsSync(paths.opencodePaths().dir)) return { installed: true, source: "config" };
  return { installed: false, source: "" };
}

/** Wire a tool into OpenCode. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      return wireMcp(
        "codegraph",
        [paths.toksaveAbs(), "runmcp", "codegraph", "serve", "--mcp"],
        opts,
      );
    case "context-mode":
      wireMcp("context-mode", [paths.toksaveAbs(), "runmcp", "context-mode"], opts);
      if (!opts.dryRun) wireCtxRules(opts);
      return true;
    case "caveman":
      return wireCaveman(opts);
    case "rtk":
      if (!opts.dryRun) {
        wireRtkPlugin(opts);
        wireRtkRules(opts);
      }
      return true;
  }
}

/** Unwire a tool from OpenCode. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeCtxRulesFile();
      return true;
    case "caveman":
      removeCavemanRules();
      return true;
    case "rtk":
      removeRtkPlugin();
      removeRtkRulesFile();
      return true;
  }
}

/** Verify a tool is wired into OpenCode. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasMcp("context-mode");
    case "caveman":
      return hasCavemanRules();
    case "rtk": {
      const rules = paths.readFile(paths.opencodePaths().agentsMd);
      return isOnPath("rtk") && hasRtkPlugin() && !!rules && hasRtkRules(rules);
    }
  }
}

// ─── MCP wiring ──────────────────────────────────────────────

function wireMcp(toolId: string, command: string[], opts: RunOpts): boolean {
  if (opts.dryRun) return true;

  verbose(`Wiring MCP ${toolId} into OpenCode`, opts.verbose);

  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};

  // Ensure $schema
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

// ─── Caveman via AGENTS.md ──────────────────────────────────

async function wireCaveman(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) return true;
  const p = paths.opencodePaths();
  paths.ensureDir(p.dir);

  verbose("Writing Caveman rules into OpenCode AGENTS.md", opts.verbose);

  const existing = paths.readFile(p.agentsMd) ?? "";
  if (existing.includes("CAVEMAN_START") && !opts.upgrade) return true;

  const cavemanBlock = await getCavemanInstructionBlock();
  const stripped = existing.replace(
    /\r?\n?<!--\s*CAVEMAN_START[\s\S]*?CAVEMAN_END\s*-->\r?\n?/g,
    "\n",
  );
  paths.writeFile(p.agentsMd, `${stripped}\n${cavemanBlock}`);
  return true;
}

function removeCavemanRules(): void {
  const p = paths.opencodePaths();
  const existing = paths.readFile(p.agentsMd);
  if (!existing) return;
  const stripped = existing
    .replace(/\r?\n?<!--\s*CAVEMAN_START[\s\S]*?CAVEMAN_END\s*-->\r?\n?/g, "")
    .trim();
  paths.writeFile(p.agentsMd, stripped);
}

function hasCavemanRules(): boolean {
  const p = paths.opencodePaths();
  const existing = paths.readFile(p.agentsMd);
  return !!existing?.includes("CAVEMAN_START");
}

// ─── Context-Mode rules ─────────────────────────────────────

function wireCtxRules(opts: RunOpts): void {
  const p = paths.opencodePaths();
  paths.ensureDir(p.dir);

  verbose("Injecting Context-Mode rules into OpenCode AGENTS.md", opts.verbose);

  const existing = paths.readFile(p.agentsMd) ?? "";
  if (hasCtxRules(existing)) return;

  paths.writeFile(p.agentsMd, `${existing}\n${CTX_RULES_BLOCK}`);
}

function removeCtxRulesFile(): void {
  const p = paths.opencodePaths();
  const existing = paths.readFile(p.agentsMd);
  if (!existing) return;
  paths.writeFile(p.agentsMd, stripCtxRules(existing));
}

function wireRtkPlugin(opts: RunOpts): void {
  const pluginFile = rtkPluginPath();
  verbose("Installing RTK plugin for OpenCode", opts.verbose);
  paths.writeFile(pluginFile, RTK_PLUGIN);
}

function removeRtkPlugin(): void {
  try {
    rmSync(rtkPluginPath(), { force: true });
  } catch {
    /* ignore */
  }
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
    output.args.command = ` + "`rtk ${command}`" + `;
  },
});
`;

function wireRtkRules(opts: RunOpts): void {
  const p = paths.opencodePaths();
  verbose("Injecting RTK rules into OpenCode AGENTS.md", opts.verbose);

  const existing = paths.readFile(p.agentsMd) ?? "";
  if (hasRtkRules(existing) && !opts.upgrade) return;

  paths.writeFile(p.agentsMd, `${removeRtkRules(existing)}\n${RTK_RULES_BLOCK}`);
}

function removeRtkRulesFile(): void {
  const p = paths.opencodePaths();
  const existing = paths.readFile(p.agentsMd);
  if (!existing) return;
  paths.writeFile(p.agentsMd, removeRtkRules(existing));
}
