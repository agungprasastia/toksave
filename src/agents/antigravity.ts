import type { Detection, RunOpts, ToolId } from "../registry.js";
import { readJsonFile, writeJsonFile, getOrCreateObject, addToArrayIfMissing } from "../config/json.js";
import { findBinaryIn, isOnPath } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { verbose } from "../util/colors.js";
import { CAVEMAN_SKILL_MD } from "../content/caveman-skill.js";
import { CTX_RULES_BLOCK, hasCtxRules, removeCtxRules as stripCtxRules } from "../content/ctx-rules.js";
import { existsSync, rmSync } from "fs";
import { join } from "path";

/** Detect if Antigravity is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("agy", paths.antigravityKnownBinDirs());
  const hasDesktop = paths.antigravityDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (existsSync(paths.antigravityPaths().dir)) return { installed: true, source: "config" };
  return { installed: false, source: "" };
}

/** Wire a tool into Antigravity. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      wireMcp("codegraph", "codegraph", ["serve", "--mcp"], opts);
      if (!opts.dryRun) allowEntry("mcp(codegraph/*)");
      return true;
    case "context-mode":
      wireMcp("context-mode", "context-mode", [], opts);
      if (!opts.dryRun) {
        allowEntry("mcp(context-mode/*)");
        installContextModeHook(opts);
        wireCtxRules(opts);
      }
      return true;
    case "rtk":
      if (!opts.dryRun) {
        allowEntry("command(rtk)");
        installRtkHook(opts);
      }
      return true;
    case "caveman":
      return wireCaveman(opts);
  }
}

/** Unwire a tool from Antigravity. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph": removeMcp("codegraph"); return true;
    case "context-mode": removeMcp("context-mode"); removeContextModeHook(); removeCtxRulesFile(); return true;
    case "rtk": removeRtkHook(); return true;
    case "caveman": removeCaveman(); return true;
  }
}

/** Verify a tool is wired into Antigravity. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph": return hasMcp("codegraph");
    case "context-mode": return hasMcp("context-mode");
    case "rtk": return hasRtkHook();
    case "caveman": return hasCavemanSkill();
  }
}

// ─── MCP wiring (multi-surface) ─────────────────────────────

function wireMcp(toolId: string, command: string, args: string[], opts: RunOpts): void {
  if (opts.dryRun) return;

  verbose(`Wiring MCP ${toolId} into Antigravity (multi-surface)`, opts.verbose);

  for (const f of paths.antigravityMcpFiles()) {
    paths.ensureDir(require("path").dirname(f));
    let cfg = readJsonFile(f) ?? {};
    const servers = getOrCreateObject(cfg, "mcpServers");

    const entry: any = { command };
    if (args.length > 0) entry.args = args;
    entry.trust = true;

    servers[toolId] = entry;
    writeJsonFile(f, cfg);
  }
}

function removeMcp(toolId: string): void {
  for (const f of paths.antigravityMcpFiles()) {
    const cfg = readJsonFile(f);
    if (cfg?.mcpServers?.[toolId]) {
      delete cfg.mcpServers[toolId];
      writeJsonFile(f, cfg);
    }
  }
}

function hasMcp(toolId: string): boolean {
  for (const f of paths.antigravityMcpFiles()) {
    const cfg = readJsonFile(f);
    if (!cfg?.mcpServers?.[toolId]) return false;
  }
  return true;
}

// ─── Permissions ─────────────────────────────────────────────

function allowEntry(entry: string): void {
  for (const f of paths.antigravitySettingsFiles()) {
    paths.ensureDir(require("path").dirname(f));
    let cfg = readJsonFile(f) ?? {};
    const perms = getOrCreateObject(cfg, "permissions");
    if (!Array.isArray(perms.allow)) perms.allow = [];
    addToArrayIfMissing(perms.allow, entry);
    writeJsonFile(f, cfg);
  }
}

// ─── RTK hook ────────────────────────────────────────────────

function installRtkHook(opts: RunOpts): void {
  const hooksFile = paths.antigravityPaths().hooks;
  paths.ensureDir(require("path").dirname(hooksFile));

  verbose("Installing RTK hook for Antigravity", opts.verbose);

  let cfg = readJsonFile(hooksFile) ?? {};
  const tok = paths.toksaveAbs();
  const command = `${tok} rtk-hook agy`;

  cfg.rtk = {
    PreToolUse: [{
      matcher: "",
      hooks: [{ type: "command", command, timeout: 10 }],
    }],
  };

  writeJsonFile(hooksFile, cfg);
}

function removeRtkHook(): void {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile);
  if (cfg?.rtk) {
    delete cfg.rtk;
    writeJsonFile(hooksFile, cfg);
  }
}

function hasRtkHook(): boolean {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile);
  return !!cfg?.rtk;
}

// ─── Context-Mode hook ──────────────────────────────────────

function installContextModeHook(opts: RunOpts): void {
  const hooksFile = paths.antigravityPaths().hooks;
  paths.ensureDir(require("path").dirname(hooksFile));

  verbose("Installing Context-Mode hook for Antigravity", opts.verbose);

  let cfg = readJsonFile(hooksFile) ?? {};

  cfg.ctx = {
    PreInvocation: [{
      matcher: "",
      hooks: [{ type: "command", command: `${paths.toksaveAbs()} context-mode-hook agy preinvocation`, timeout: 10 }],
    }],
    PreToolUse: [{
      matcher: "read_url_content|run_command|view_file",
      hooks: [{ type: "command", command: "context-mode hook gemini-cli beforetool", timeout: 10 }],
    }],
  };

  writeJsonFile(hooksFile, cfg);
}

function removeContextModeHook(): void {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile);
  if (cfg?.ctx) {
    delete cfg.ctx;
    writeJsonFile(hooksFile, cfg);
  }
}

// ─── Caveman ─────────────────────────────────────────────────

function wireCaveman(opts: RunOpts): boolean {
  if (opts.dryRun) return true;
  const gemini = paths.antigravityPaths().dir;
  const skillDir = join(gemini, "config", "skills", "caveman");
  paths.ensureDir(skillDir);
  const skillFile = join(skillDir, "SKILL.md");
  if (!existsSync(skillFile)) {
    verbose("Writing Caveman SKILL.md for Antigravity", opts.verbose);
    paths.writeFile(skillFile, CAVEMAN_SKILL_MD);
  }
  return true;
}

function removeCaveman(): void {
  const gemini = paths.antigravityPaths().dir;
  const skillDir = join(gemini, "config", "skills", "caveman");
  try { rmSync(skillDir, { recursive: true, force: true }); } catch { /* ignore */ }
}

function hasCavemanSkill(): boolean {
  const gemini = paths.antigravityPaths().dir;
  return existsSync(join(gemini, "config", "skills", "caveman", "SKILL.md"));
}

// ─── Context-Mode rules ─────────────────────────────────────

function wireCtxRules(opts: RunOpts): void {
  const mdFile = paths.antigravityPaths().agentsMd;

  verbose("Injecting Context-Mode rules into Antigravity AGENTS.md", opts.verbose);

  const existing = paths.readFile(mdFile) ?? "";
  if (hasCtxRules(existing)) return;

  paths.writeFile(mdFile, existing + "\n" + CTX_RULES_BLOCK);
}

function removeCtxRulesFile(): void {
  const mdFile = paths.antigravityPaths().agentsMd;
  const existing = paths.readFile(mdFile);
  if (!existing) return;
  paths.writeFile(mdFile, stripCtxRules(existing));
}
