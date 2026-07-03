import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, join } from "node:path";

function home(): string {
  return homedir();
}

// ─── Claude Code ─────────────────────────────────────────────

export interface ClaudePaths {
  dir: string;
  globalJson: string;
  settings: string;
  skillsDir: string;
  agentsMd: string;
}

export function claudePaths(): ClaudePaths {
  const h = home();
  const dir = join(h, ".claude");
  return {
    dir,
    globalJson: join(h, ".claude.json"),
    settings: join(dir, "settings.json"),
    skillsDir: join(dir, "skills"),
    agentsMd: join(dir, "AGENTS.md"),
  };
}

export function claudeKnownBinDirs(): string[] {
  return [join(home(), ".local", "bin")];
}

export function claudeDesktopPaths(): string[] {
  if (process.platform === "win32") {
    const paths: string[] = [];
    const local = process.env.LOCALAPPDATA;
    if (local) paths.push(join(local, "AnthropicClaude", "claude.exe"));
    const roam = process.env.APPDATA;
    if (roam) paths.push(join(roam, "Claude", "claude.exe"));
    return paths;
  }
  if (process.platform === "darwin") {
    return ["/Applications/Claude.app"];
  }
  return [];
}

// ─── OpenCode ────────────────────────────────────────────────

export interface OpenCodePaths {
  dir: string;
  config: string;
  agentsMd: string;
}

export function opencodePaths(): OpenCodePaths {
  let dir: string;
  if (process.platform === "win32") {
    const roam = process.env.APPDATA;
    dir = roam ? join(roam, "opencode") : join(home(), ".config", "opencode");
  } else {
    dir = join(home(), ".config", "opencode");
  }
  return {
    dir,
    config: join(dir, "config.json"),
    agentsMd: join(dir, "AGENTS.md"),
  };
}

export function opencodeKnownBinDirs(): string[] {
  const dirs = [join(home(), ".opencode", "bin"), join(home(), ".local", "bin")];
  if (process.platform === "win32") {
    dirs.push(join(home(), "scoop", "shims"));
  }
  return dirs;
}

export function opencodeDesktopPaths(): string[] {
  if (process.platform === "win32") {
    const local = process.env.LOCALAPPDATA;
    if (local) return [join(local, "Programs", "OpenCode", "OpenCode.exe")];
    return [];
  }
  if (process.platform === "darwin") {
    return ["/Applications/OpenCode.app"];
  }
  return ["/usr/bin/ai.opencode.desktop"];
}

// ─── Codex (OpenAI) ──────────────────────────────────────────

export interface CodexPaths {
  dir: string;
  config: string;
  hooks: string;
  instructions: string;
}

export function codexPaths(): CodexPaths {
  const dir = join(home(), ".codex");
  return {
    dir,
    config: join(dir, "config.toml"),
    hooks: join(dir, "hooks.json"),
    instructions: join(dir, "instructions.md"),
  };
}

export function codexKnownBinDirs(): string[] {
  const dirs: string[] = [];
  const envDir = process.env.CODEX_INSTALL_DIR;
  if (envDir) dirs.push(envDir);
  if (process.platform === "win32") {
    const la = process.env.LOCALAPPDATA;
    if (la) dirs.push(join(la, "Programs", "OpenAI", "Codex", "bin"));
  }
  dirs.push(join(home(), ".local", "bin"));
  dirs.push(join(home(), ".cargo", "bin"));
  return dirs;
}

// ─── Antigravity (Google) ────────────────────────────────────

export interface AntigravityPaths {
  dir: string;
  mcpCli: string;
  mcpIde: string;
  settingsCli: string;
  settingsIde: string;
  hooks: string;
  agentsMd: string;
}

