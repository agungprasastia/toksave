import { existsSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import {
  addToArrayIfMissing,
  getOrCreateObject,
  readJsonFile,
  writeJsonFile,
} from "../config/json.js";
import type { Detection, RunOpts, ToolId } from "../registry.js";
import { getSkillContent } from "../tools/caveman.js";
import { verbose } from "../util/colors.js";
import { findBinaryIn } from "../util/detect.js";
import * as paths from "../util/paths.js";
import { hasOwner, removeOwner, writeOwner } from "../util/unified-block.js";

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
      wireMcp(
        "codegraph",
        paths.toksaveAbs(),
        ["runmcp", "--agent", "antigravity", "codegraph", "serve", "--mcp"],
        opts,
      );
      if (!opts.dryRun) {
        allowEntry("mcp(codegraph/*)");
        writeOwner("antigravity", "codegraph");
        installCodegraphIndexHook();
      }
      return true;
    case "context-mode":
      wireMcp(
        "context-mode",
        paths.toksaveAbs(),
        ["runmcp", "--agent", "antigravity", "context-mode"],
        opts,
      );
      if (!opts.dryRun) {
        allowEntry("mcp(context-mode/*)");
        installContextModeHook(opts);
        writeOwner("antigravity", "context-mode");
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
    case "ponytail":
      if (!opts.dryRun) writeOwner("antigravity", "ponytail");
      return true;
    case "principles":
      if (!opts.dryRun) writeOwner("antigravity", "principles");
      return true;
    default:
      return false;
  }
}

/** Unwire a tool from Antigravity. */
export async function unwire(tool: ToolId, _opts: RunOpts): Promise<boolean> {
  switch (tool) {
    case "codegraph":
      removeMcp("codegraph");
      removeOwner("antigravity", "codegraph");
      removeCodegraphIndexHook();
      return true;
    case "context-mode":
      removeMcp("context-mode");
      removeContextModeHook();
      removeOwner("antigravity", "context-mode");
      return true;
    case "rtk":
      removeRtkHook();
      return true;
    case "caveman":
      removeCaveman();
      return true;
    case "ponytail":
      removeOwner("antigravity", "ponytail");
      return true;
    case "principles":
      removeOwner("antigravity", "principles");
      return true;
    default:
      return false;
  }
}

/** Verify a tool is wired into Antigravity. */
export function verify(tool: ToolId): boolean | null {
  switch (tool) {
    case "codegraph":
      return hasMcp("codegraph");
    case "context-mode":
      return hasMcp("context-mode");
    case "rtk":
      return hasRtkHook();
    case "caveman":
      return hasCavemanSkill() || hasOwner("antigravity", "caveman");
    case "ponytail":
      return hasOwner("antigravity", "ponytail");
    case "principles":
      return hasOwner("antigravity", "principles");
    default:
      return null;
  }
}

// ─── MCP wiring (multi-surface, atomic) ─────────────────────

function wireMcp(toolId: string, command: string, args: string[], opts: RunOpts): void {
  if (opts.dryRun) return;

  verbose(`Wiring MCP ${toolId} into Antigravity (multi-surface)`, opts.verbose);

  const mcpFiles = paths.antigravityMcpFiles();
  const prepared: { file: string; cfg: Record<string, unknown> }[] = [];
  for (const f of mcpFiles) {
    paths.ensureDir(dirname(f));
    const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
    const servers = getOrCreateObject(cfg, "mcpServers");

    const entry: Record<string, unknown> = { command };
    if (args.length > 0) entry.args = args;
    entry.trust = true;

    (servers as Record<string, unknown>)[toolId] = entry;
    prepared.push({ file: f, cfg });
  }

  atomicWriteAll(prepared, `wire MCP ${toolId}`);
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
  const settingsFiles = paths.antigravitySettingsFiles();
  const prepared: { file: string; cfg: Record<string, unknown> }[] = [];
  for (const f of settingsFiles) {
    paths.ensureDir(dirname(f));
    const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
    const perms = getOrCreateObject(cfg, "permissions");
    if (!Array.isArray(perms.allow)) perms.allow = [];
    addToArrayIfMissing(perms.allow as unknown[], entry);
    prepared.push({ file: f, cfg });
  }

  atomicWriteAll(prepared, `allow ${entry}`);
}

