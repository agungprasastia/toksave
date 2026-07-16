import { existsSync } from "node:fs";
import { getOrCreateObject, readJsonFile, writeJsonFile } from "../config/json.js";
import * as toml from "../config/toml.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner, removeOwner, writeOwner } from "../util/unified-block.js";

/** Detect if Codex is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("codex", paths.codexKnownBinDirs());
  if (hasCli) return { installed: true, source: "cli" };
  if (process.env.NODE_ENV === "test" && existsSync(paths.codexPaths().dir)) {
    return { installed: true, source: "config" };
  }
  return { installed: false, source: "" };
}

/** Wire a tool into Codex. */
export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      wireMcp(
        "codegraph",
        paths.toksaveAbs(),
        ["runmcp", "--agent", "codex", "codegraph", "serve", "--mcp"],
        opts,
      );
      if (!opts.dryRun) {
        writeOwner("codex", "codegraph");
        unwireAutoIndexCodex();
      }
      return true;
    case "context-mode":
      wireMcp(
        "context-mode",
        paths.toksaveAbs(),
        ["runmcp", "--agent", "codex", "context-mode"],
        opts,
      );
      if (!opts.dryRun) {
        writeOwner("codex", "context-mode");
        cleanupCodexContextModeHooks();
      }
      return true;
    case "rtk":
      if (!opts.dryRun) wireRtkHook(opts);
      return true;
    case "caveman":
      if (!opts.dryRun) writeOwner("codex", "caveman");
      return true;
    case "ponytail":
      if (!opts.dryRun) writeOwner("codex", "ponytail");
      return true;
    case "principles":
      if (!opts.dryRun) writeOwner("codex", "principles");
      return true;
    default:
      return false;
  }
}

