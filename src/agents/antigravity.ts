import { existsSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import {
  addToArrayIfMissing,
  getOrCreateObject,
  readJsonFile,
  writeJsonFile,
} from "../config/json.js";
import {
  CTX_RULES_BLOCK,
  hasCtxRules,
  removeCtxRules as stripCtxRules,
} from "../content/ctx-rules.js";
import { hasRtkRules, RTK_RULES_BLOCK, removeRtkRules } from "../content/rtk-rules.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { getSkillContent } from "../tools/caveman.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";

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
      wireMcp("codegraph", paths.toksaveAbs(), ["runmcp", "codegraph", "serve", "--mcp"], opts);
      if (!opts.dryRun) allowEntry("mcp(codegraph/*)");
      return true;
    case "context-mode":
      wireMcp("context-mode", paths.toksaveAbs(), ["runmcp", "context-mode"], opts);
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
        wireRtkRules(opts);
      }
      return true;
    case "caveman":
      return wireCaveman(opts);
  }
}

/** Unwire a tool from Antigravity. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeContextModeHook();
      removeCtxRulesFile();
      return true;
    case "rtk":
      removeRtkHook();
      removeRtkRulesFile();
      return true;
    case "caveman":
      removeCaveman();
      return true;
  }
}

/** Verify a tool is wired into Antigravity. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasMcp("context-mode");
    case "rtk": {
      const rules = paths.readFile(paths.antigravityPaths().agentsMd);
      return hasRtkHook() && !!rules && hasRtkRules(rules);
    }
    case "caveman":
      return hasCavemanSkill();
  }
}

// ─── MCP wiring (multi-surface) ─────────────────────────────

function wireMcp(toolId: string, command: string, args: string[], opts: RunOpts): void {
  if (opts.dryRun) return;

  verbose(`Wiring MCP ${toolId} into Antigravity (multi-surface)`, opts.verbose);

  const failures: string[] = [];
  for (const f of paths.antigravityMcpFiles()) {
    try {
      paths.ensureDir(dirname(f));
      const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
      const servers = getOrCreateObject(cfg, "mcpServers");

      const entry: Record<string, unknown> = { command };
      if (args.length > 0) entry.args = args;
      entry.trust = true;

      (servers as Record<string, unknown>)[toolId] = entry;
      writeJsonFile(f, cfg);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      failures.push(`${f}: ${msg}`);
    }
  }

  if (failures.length > 0) {
    throw new Error(`Failed to wire ${toolId} into Antigravity: ${failures.join("; ")}`);
  }
}

function removeMcp(toolId: string): void {
  for (const f of paths.antigravityMcpFiles()) {
    const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
    const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
    if (mcp?.[toolId]) {
      delete mcp[toolId];
      writeJsonFile(f, cfg);
    }
  }
}

function hasMcp(toolId: string): boolean {
  for (const f of paths.antigravityMcpFiles()) {
    const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
    const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
    if (!mcp?.[toolId]) return false;
  }
  return true;
}

// ─── Permissions ─────────────────────────────────────────────

function allowEntry(entry: string): void {
  const failures: string[] = [];
  for (const f of paths.antigravitySettingsFiles()) {
    try {
      paths.ensureDir(dirname(f));
      const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
      const perms = getOrCreateObject(cfg, "permissions");
      if (!Array.isArray(perms.allow)) perms.allow = [];
      addToArrayIfMissing(perms.allow as unknown[], entry);
      writeJsonFile(f, cfg);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      failures.push(`${f}: ${msg}`);
    }
  }

  if (failures.length > 0) {
    throw new Error(`Failed to update Antigravity permissions: ${failures.join("; ")}`);
  }
}

// ─── RTK hook ────────────────────────────────────────────────

function installRtkHook(opts: RunOpts): void {
  const hooksFile = paths.antigravityPaths().hooks;
  paths.ensureDir(dirname(hooksFile));

  verbose("Installing RTK hook for Antigravity", opts.verbose);

  const cfg = (readJsonFile(hooksFile) as Record<string, unknown>) ?? {};
  const tok = paths.toksaveAbs();
  const command = `${tok} rtk-hook agy`;

  (cfg as Record<string, unknown>).rtk = {
    PreToolUse: [
      {
        matcher: "^(Bash|run_command|execute_command|cmd|sh|pwsh)$",
        hooks: [{ type: "command", command, timeout: 10 }],
      },
    ],
  };

  writeJsonFile(hooksFile, cfg);
}

function removeRtkHook(): void {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile) as Record<string, unknown>;
  if ((cfg as Record<string, unknown>)?.rtk) {
    (cfg as Record<string, unknown>).rtk = undefined;
    writeJsonFile(hooksFile, cfg);
  }
}

function hasRtkHook(): boolean {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile) as Record<string, unknown>;
  return !!(cfg as Record<string, unknown>)?.rtk;
}

// ─── Context-Mode hook ──────────────────────────────────────

function installContextModeHook(opts: RunOpts): void {
  const hooksFile = paths.antigravityPaths().hooks;
  paths.ensureDir(dirname(hooksFile));

  verbose("Installing Context-Mode hook for Antigravity", opts.verbose);

  const cfg = (readJsonFile(hooksFile) as Record<string, unknown>) ?? {};

  (cfg as Record<string, unknown>).ctx = {
    PreInvocation: [
      {
        matcher: "",
        hooks: [
          {
            type: "command",
            command: `${paths.toksaveAbs()} context-mode-hook agy preinvocation`,
            timeout: 10,
          },
        ],
      },
    ],
    PreToolUse: [
      {
        matcher: "read_url_content|run_command|view_file",
        hooks: [
          {
            type: "command",
            command: `${paths.toksaveAbs()} context-mode-hook gemini-cli beforetool`,
            timeout: 10,
          },
        ],
      },
    ],
  };

  writeJsonFile(hooksFile, cfg);
}

function removeContextModeHook(): void {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile) as Record<string, unknown>;
  if ((cfg as Record<string, unknown>)?.ctx) {
    (cfg as Record<string, unknown>).ctx = undefined;
    writeJsonFile(hooksFile, cfg);
  }
}

// ─── Caveman ─────────────────────────────────────────────────

async function wireCaveman(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) return true;
  const gemini = paths.antigravityPaths().dir;
  const skillDir = join(gemini, "config", "skills", "caveman");
  paths.ensureDir(skillDir);
  const skillFile = join(skillDir, "SKILL.md");
  if (!existsSync(skillFile) || opts.upgrade) {
    verbose("Writing Caveman SKILL.md for Antigravity", opts.verbose);
    const skillContent = await getSkillContent();
    paths.writeFile(skillFile, skillContent);
  }
  return true;
}

function removeCaveman(): void {
  const gemini = paths.antigravityPaths().dir;
  const skillDir = join(gemini, "config", "skills", "caveman");
  try {
    rmSync(skillDir, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
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

  paths.writeFile(mdFile, `${existing}\n${CTX_RULES_BLOCK}`);
}

function removeCtxRulesFile(): void {
  const mdFile = paths.antigravityPaths().agentsMd;
  const existing = paths.readFile(mdFile);
  if (!existing) return;
  paths.writeFile(mdFile, stripCtxRules(existing));
}

function wireRtkRules(opts: RunOpts): void {
  const p = paths.antigravityPaths();
  verbose("Injecting RTK rules into Antigravity AGENTS.md", opts.verbose);

  const existing = paths.readFile(p.agentsMd) ?? "";
  if (hasRtkRules(existing) && !opts.upgrade) return;

  paths.writeFile(p.agentsMd, `${removeRtkRules(existing)}\n${RTK_RULES_BLOCK}`);
}

function removeRtkRulesFile(): void {
  const p = paths.antigravityPaths();
  const existing = paths.readFile(p.agentsMd);
  if (!existing) return;
  paths.writeFile(p.agentsMd, removeRtkRules(existing));
}
