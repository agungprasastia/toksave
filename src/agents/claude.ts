import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import {
  addToArrayIfMissing,
  getOrCreateObject,
  readJsonFile,
  writeJsonFile,
} from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { cavemanExec, getSkillContent } from "../tools/caveman.js";
import { verbose } from "../util/colors.js";
import { findBinary, findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner, removeOwner, writeOwner } from "../util/unified-block.js";

/** Detect if Claude Code is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("claude", paths.claudeKnownBinDirs());
  const hasDesktop = paths.claudeDesktopPaths().some((p) => existsSync(p));
  if (hasCli && hasDesktop) return { installed: true, source: "cli+desktop" };
  if (hasCli) return { installed: true, source: "cli" };
  if (hasDesktop) return { installed: true, source: "desktop" };
  if (process.env.NODE_ENV === "test" && existsSync(paths.claudePaths().dir)) {
    return { installed: true, source: "config" };
  }
  return { installed: false, source: "" };
}

/** Wire a tool into Claude Code. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      wireMcp("codegraph", paths.toksaveAbs(), ["runmcp", "codegraph", "serve", "--mcp"], opts);
      if (!opts.dryRun) {
        writeOwner("claude", "codegraph");
        // Clean legacy auto-index SessionStart hooks (tokless/toksave index --auto)
        unwireAutoIndexClaude();
        installClaudeAutoIndexHook();
      }
      return true;
    case "context-mode":
      wireMcp("context-mode", paths.toksaveAbs(), ["runmcp", "context-mode"], opts);
      if (!opts.dryRun) {
        writeOwner("claude", "context-mode");
      }
      return true;
    case "caveman":
      return wireCaveman(opts);
    case "rtk":
      if (!opts.dryRun) {
        allowBashPattern("Bash(rtk *)");
        wireRtkHook(opts);
        // Override rtk's own hook if present, strip @RTK.md ref
        overrideClaudeRtkHook();
      }
      return true;
    case "ponytail":
      if (!opts.dryRun) writeOwner("claude", "ponytail");
      return true;
    case "principles":
      if (!opts.dryRun) writeOwner("claude", "principles");
      return true;
    default:
      return false;
  }
}

/** Unwire a tool from Claude Code. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      removeOwner("claude", "codegraph");
      if (!_opts.dryRun) removeClaudeAutoIndexHook();
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeOwner("claude", "context-mode");
      return true;
    case "caveman":
      removeCaveman();
      return true;
    case "rtk":
      removeRtkHook();
      return true;
    case "ponytail":
      removeOwner("claude", "ponytail");
      return true;
    case "principles":
      removeOwner("claude", "principles");
      return true;
    default:
      return false;
  }
}

/** Verify a tool is wired into Claude Code. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph") && hasClaudeAutoIndexHook();
    case "context-mode":
      return hasMcp("context-mode");
    case "caveman":
      return hasCavemanSkill() || hasOwner("claude", "caveman");
    case "rtk":
      return hasRtkHook();
    case "ponytail":
      return hasOwner("claude", "ponytail");
    case "principles":
      return hasOwner("claude", "principles");
    default:
      return null;
  }
}

// ─── MCP wiring ──────────────────────────────────────────────

function wireMcp(toolId: string, command: string, args: string[], opts: RunOpts): boolean {
  const p = paths.claudePaths();
  if (opts.dryRun) return true;

  verbose(`Wiring MCP ${toolId} into Claude Code`, opts.verbose);

  allowMcpTool(toolId);

  const cfg = (readJsonFile(p.globalJson) as Record<string, unknown>) ?? {};
  const servers = getOrCreateObject(cfg, "mcpServers");

  const entry: Record<string, unknown> = { type: "stdio", command, args };

  const srv = servers as Record<string, unknown>;
  const existing = srv[toolId] as { type?: unknown; command?: unknown; args?: unknown } | undefined;
  if (
    existing &&
    existing.type === entry.type &&
    existing.command === command &&
    Array.isArray(existing.args) &&
    existing.args.length === args.length &&
    existing.args.every((arg, i) => arg === args[i])
  ) {
    return true;
  }
  srv[toolId] = entry;
  writeJsonFile(p.globalJson, cfg);
  return true;
}

function removeMcp(toolId: string): void {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.globalJson) as Record<string, unknown>) ?? {};
  const mcp = cfg.mcpServers as Record<string, unknown> | undefined;
  if (mcp?.[toolId]) {
    delete mcp[toolId];
    writeJsonFile(p.globalJson, cfg);
  }
}

function hasMcp(toolId: string): boolean {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.globalJson) as Record<string, unknown>) ?? {};
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

function wireRtkHook(opts: RunOpts): void {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.settings) as Record<string, unknown>) ?? {};
  const hooks = getOrCreateObject(cfg, "hooks");
  const command = `${paths.toksaveAbs()} rtk-hook claude`;

  verbose("Installing RTK hook for Claude Code", opts.verbose);

  addHook(hooks, "PreToolUse", {
    matcher: "Bash",
    hooks: [{ type: "command", command, timeout: 10 }],
  });

  writeJsonFile(p.settings, cfg);
}

function addHook(
  hooks: Record<string, unknown>,
  name: string,
  hook: Record<string, unknown>,
): void {
  if (!Array.isArray(hooks[name])) hooks[name] = [];
  const arr = hooks[name] as unknown[];
  const command = (hook.hooks as { command?: string }[] | undefined)?.[0]?.command;
  if (command && arr.some((item) => hookGroupHasCommand(item, command))) return;
  arr.push(hook);
}

function hookGroupHasCommand(group: unknown, command: string): boolean {
  const hooks = (group as { hooks?: { command?: string }[] })?.hooks;
  return Array.isArray(hooks) && hooks.some((hook) => hook.command === command);
}

function removeRtkHook(): void {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.settings) as Record<string, unknown>) ?? {};
  const hooks = (cfg.hooks as Record<string, unknown> | undefined) ?? {};
  hooks.PreToolUse = removeToksaveHookGroups(hooks.PreToolUse, "rtk-hook claude");
  writeJsonFile(p.settings, cfg);
}

function removeToksaveHookGroups(groups: unknown, marker: string): unknown[] | undefined {
  if (!Array.isArray(groups)) return undefined;
  const remaining = groups.filter((group) => !hookGroupContainsMarker(group, marker));
  return remaining.length > 0 ? remaining : undefined;
}

function hookGroupContainsMarker(group: unknown, marker: string): boolean {
  const hooks = (group as { hooks?: { command?: string }[] })?.hooks;
  return Array.isArray(hooks) && hooks.some((hook) => hook.command?.includes(marker));
}

function hasRtkHook(): boolean {
  const p = paths.claudePaths();
  const cfg = (readJsonFile(p.settings) as Record<string, unknown>) ?? {};
  const arr = (cfg.hooks as Record<string, unknown> | undefined)?.PreToolUse;
  return (
    Array.isArray(arr) && arr.some((group) => hookGroupContainsMarker(group, "rtk-hook claude"))
  );
}

// ─── RTK override — replace rtk's own "rtk hook claude" with toksave wrapper ─

function overrideClaudeRtkHook(): void {
  const p = paths.claudePaths();
  const tok = paths.toksaveAbs();
  const newCmd = `${tok} rtk-hook claude`;
  const raw = paths.readFile(p.settings);
  if (!raw) return;
  try {
    const cfg = JSON.parse(raw) as Record<string, unknown>;
    const hooks = cfg.hooks as Record<string, unknown> | undefined;
    if (!hooks) return;
    const pre = hooks.PreToolUse as unknown[] | undefined;
    if (!Array.isArray(pre)) return;
    let changed = false;
    for (const g of pre) {
      const gm = g as Record<string, unknown>;
      const inner = gm.hooks as unknown[] | undefined;
      if (!Array.isArray(inner)) continue;
      for (const h of inner) {
        const hm = h as Record<string, unknown>;
        const cmd = hm.command as string | undefined;
        if (cmd?.includes("rtk hook claude") && !cmd.includes("rtk-hook claude")) {
          hm.command = newCmd;
          changed = true;
        }
      }
    }
    // Deduplicate groups with same command
    if (pre.length > 1) {
      const seen = new Set<string>();
      const dedup: unknown[] = [];
      for (const g of pre) {
        const first = firstHookCommand(g);
        if (!seen.has(first)) {
          seen.add(first);
          dedup.push(g);
        } else {
          changed = true;
        }
      }
      if (changed) hooks.PreToolUse = dedup as never;
    }
    if (changed) {
      const ordered = readJsonFile(p.settings) as Record<string, unknown>;
      // Preserve order using existing config parser
      const hooksMap = getOrCreateObject(ordered, "hooks");
      hooksMap.PreToolUse = hooks.PreToolUse;
      writeJsonFile(p.settings, ordered);
    }
  } catch {
    /* ignore */
  }
  allowBashPattern("Bash(rtk *)");
  // Remove RTK.md + strip @RTK.md ref
  try {
    const rtkMd = join(p.dir, "RTK.md");
    rmSync(rtkMd, { force: true });
  } catch {}
  stripRtkRefFromMd(p.agentsMd);
}

