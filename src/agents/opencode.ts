import { existsSync } from "node:fs";
import { getOrCreateObject, hasKey, readJsonFile, writeJsonFile } from "../config/json.js";
import {
  CTX_RULES_BLOCK,
  hasCtxRules,
  removeCtxRules as stripCtxRules,
} from "../content/ctx-rules.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
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
      return wireMcp("codegraph", ["codegraph", "serve", "--mcp"], opts);
    case "context-mode":
      wireMcp("context-mode", ["context-mode"], opts);
      if (!opts.dryRun) wireCtxRules(opts);
      return true;
    case "caveman":
      return wireCaveman(opts);
    case "rtk":
      return true; // RTK wiring for OpenCode via AGENTS.md
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
    case "rtk":
      return isOnPath("rtk");
  }
}

// ─── MCP wiring ──────────────────────────────────────────────

function wireMcp(toolId: string, command: string[], opts: RunOpts): boolean {
  if (opts.dryRun) return true;

  verbose(`Wiring MCP ${toolId} into OpenCode`, opts.verbose);

  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) ?? {};

  // Ensure $schema
  if (!hasKey(cfg, "$schema")) {
    cfg.$schema = "https://opencode.ai/config.json";
  }

  const mcp = getOrCreateObject(cfg, "mcp");
  const entry = { type: "local", command, enabled: true };

  if (mcp[toolId] && JSON.stringify(mcp[toolId]) === JSON.stringify(entry)) {
    return true;
  }

  mcp[toolId] = entry;
  writeJsonFile(p.config, cfg);
  return true;
}

function removeMcp(toolId: string): void {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config);
  if (cfg?.mcp?.[toolId]) {
    delete cfg.mcp[toolId];
    writeJsonFile(p.config, cfg);
  }
}

function hasMcp(toolId: string): boolean {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config);
  return !!cfg?.mcp?.[toolId];
}

// ─── Caveman via AGENTS.md ──────────────────────────────────

const CAVEMAN_AGENTS_BLOCK = `
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
  const p = paths.opencodePaths();
  paths.ensureDir(p.dir);

  verbose("Writing Caveman rules into OpenCode AGENTS.md", opts.verbose);

  const existing = paths.readFile(p.agentsMd) ?? "";
  if (existing.includes("CAVEMAN_START")) return true;

  paths.writeFile(p.agentsMd, `${existing}\n${CAVEMAN_AGENTS_BLOCK}`);
  return true;
}

function removeCavemanRules(): void {
  const p = paths.opencodePaths();
  const existing = paths.readFile(p.agentsMd);
  if (!existing) return;
  const stripped = existing.replace(/\n?<!-- CAVEMAN_START[\s\S]*?CAVEMAN_END -->\n?/g, "").trim();
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
