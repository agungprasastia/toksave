import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import {
  addToArrayIfMissing,
  getOrCreateObject,
  readJsonFile,
  writeJsonFile,
} from "../config/json.js";
import { CAVEMAN_SKILL_MD } from "../content/caveman-skill.js";
import { CTX_RULES_BLOCK, hasCtxRules } from "../content/ctx-rules.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn, isOnPath } from "../util/detect.js";
import * as paths from "../util/paths.js";

/** Detect if Claude Code is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("claude", paths.claudeKnownBinDirs());
  const hasDesktop = paths.claudeDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (existsSync(paths.claudePaths().dir)) return { installed: true, source: "config" };
  return { installed: false, source: "" };
}

/** Wire a tool into Claude Code. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      return wireMcp(
        "codegraph",
        paths.toksaveAbs(),
        ["runmcp", "codegraph", "serve", "--mcp"],
        opts,
      );
    case "context-mode":
      wireMcp("context-mode", paths.toksaveAbs(), ["runmcp", "context-mode"], opts);
      if (!opts.dryRun) wireCtxRules(opts);
      return true;
    case "caveman":
      return wireCaveman(opts);
    case "rtk":
      if (!opts.dryRun) allowBashPattern("Bash(rtk *)");
      return true;
  }
}

/** Unwire a tool from Claude Code. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeCtxRules();
      return true;
    case "caveman":
      removeCaveman();
      return true;
    case "rtk":
      return true;
  }
}

/** Verify a tool is wired into Claude Code. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasMcp("context-mode");
    case "caveman":
      return hasCavemanSkill();
    case "rtk":
      return isOnPath("rtk");
  }
}

// ─── MCP wiring ──────────────────────────────────────────────

function wireMcp(toolId: string, command: string, args: string[], opts: RunOpts): boolean {
  const p = paths.claudePaths();
  if (opts.dryRun) return true;

  verbose(`Wiring MCP ${toolId} into Claude Code`, opts.verbose);

  // Auto-approve MCP tools
  allowMcpTool(toolId);

  const cfg = (readJsonFile(p.globalJson) as Record<string, unknown>) ?? {};
  const servers = getOrCreateObject(cfg, "mcpServers");

  const entry: Record<string, unknown> = { type: "stdio", command, args };

  // Check if already identical
  const srv = servers as Record<string, unknown>;
  if (srv[toolId] && JSON.stringify(srv[toolId]) === JSON.stringify(entry)) {
    return true;
  }
  srv[toolId] = entry;
  writeJsonFile(p.globalJson, cfg);
  return true;
}

function removeMcp(toolId: string): void {
  const p = paths.claudePaths();
  const cfg = readJsonFile(p.globalJson) as Record<string, unknown>;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  if (mcp?.[toolId]) {
    delete mcp[toolId];
    writeJsonFile(p.globalJson, cfg);
  }
}

function hasMcp(toolId: string): boolean {
  const p = paths.claudePaths();
  const cfg = readJsonFile(p.globalJson) as Record<string, unknown>;
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  return !!mcp?.[toolId];
}

function allowMcpTool(toolId: string): void {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.settings) as Record<string, unknown>) ?? {};
  const perms = getOrCreateObject(cfg, "permissions");
  if (!Array.isArray(perms.allow)) perms.allow = [];
  addToArrayIfMissing(perms.allow as unknown[], `mcp__${toolId}__.*`);
  writeJsonFile(p.settings, cfg);
}

function allowBashPattern(pattern: string): void {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.settings) as Record<string, unknown>) ?? {};
  const perms = getOrCreateObject(cfg, "permissions");
  if (!Array.isArray(perms.allow)) perms.allow = [];
  addToArrayIfMissing(perms.allow as unknown[], pattern);
  writeJsonFile(p.settings, cfg);
}

// ─── Caveman wiring ─────────────────────────────────────────

function wireCaveman(opts: RunOpts): boolean {
  if (opts.dryRun) return true;
  const p = paths.claudePaths();
  const skillDir = join(p.skillsDir, "caveman");
  paths.ensureDir(skillDir);
  const skillFile = join(skillDir, "SKILL.md");
  if (!existsSync(skillFile) || opts.upgrade) {
    verbose("Writing Caveman SKILL.md for Claude Code", opts.verbose);
    paths.writeFile(skillFile, CAVEMAN_SKILL_MD);
  }
  return true;
}

function removeCaveman(): void {
  const p = paths.claudePaths();
  const skillDir = join(p.skillsDir, "caveman");
  try {
    rmSync(skillDir, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
}

function hasCavemanSkill(): boolean {
  const p = paths.claudePaths();
  return existsSync(join(p.skillsDir, "caveman", "SKILL.md"));
}

// ─── Context-Mode rules wiring ──────────────────────────────

function wireCtxRules(opts: RunOpts): void {
  const p = paths.claudePaths();
  const mdFile = p.agentsMd;

  verbose("Injecting Context-Mode rules into Claude AGENTS.md", opts.verbose);

  const existing = paths.readFile(mdFile) ?? "";
  if (hasCtxRules(existing)) return; // Already present

  paths.writeFile(mdFile, `${existing}\n${CTX_RULES_BLOCK}`);
}

function removeCtxRules(): void {
  const p = paths.claudePaths();
  const mdFile = p.agentsMd;
  const existing = paths.readFile(mdFile);
  if (!existing) return;

  const { removeCtxRules: strip } = require("../content/ctx-rules.js");
  paths.writeFile(mdFile, strip(existing));
}
