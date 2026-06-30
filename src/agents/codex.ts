import { existsSync } from "node:fs";
import { getOrCreateObject, readJsonFile, writeJsonFile } from "../config/json.js";
import * as toml from "../config/toml.js";
import {
  CTX_RULES_BLOCK,
  hasCtxRules,
  removeCtxRules as stripCtxRules,
} from "../content/ctx-rules.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn, isOnPath } from "../util/detect.js";
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
      return wireMcp("codegraph", "codegraph", ["serve", "--mcp"], opts);
    case "context-mode":
      wireMcp("context-mode", "context-mode", [], opts);
      if (!opts.dryRun) wireCtxRules(opts);
      return true;
    case "rtk":
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
    case "rtk":
      return hasRtkHook();
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
  toml.setTopKey(doc, "approval_policy", "never");

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

  const cfg = readJsonFile(p.hooks) ?? {};
  const hooks = getOrCreateObject(cfg, "hooks");

  hooks.PreToolUse = [
    {
      matcher: "Bash",
      hooks: [{ type: "command", command, timeout: 10 }],
    },
  ];

  writeJsonFile(p.hooks, cfg);
  return true;
}

function removeRtkHook(): void {
  const p = paths.codexPaths();
  const cfg = readJsonFile(p.hooks);
  if (cfg?.hooks?.PreToolUse) {
    cfg.hooks.PreToolUse = undefined;
    writeJsonFile(p.hooks, cfg);
  }
}

function hasRtkHook(): boolean {
  const p = paths.codexPaths();
  const cfg = readJsonFile(p.hooks);
  const arr = cfg?.hooks?.PreToolUse;
  if (!Array.isArray(arr)) return false;
  return arr.some((g: any) => g?.hooks?.some((h: any) => h?.command?.includes("rtk-hook codex")));
}

// ─── Caveman via instructions.md ─────────────────────────────

const CAVEMAN_INSTRUCTIONS_BLOCK = `
<!-- CAVEMAN_START — managed by toksave, do not edit -->
## Caveman Mode

Respond terse like smart caveman. All technical substance stay. Only fluff die.
Drop: articles (a/an/the), filler, pleasantries, hedging. Fragments OK.
Pattern: [thing] [action] [reason]. [next step].
Code/commits: write normal. "stop caveman": revert.
<!-- CAVEMAN_END -->
`;

function wireCaveman(opts: RunOpts): boolean {
  if (opts.dryRun) return true;
  const p = paths.codexPaths();
  paths.ensureDir(p.dir);

  verbose("Writing Caveman rules into Codex instructions.md", opts.verbose);

  const existing = paths.readFile(p.instructions) ?? "";
  if (existing.includes("CAVEMAN_START")) return true;

  paths.writeFile(p.instructions, `${existing}\n${CAVEMAN_INSTRUCTIONS_BLOCK}`);
  return true;
}

function removeCavemanRules(): void {
  const p = paths.codexPaths();
  const existing = paths.readFile(p.instructions);
  if (!existing) return;
  const stripped = existing.replace(/\n?<!-- CAVEMAN_START[\s\S]*?CAVEMAN_END -->\n?/g, "").trim();
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