function firstHookCommand(g: unknown): string {
  const gm = g as Record<string, unknown> | undefined;
  const arr = gm?.hooks as unknown[] | undefined;
  if (!Array.isArray(arr) || arr.length === 0) return "";
  const first = arr[0] as Record<string, unknown> | undefined;
  return (first?.command as string) ?? "";
}

function stripRtkRefFromMd(filePath: string): void {
  const raw = paths.readFile(filePath);
  if (!raw) return;
  const lines = raw.split("\n");
  const kept = lines.filter((l) => {
    const t = l.trim();
    return !(t.startsWith("@") && t.endsWith("RTK.md"));
  });
  const result = kept.join("\n").trim();
  if (!result) {
    try {
      rmSync(filePath, { force: true });
    } catch {}
    return;
  }
  if (result !== raw.trim()) {
    paths.writeFile(filePath, `${result}\n`);
  }
}

// Remove legacy auto-index SessionStart hooks (tokless index --auto)
function unwireAutoIndexClaude(): void {
  const p = paths.claudePaths();
  if (!existsSync(p.settings)) return;
  try {
    const cfg = readJsonFile(p.settings) as Record<string, unknown>;
    const hooks = (cfg?.hooks as Record<string, unknown>) ?? {};
    const ss = hooks.SessionStart as unknown[] | undefined;
    if (!Array.isArray(ss)) return;
    const filtered = ss.filter((g) => {
      const inner = (g as Record<string, unknown>)?.hooks as unknown[] | undefined;
      if (!Array.isArray(inner)) return true;
      return !inner.some((h) => {
        const cmd = (h as Record<string, unknown>)?.command as string | undefined;
        return cmd?.includes("index --auto");
      });
    });
    if (filtered.length !== ss.length) {
      if (filtered.length === 0) {
        delete hooks.SessionStart;
      } else {
        hooks.SessionStart = filtered as never;
      }
      if (Object.keys(hooks).length === 0) {
        delete (cfg as Record<string, unknown>).hooks;
      }
      writeJsonFile(p.settings, cfg);
    }
  } catch {}
}

