import { existsSync } from "node:fs";
import { getOrCreateObject, readJsonFile, writeJsonFile } from "../config/json.js";
import * as toml from "../config/toml.js";
import {
  CTX_RULES_BLOCK,
  hasCtxRules,
  removeCtxRules as stripCtxRules,
} from "../content/ctx-rules.js";
import { hasRtkRules, RTK_RULES_BLOCK, removeRtkRules } from "../content/rtk-rules.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { getCavemanInstructionBlock } from "../tools/caveman.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";

/** Detect if Codex is installed. */
export function detect(): Detection {
  const hasCli = !!findBinaryIn("codex", paths.codexKnownBinDirs());
  if (hasCli) return { installed: true, source: "cli" };
  if (existsSync(paths.codexPaths().dir)) return { installed: true, source: "config" };
  return { installed: false, source: "" };
}

/** Wire a tool into Codex. */
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
    case "rtk":
      if (!opts.dryRun) wireRtkRules(opts);
      return wireRtkHook(opts);
    case "caveman":
      return wireCaveman(opts);
  }
}

/** Unwire a tool from Codex. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeCtxRulesFile();
      return true;
    case "rtk":
      removeRtkHook();
      removeRtkRulesFile();
      return true;
    case "caveman":
      removeCavemanRules();
      return true;
  }
}

/** Verify a tool is wired into Codex. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasMcp("context-mode");
    case "rtk": {
      const rules = paths.readFile(paths.codexPaths().instructions);
      return hasRtkHook() && !!rules && hasRtkRules(rules);
    }
    case "caveman":
      return hasCavemanRules();
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
  } catch {
    /* ignore */
  }
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
  const cfg = readJsonFile(p.hooks) as Record<string, unknown>;
  const hooks = ((cfg as Record<string, unknown>)?.hooks as Record<string, unknown>) ?? {};
  hooks.PreToolUse = removeToksaveHookGroups(hooks.PreToolUse, "rtk-hook codex");
  hooks.PermissionRequest = removeToksaveHookGroups(hooks.PermissionRequest, "codex-perm-hook");
  writeJsonFile(p.hooks, cfg);
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
  const p = paths.codexPaths();
  const cfg = readJsonFile(p.hooks) as Record<string, unknown>;
  const arr = ((cfg as Record<string, unknown>)?.hooks as Record<string, unknown>)?.PreToolUse;
  if (!Array.isArray(arr)) return false;
  return arr.some((g: unknown) => {
    const group = g as { hooks?: { command?: string }[] };
    return group?.hooks?.some((h) => h?.command?.includes("rtk-hook codex"));
  });
}

// ─── Caveman via instructions.md ─────────────────────────────

async function wireCaveman(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) return true;
  const p = paths.codexPaths();
  paths.ensureDir(p.dir);

  verbose("Writing Caveman rules into Codex instructions.md", opts.verbose);

  const existing = paths.readFile(p.instructions) ?? "";
  if (existing.includes("CAVEMAN_START") && !opts.upgrade) return true;

  const cavemanBlock = await getCavemanInstructionBlock();
  const stripped = existing.replace(
    /\r?\n?<!--\s*CAVEMAN_START[\s\S]*?CAVEMAN_END\s*-->\r?\n?/g,
    "\n",
  );
  paths.writeFile(p.instructions, `${stripped}\n${cavemanBlock}`);
  return true;
}

function removeCavemanRules(): void {
  const p = paths.codexPaths();
  const existing = paths.readFile(p.instructions);
  if (!existing) return;
  const stripped = existing
    .replace(/\r?\n?<!--\s*CAVEMAN_START[\s\S]*?CAVEMAN_END\s*-->\r?\n?/g, "")
    .trim();
  paths.writeFile(p.instructions, stripped);
}

function hasCavemanRules(): boolean {
  const p = paths.codexPaths();
  const existing = paths.readFile(p.instructions);
  return !!existing?.includes("CAVEMAN_START");
}

// ─── Context-Mode rules ─────────────────────────────────────

function wireCtxRules(opts: RunOpts): void {
  const p = paths.codexPaths();
  paths.ensureDir(p.dir);

  verbose("Injecting Context-Mode rules into Codex instructions.md", opts.verbose);

  const existing = paths.readFile(p.instructions) ?? "";
  if (hasCtxRules(existing)) return;

  paths.writeFile(p.instructions, `${existing}\n${CTX_RULES_BLOCK}`);
}

function removeCtxRulesFile(): void {
  const p = paths.codexPaths();
  const existing = paths.readFile(p.instructions);
  if (!existing) return;
  paths.writeFile(p.instructions, stripCtxRules(existing));
}

function wireRtkRules(opts: RunOpts): void {
  const p = paths.codexPaths();
  verbose("Injecting RTK rules into Codex instructions.md", opts.verbose);

  const existing = paths.readFile(p.instructions) ?? "";
  if (hasRtkRules(existing) && !opts.upgrade) return;

  paths.writeFile(p.instructions, `${removeRtkRules(existing)}\n${RTK_RULES_BLOCK}`);
}

function removeRtkRulesFile(): void {
  const p = paths.codexPaths();
  const existing = paths.readFile(p.instructions);
  if (!existing) return;
  paths.writeFile(p.instructions, removeRtkRules(existing));
}