/** Unwire a tool from Codex. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      removeOwner("codex", "codegraph");
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeOwner("codex", "context-mode");
      // also cleanup hooks
      cleanupCodexContextModeHooks();
      return true;
    case "rtk":
      removeRtkHook();
      return true;
    case "caveman":
      removeOwner("codex", "caveman");
      return true;
    case "ponytail":
      removeOwner("codex", "ponytail");
      return true;
    case "principles":
      removeOwner("codex", "principles");
      return true;
    default:
      return false;
  }
}

/** Verify a tool is wired into Codex. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasMcp("context-mode");
    case "rtk":
      return hasRtkHook();
    case "caveman":
      return hasOwner("codex", "caveman");
    case "ponytail":
      return hasOwner("codex", "ponytail");
    case "principles":
      return hasOwner("codex", "principles");
    default:
      return null;
  }
}

// ─── MCP wiring (TOML) ──────────────────────────────────────

function wireMcp(toolId: string, command: string, args: string[], opts: RunOpts): boolean {
  if (opts.dryRun) return true;

  verbose(`Wiring MCP ${toolId} into Codex`, opts.verbose);

  const p = paths.codexPaths();
  paths.ensureDir(p.dir);

  const doc = toml.readTomlFile(p.config);
  const tablePath = `mcp_servers.${toolId}`;

  toml.upsertTable(doc, tablePath, { command });
  if (args.length > 0) {
    toml.setTableArray(doc, tablePath, "args", args);
  }
  toml.upsertTableBool(doc, tablePath, "enabled", true);

  toml.writeTomlFile(p.config, doc);
  return true;
}

function removeMcp(toolId: string): void {
  const p = paths.codexPaths();
  try {
    const doc = toml.readTomlFile(p.config);
    toml.removeTable(doc, `mcp_servers.${toolId}`);
    toml.writeTomlFile(p.config, doc);
  } catch {}
}

function hasMcp(toolId: string): boolean {
  const p = paths.codexPaths();
  try {
    const doc = toml.readTomlFile(p.config);
    return toml.hasTable(doc, `mcp_servers.${toolId}`);
  } catch {
    return false;
  }
}

// ─── RTK hook wiring ─────────────────────────────────────────

function wireRtkHook(opts: RunOpts): boolean {
  if (opts.dryRun) return true;

  verbose("Installing RTK hook for Codex", opts.verbose);

  const p = paths.codexPaths();
  paths.ensureDir(p.dir);

  const tok = paths.toksaveAbs();
  const command = `${tok} rtk-hook codex`;

  const cfg = (readJsonFile(p.hooks) as Record<string, unknown>) ?? {};
  const hooks = getOrCreateObject(cfg, "hooks");

  addHook(hooks, "PreToolUse", {
    matcher: "Bash",
    hooks: [{ type: "command", command, timeout: 10 }],
  });

  addHook(hooks, "PermissionRequest", {
    matcher: "",
    hooks: [{ type: "command", command: `${tok} codex-perm-hook`, timeout: 5 }],
  });

  writeJsonFile(p.hooks, cfg);
  return true;
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
  const p = paths.codexPaths();
  const cfg = (readJsonFile(p.hooks) as Record<string, unknown>) ?? {};
  const hooks = ((cfg as Record<string, unknown>)?.hooks as Record<string, unknown>) ?? {};
  hooks.PreToolUse = removeHookGroups(hooks.PreToolUse, "rtk-hook codex");
  hooks.PermissionRequest = removeHookGroups(hooks.PermissionRequest, "codex-perm-hook");
  writeJsonFile(p.hooks, cfg);
}

function removeHookGroups(groups: unknown, marker: string): unknown[] | undefined {
  if (!Array.isArray(groups)) return undefined;
  const remaining = groups.filter((group) => !hookGroupContainsMarker(group, marker));
  return remaining.length > 0 ? remaining : undefined;
}

function hookGroupContainsMarker(group: unknown, marker: string): boolean {
  const hooks = (group as { hooks?: { command?: string }[] })?.hooks;
  return Array.isArray(hooks) && hooks.some((hook) => hook.command?.includes(marker));
}

function hasRtkHook(): boolean {
  const p = paths.codexPaths();
  const cfg = (readJsonFile(p.hooks) as Record<string, unknown>) ?? {};
  const arr = ((cfg as Record<string, unknown>)?.hooks as Record<string, unknown>)?.PreToolUse;
  if (!Array.isArray(arr)) return false;
  return arr.some((g: unknown) => {
    const group = g as { hooks?: { command?: string }[] };
    return group?.hooks?.some((h) => h?.command?.includes("rtk-hook codex"));
  });
}

// ─── Auto-index legacy cleanup ──────────────────────────────

function unwireAutoIndexCodex(): void {
  const p = paths.codexPaths();
  const hooksPath = p.hooks;
  if (!existsSync(hooksPath)) return;
  try {
    const cfg = readJsonFile(hooksPath) as Record<string, unknown>;
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
      writeJsonFile(hooksPath, cfg);
    }
  } catch {}
}

function cleanupCodexContextModeHooks(): void {
  const p = paths.codexPaths();
  // Clean hooks.json if it has context-mode hook codex entries
  if (!existsSync(p.hooks)) return;
  try {
    const cfg = readJsonFile(p.hooks) as Record<string, unknown>;
    const hooks = (cfg?.hooks as Record<string, unknown>) ?? {};
    let changed = false;
    for (const event of [
      "PreToolUse",
      "PostToolUse",
      "UserPromptSubmit",
      "SessionStart",
      "PreCompact",
      "Stop",
      "PermissionRequest",
    ]) {
      const arr = hooks[event] as unknown[] | undefined;
      if (!Array.isArray(arr)) continue;
      const filtered = arr.filter((entry) => !isCtxHookForEvent(entry, event));
      if (filtered.length !== arr.length) {
        if (filtered.length === 0) delete hooks[event];
        else hooks[event] = filtered as never;
        changed = true;
      }
    }
    if (changed) {
      if (Object.keys(hooks).length === 0) delete (cfg as Record<string, unknown>).hooks;
      writeJsonFile(p.hooks, cfg);
    }
  } catch {}
}

function isCtxHookForEvent(entry: unknown, _event: string): boolean {
  const em = entry as Record<string, unknown> | undefined;
  const inner = em?.hooks as unknown[] | undefined;
  if (!Array.isArray(inner)) return false;
  for (const h of inner) {
    const hm = h as Record<string, unknown> | undefined;
    const cmd = hm?.command as string | undefined;
    if (
      cmd &&
      (cmd.includes("context-mode hook codex") ||
        cmd.includes("context-mode-hook codex") ||
        cmd.includes("toksave codex-sessionstart"))
    ) {
      return true;
    }
  }
  return false;
}
