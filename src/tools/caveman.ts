import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, join } from "node:path";
import { readJsonFile, writeJsonFile } from "../config/json.js";
import { CAVEMAN_SKILL_MD, CAVEMAN_SKILL_VERSION } from "../content/caveman-skill.js";
import type { RunOpts } from "../registry.js";
import { isOnPath } from "../util/detect.js";
import type { HealthStatus, RepairResult } from "../util/health.js";
import * as paths from "../util/paths.js";
import { userAgent } from "../util/version.js";

export const CAVEMAN_SKILL_NAMES = [
  "caveman",
  "caveman-commit",
  "caveman-compress",
  "caveman-help",
  "caveman-review",
  "caveman-stats",
  "cavecrew",
];

const CAVEMAN_OPENCODE_PLUGIN_REL = "./plugins/caveman/plugin.js";

/** Install caveman: try npm global install from github, fallback no-op. */
export async function install(opts: RunOpts): Promise<boolean> {
  if (!opts.dryRun && process.env.NODE_ENV !== "test" && !opts.upgrade && isOnPath("caveman")) {
    return true;
  }
  if (opts.dryRun || process.env.NODE_ENV === "test") return true;

  if (!isOnPath("npm")) return false;

  try {
    const result = spawnSync("npm", ["install", "-g", "github:JuliusBrussee/caveman"], {
      stdio: "pipe",
      timeout: 15 * 60 * 1000,
    });
    return result.status === 0 || isOnPath("caveman");
  } catch {
    return isOnPath("caveman");
  }
}

/** Get installed Caveman skill version by reading skill file. */
export function installedVersion(): string | null {
  const instructionFiles = [paths.opencodePaths().agentsMd, paths.codexPaths().instructions];
  for (const instructionFile of instructionFiles) {
    const content = paths.readFile(instructionFile);
    if (content?.includes("CAVEMAN_START")) return CAVEMAN_SKILL_VERSION;
  }

  const skillPaths = [
    join(paths.claudePaths().skillsDir, "caveman/SKILL.md"),
    join(paths.antigravityPaths().dir, "config", "skills", "caveman", "SKILL.md"),
  ];

  for (const skillPath of skillPaths) {
    try {
      if (!existsSync(skillPath)) continue;

      const content = readFileSync(skillPath, "utf-8");
      const versionMatch = content.match(/^version:\s*(.+)$/m);
      if (versionMatch?.[1]) return versionMatch[1].trim();

      return CAVEMAN_SKILL_VERSION;
    } catch {}
  }

  return null;
}

/** Get latest Caveman skill version from GitHub releases. */
export async function latestVersion(): Promise<string | null> {
  try {
    const resp = await fetch("https://api.github.com/repos/JuliusBrussee/caveman/releases/latest", {
      headers: { "User-Agent": userAgent() },
      signal: AbortSignal.timeout(10000),
    });
    if (!resp.ok) return null;
    const json = await resp.json();
    return json.tag_name?.replace(/^v/, "").trim() || null;
  } catch {
    return null;
  }
}

/** Fetch official SKILL.md from JuliusBrussee/caveman GitHub repo. */
async function fetchOfficialSkill(): Promise<string | null> {
  try {
    const resp = await fetch(
      "https://raw.githubusercontent.com/JuliusBrussee/caveman/main/skills/caveman/SKILL.md",
      {
        headers: { "User-Agent": userAgent() },
        signal: AbortSignal.timeout(10000),
      },
    );
    if (!resp.ok) return null;
    return await resp.text();
  } catch {
    return null;
  }
}

/** Get Caveman SKILL.md content - fetches from GitHub, falls back to local template. */
export async function getSkillContent(): Promise<string> {
  const official = await fetchOfficialSkill();
  return official ?? CAVEMAN_SKILL_MD;
}

/** Get Caveman instruction block for AGENTS.md/instructions.md - fetches from GitHub. */
export async function getCavemanInstructionBlock(): Promise<string> {
  const skillContent = await getSkillContent();

  const lines = skillContent.split("\n");
  let inFrontmatter = false;
  const contentLines: string[] = [];

  for (const line of lines) {
    if (line.trim() === "---") {
      inFrontmatter = !inFrontmatter;
      continue;
    }
    if (!inFrontmatter && line.trim()) {
      contentLines.push(line);
    }
  }

  const coreContent = contentLines.slice(0, 20).join("\n");

  return `
<!-- CAVEMAN_START — managed by toksave, do not edit -->
${coreContent}
<!-- CAVEMAN_END -->
`.trim();
}

// ─── Helpers for tokless parity ────────────────────────────────

export function skillsAgentID(agent: string): string {
  if (agent === "copilot") return "github-copilot";
  return agent;
}

export function resolveCavemanBin(
  agent: string,
  upgrade: boolean,
): { bin: string; args: string[] } {
  const useNpx = !isOnPath("caveman");
  const bin = useNpx ? "npx" : "caveman";
  const args = useNpx
    ? ["-y", "github:JuliusBrussee/caveman", "--", "--only", agent, "--no-mcp-shrink"]
    : ["--only", agent, "--no-mcp-shrink"];
  if (upgrade) args.push("--force");
  return { bin, args };
}

export function resolveSkillsBin(npxArgs: string[]): { bin: string; args: string[] } {
  if (isOnPath("skills")) {
    return { bin: "skills", args: npxArgs.slice(2) };
  }
  return { bin: "npx", args: npxArgs };
}

export function cavemanSkillsAddArgs(agent: string): string[] {
  return [
    "-y",
    "skills",
    "add",
    "JuliusBrussee/caveman",
    "-a",
    skillsAgentID(agent),
    "-s",
    "*",
    "--yes",
    "-g",
  ];
}