export function antigravityPaths(): AntigravityPaths {
  const gemini = join(home(), ".gemini");
  return {
    dir: gemini,
    mcpCli: join(gemini, "antigravity-cli", "mcp_config.json"),
    mcpIde: join(gemini, "antigravity-ide", "mcp_config.json"),
    settingsCli: join(gemini, "antigravity-cli", "settings.json"),
    settingsIde: join(gemini, "antigravity-ide", "settings.json"),
    hooks: join(gemini, "config", "hooks.json"),
    agentsMd: join(gemini, "config", "AGENTS.md"),
  };
}

/** All MCP config files that Antigravity reads. */
export function antigravityMcpFiles(): string[] {
  const gemini = join(home(), ".gemini");
  const files = [
    join(gemini, "antigravity-cli", "mcp_config.json"),
    join(gemini, "antigravity-ide", "mcp_config.json"),
  ];
  const desktop = join(gemini, "antigravity-desktop");
  if (existsSync(desktop)) {
    files.push(join(desktop, "mcp_config.json"));
  }
  return files;
}

/** All settings.json files that need permissions entries. */
export function antigravitySettingsFiles(): string[] {
  const gemini = join(home(), ".gemini");
  return [
    join(gemini, "antigravity-cli", "settings.json"),
    join(gemini, "antigravity-ide", "settings.json"),
  ];
}

export function antigravityKnownBinDirs(): string[] {
  if (process.platform === "win32") {
    const local = process.env.LOCALAPPDATA;
    return local ? [join(local, "agy", "bin")] : [];
  }
  return [join(home(), ".local", "bin")];
}

export function antigravityDesktopPaths(): string[] {
  if (process.platform === "win32") {
    const local = process.env.LOCALAPPDATA;
    if (!local) return [];
    return [
      join(local, "Programs", "Antigravity", "Antigravity.exe"),
      join(local, "Programs", "Antigravity IDE", "Antigravity IDE.exe"),
    ];
  }
  if (process.platform === "darwin") {
    return ["/Applications/Antigravity.app", "/Applications/Antigravity IDE.app"];
  }
  return ["/opt/antigravity", "/opt/antigravity-ide"];
}

// ─── Shared ──────────────────────────────────────────────────

/** Local bin directory for tool installs. */
export function localBin(): string {
  if (process.platform === "win32") {
    const la = process.env.LOCALAPPDATA;
    if (la) return join(la, "Programs", "toksave");
    // Fallback when LOCALAPPDATA is not set
    return join(home(), "AppData", "Local", "Programs", "toksave");
  }
  return join(home(), ".local", "bin");
}

/** Cache directory for toksave. */
export function cacheDir(): string {
  if (process.env.TOKSAVE_CACHE_DIR) return process.env.TOKSAVE_CACHE_DIR;
  return join(home(), ".cache", "toksave");
}

/** Ensure a directory exists. */
export function ensureDir(path: string): void {
  if (!existsSync(path)) {
    mkdirSync(path, { recursive: true });
  }
}

/** Read a file, returning null if it doesn't exist. */
export function readFile(path: string): string | null {
  try {
    return readFileSync(path, "utf-8");
  } catch {
    return null;
  }
}

/** Write a file, creating parent dirs if needed. */
export function writeFile(path: string, content: string): void {
  ensureDir(dirname(path));
  const tmp = `${path}.${process.pid}.tmp`;
  writeFileSync(tmp, content, "utf-8");
  renameSync(tmp, path);
}

/** Append to a file, creating it if needed. */
export function appendFile(path: string, content: string): void {
  ensureDir(dirname(path));
  appendFileSync(path, content, "utf-8");
}

/** Get the toksave binary absolute path or alias. */
export function toksaveAbs(): string {
  if (process.env.NODE_ENV === "test") return "toksave";

  const candidates = [process.argv[1], process.execPath, join(localBin(), "toksave")];
  if (process.platform === "win32") candidates.push(join(localBin(), "toksave.exe"));

  for (const candidate of candidates) {
    if (!candidate || !existsSync(candidate)) continue;
    const name = basename(candidate).toLowerCase();
    if (name === "toksave" || name === "toksave.exe") return candidate;
  }

  return "toksave";
}