// ─── Auto-index SessionStart hook ────────────────────────────

function installClaudeAutoIndexHook(): void {
  const cp = paths.claudePaths();
  const cfg = (readJsonFile(cp.settings) as Record<string, unknown>) ?? {};
  const hooks = getOrCreateObject(cfg, "hooks") as Record<string, unknown>;
  const cmd = "toksave index --auto";
  const ss = Array.isArray(hooks.SessionStart) ? hooks.SessionStart : [];
  if (
    ss.some(
      (g) =>
        typeof g === "object" &&
        g !== null &&
        (g as { hooks?: { command?: string }[] })?.hooks?.[0]?.command === cmd,
    )
  )
    return;
  const entry = {
    hooks: [{ type: "command", command: cmd, timeout: 120000 }],
  };
  ss.push(entry);
  hooks.SessionStart = ss as never;
  writeJsonFile(cp.settings, cfg);
}

function removeClaudeAutoIndexHook(): void {
  const cp = paths.claudePaths();
  const cfg = (readJsonFile(cp.settings) as Record<string, unknown>) ?? {};
  const hooks = (cfg.hooks as Record<string, unknown> | undefined) ?? {};
  const ss = hooks.SessionStart;
  if (!Array.isArray(ss)) return;
  const cmd = "toksave index --auto";
  const kept = ss.filter(
    (g) =>
      !(
        typeof g === "object" &&
        g !== null &&
        (g as { hooks?: { command?: string }[] })?.hooks?.[0]?.command === cmd
      ),
  );
  if (kept.length === 0) delete hooks.SessionStart;
  else hooks.SessionStart = kept as never;
  writeJsonFile(cp.settings, cfg);
}

function hasClaudeAutoIndexHook(): boolean {
  const cp = paths.claudePaths();
  const cfg = (readJsonFile(cp.settings) as Record<string, unknown>) ?? {};
  const hooks = cfg.hooks as Record<string, unknown> | undefined;
  const ss = hooks?.SessionStart;
  if (!Array.isArray(ss)) return false;
  const cmd = "toksave index --auto";
  return ss.some(
    (g) =>
      typeof g === "object" &&
      g !== null &&
      (g as { hooks?: { command?: string }[] })?.hooks?.[0]?.command === cmd,
  );
}

// ─── Caveman wiring ─────────────────────────────────────────

async function wireCaveman(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) return true;

  if (process.env.NODE_ENV !== "test" && findBinary("claude")) {
    const ok = cavemanExec(
      "claude",
      ["plugin", "marketplace", "add", "JuliusBrussee/caveman"],
      opts,
      "claude plugin marketplace add JuliusBrussee/caveman && claude plugin install caveman@caveman",
    );
    if (ok) {
      cavemanExec("claude", ["plugin", "install", "caveman@caveman"], opts, "");
    }
    writeOwner("claude", "caveman");
    return ok;
  }

  const p = paths.claudePaths();
  const skillDir = join(p.skillsDir, "caveman");
  paths.ensureDir(skillDir);
  const skillFile = join(skillDir, "SKILL.md");
  if (!existsSync(skillFile) || opts.upgrade) {
    verbose("Writing Caveman SKILL.md for Claude Code", opts.verbose);
    const skillContent = await getSkillContent();
    paths.writeFile(skillFile, skillContent);
  }
  writeOwner("claude", "caveman");
  return true;
}

function removeCaveman(): void {
  const p = paths.claudePaths();
  const skillDir = join(p.skillsDir, "caveman");
  try {
    rmSync(skillDir, { recursive: true, force: true });
  } catch {}
  removeOwner("claude", "caveman");
}

function hasCavemanSkill(): boolean {
  const p = paths.claudePaths();
  if (existsSync(join(p.skillsDir, "caveman", "SKILL.md"))) return true;
  // Check unified owner
  return hasOwner("claude", "caveman");
}