export function cavemanSkillsRemoveArgs(agent: string): string[] {
  return ["-y", "skills", "remove", ...CAVEMAN_SKILL_NAMES, "-a", skillsAgentID(agent), "-y", "-g"];
}

export function cavemanExec(bin: string, args: string[], opts: RunOpts, _dryHint: string): boolean {
  if (opts.dryRun) return true;
  if (process.env.NODE_ENV === "test") return true;
  try {
    const result = spawnSync(bin, args, { stdio: "pipe", timeout: 15 * 60 * 1000 });
    return result.status === 0;
  } catch {
    return false;
  }
}

export function cavemanOpencodeInstallEnv(): Record<string, string> | undefined {
  const dir = paths.opencodePaths().dir;
  if (basename(dir) !== "opencode") return undefined;
  return { XDG_CONFIG_HOME: dirname(dir) };
}

// ─── OpenCode plugin registration ─────────────────────────────

export function registerCavemanOpencode(): void {
  const p = paths.opencodePaths();
  paths.ensureDir(p.dir);
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {};
  if (!cfg.$schema) cfg.$schema = "https://opencode.ai/config.json";

  const mcp = cfg.mcp as Record<string, unknown> | undefined;
  if (mcp?.["caveman-shrink"]) {
    delete mcp["caveman-shrink"];
    if (Object.keys(mcp).length === 0) delete cfg.mcp;
  }

  const plugins = (cfg.plugin as unknown[]) ?? [];
  if (!plugins.some((pl) => typeof pl === "string" && pl.toLowerCase().includes("caveman"))) {
    plugins.push(CAVEMAN_OPENCODE_PLUGIN_REL);
  }
  cfg.plugin = plugins;
  writeJsonFile(p.config, cfg);
}

export function unregisterCavemanOpencode(): void {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) as Record<string, unknown> | null;
  if (!cfg) return;

  const plugins = cfg.plugin as unknown[] | undefined;
  if (Array.isArray(plugins)) {
    cfg.plugin = plugins.filter(
      (pl) => !(typeof pl === "string" && pl === CAVEMAN_OPENCODE_PLUGIN_REL),
    );
  }

  const mcp = cfg.mcp as Record<string, unknown> | undefined;
  if (mcp?.["caveman-shrink"]) {
    delete mcp["caveman-shrink"];
    if (Object.keys(mcp).length === 0) delete cfg.mcp;
  }
  writeJsonFile(p.config, cfg);
}

export function opencodePluginInstalled(): boolean {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) as Record<string, unknown> | null;
  if (!cfg) return false;
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return false;
  return plugins.some((pl) => typeof pl === "string" && pl.toLowerCase().includes("caveman"));
}

export function opencodePluginFilesPresent(): boolean {
  return existsSync(join(paths.opencodePaths().dir, "plugins", "caveman", "plugin.js"));
}

// ─── Per-agent verification helpers ───────────────────────────

export function claudeCavemanInstalled(): boolean {
  const p = paths.claudePaths();
  if (existsSync(join(p.dir, ".caveman-active"))) return true;
  const settings = paths.readFile(p.settings);
  return !!settings?.toLowerCase().includes("caveman");
}

export function codexCavemanInstalled(): boolean {
  const p = paths.codexPaths();
  if (existsSync(join(p.dir, "skills", "caveman"))) return true;
  return existsSync(join(homedir(), ".agents", "skills", "caveman"));
}

export function antigravityCavemanInstalled(): boolean {
  const gemini = paths.antigravityPaths().dir;
  return (
    existsSync(join(gemini, "config", "skills", "caveman")) ||
    existsSync(join(gemini, "antigravity", "skills", "caveman"))
  );
}

export function copilotCavemanInstalled(): boolean {
  const p = paths.copilotPaths();
  return (
    existsSync(join(p.skillsDir, "caveman")) ||
    existsSync(join(homedir(), ".agents", "skills", "caveman"))
  );
}

// ─── Skill relocation ──────────────────────────────────────────

export function relocateCavemanSkills(dstDir: string): void {
  const src = join(homedir(), ".agents", "skills");
  for (const name of CAVEMAN_SKILL_NAMES) {
    const s = join(src, name);
    if (!existsSync(s)) continue;
    const d = join(dstDir, name);
    paths.ensureDir(d);
    try {
      const files = readdirSync(s);
      for (const f of files) {
        const content = readFileSync(join(s, f), "utf-8");
        paths.writeFile(join(d, f), content);
      }
      rmSync(s, { recursive: true, force: true });
    } catch {}
  }
}

export function removeCavemanSkillCopies(dir: string): void {
  for (const name of CAVEMAN_SKILL_NAMES) {
    try {
      rmSync(join(dir, name), { recursive: true, force: true });
    } catch {}
  }
}

// ─── Health & Repair ───────────────────────────────────────────

/** Check if Caveman skill files are installed. */
export function healthCheck(): HealthStatus {
  const version = installedVersion();

  if (!version) {
    return {
      healthy: false,
      version: null,
      issues: [
        {
          severity: "error",
          message: "Caveman skill not found",
          remediation: "Run: toksave install caveman",
        },
      ],
    };
  }

  return {
    healthy: true,
    version,
    issues: [],
  };
}

/** Attempt to repair Caveman installation. */
export async function repair(_opts: RunOpts): Promise<RepairResult> {
  const beforeHealth = healthCheck();

  if (beforeHealth.healthy) {
    return {
      success: true,
      message: "Caveman is already healthy, no repair needed",
      healthAfterRepair: beforeHealth,
    };
  }

  return {
    success: false,
    message: "Caveman repair requires running: toksave init caveman",
  };
}