function atomicWriteAll(
  writes: { file: string; cfg: Record<string, unknown> }[],
  operation: string,
): void {
  const originals = new Map<string, string | null>();
  for (const { file } of writes) {
    try {
      originals.set(file, paths.readFile(file));
    } catch {
      originals.set(file, null);
    }
  }

  const written: string[] = [];
  try {
    for (const { file, cfg } of writes) {
      writeJsonFile(file, cfg);
      written.push(file);
    }
  } catch (err) {
    for (const file of written) {
      const original = originals.get(file);
      if (original === null) {
        try {
          rmSync(file, { force: true });
        } catch {}
      } else if (original !== undefined) {
        try {
          paths.writeFile(file, original);
        } catch {}
      }
    }
    const msg = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to ${operation}: ${msg}`);
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
  const cfg = (readJsonFile(hooksFile) as Record<string, unknown>) ?? {};
  if ((cfg as Record<string, unknown>)?.rtk) {
    (cfg as Record<string, unknown>).rtk = undefined;
    writeJsonFile(hooksFile, cfg);
  }
}

function hasRtkHook(): boolean {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = (readJsonFile(hooksFile) as Record<string, unknown>) ?? {};
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
  const cfg = (readJsonFile(hooksFile) as Record<string, unknown>) ?? {};
  if ((cfg as Record<string, unknown>)?.ctx) {
    (cfg as Record<string, unknown>).ctx = undefined;
    writeJsonFile(hooksFile, cfg);
  }
}

// ─── Codegraph index hook ───────────────────────────────────

function installCodegraphIndexHook(): void {
  const hooksFile = paths.antigravityPaths().hooks;
  paths.ensureDir(dirname(hooksFile));
  const cfg = (readJsonFile(hooksFile) as Record<string, unknown>) ?? {};
  const existing = (cfg as Record<string, unknown>).codegraphIndex as
    | Record<string, unknown>
    | undefined;
  if (existing) return;
  // Use generic hooks.json SessionStart style similar to tokless
  const hooks = getOrCreateObject(cfg, "hooks");
  const ss = (hooks.SessionStart as unknown[]) ?? [];
  const command = `${paths.toksaveAbs()} agy-hook codegraph-index`;
  if (!ss.some((g) => JSON.stringify(g).includes("codegraph-index"))) {
    ss.push({
      matcher: ".*",
      hooks: [{ type: "command", command, timeout: 10 }],
    });
    hooks.SessionStart = ss as never;
    writeJsonFile(hooksFile, cfg);
  }
}

function removeCodegraphIndexHook(): void {
  const hooksFile = paths.antigravityPaths().hooks;
  const cfg = readJsonFile(hooksFile) as Record<string, unknown> | null;
  if (!cfg) return;
  const hooks = (cfg as Record<string, unknown>)?.hooks as Record<string, unknown> | undefined;
  if (!hooks?.SessionStart) {
    // Also check top-level codegraphIndex
    if ((cfg as Record<string, unknown>).codegraphIndex) {
      delete (cfg as Record<string, unknown>).codegraphIndex;
      writeJsonFile(hooksFile, cfg);
    }
    return;
  }
  const ss = hooks.SessionStart as unknown[] | undefined;
  if (!Array.isArray(ss)) return;
  const filtered = ss.filter((g) => !JSON.stringify(g).includes("codegraph-index"));
  if (filtered.length !== ss.length) {
    if (filtered.length === 0) delete hooks.SessionStart;
    else hooks.SessionStart = filtered as never;
    writeJsonFile(hooksFile, cfg);
  }
}

// ─── Caveman ─────────────────────────────────────────────────

async function wireCaveman(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) {
    writeOwner("antigravity", "caveman");
    return true;
  }
  const gemini = paths.antigravityPaths().dir;
  const skillDir = join(gemini, "config", "skills", "caveman");
  paths.ensureDir(skillDir);
  const skillFile = join(skillDir, "SKILL.md");
  if (!existsSync(skillFile) || opts.upgrade) {
    verbose("Writing Caveman SKILL.md for Antigravity", opts.verbose);
    const skillContent = await getSkillContent();
    paths.writeFile(skillFile, skillContent);
  }
  writeOwner("antigravity", "caveman");
  return true;
}

function removeCaveman(): void {
  const gemini = paths.antigravityPaths().dir;
  const skillDir = join(gemini, "config", "skills", "caveman");
  try {
    rmSync(skillDir, { recursive: true, force: true });
  } catch {}
  removeOwner("antigravity", "caveman");
}

function hasCavemanSkill(): boolean {
  const gemini = paths.antigravityPaths().dir;
  return existsSync(join(gemini, "config", "skills", "caveman", "SKILL.md"));
}
