import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { run } from "./exec.js";
import { ensureDir } from "./paths.js";

const MARK_START = "# >>> toksave path >>>";
const MARK_END = "# <<< toksave path <<<";

function home(): string {
  return process.env.HOME ?? process.env.USERPROFILE ?? "";
}

/** Directories that should be on PATH for toksave-installed tools. */
export function expectedBinDirs(): string[] {
  const h = home();
  if (process.platform === "win32") {
    return [join(h, ".local", "bin"), join(h, ".bun", "bin")];
  }
  return [join(h, ".local", "bin"), join(h, ".bun", "bin"), join(h, ".cargo", "bin")];
}

/** Prepend existing expected dirs to PATH for this process. */
export function ensureProcessPath(): string[] {
  const sep = process.platform === "win32" ? ";" : ":";
  const current = (process.env.PATH ?? "").split(sep);
  const inPath = new Set(current);
  const added: string[] = [];

  for (const dir of expectedBinDirs()) {
    if (!inPath.has(dir) && existsSync(dir)) {
      current.unshift(dir);
      added.push(dir);
    }
  }

  if (added.length > 0) {
    process.env.PATH = current.join(sep);
  }
  return added;
}

/** Persist expected dirs to shell rc (Unix) or user PATH registry (Windows). */
export function ensurePersistentPath(): string[] {
  if (process.platform === "win32") {
    return ensurePersistentPathWindows();
  }
  return ensurePersistentPathUnix();
}

function ensurePersistentPathWindows(): string[] {
  const missing = expectedBinDirs().filter((d) => existsSync(d));
  return persistWindowsPathDirs(missing);
}

function persistWindowsPathDirs(dirs: string[]): string[] {
  if (dirs.length === 0) return [];

  const quoted = dirs.map((d) => `'${d.replace(/'/g, "''")}'`).join(",");
  const ps = `$ErrorActionPreference='Stop'
$k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
$cur = ''
if ($null -ne $k.GetValue('Path')) {
  $cur = $k.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
}
$parts = $cur -split ';' | Where-Object { $_ -ne '' }
$expanded = $parts | ForEach-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\\') }
$add = @(${quoted})
$new = $parts
$changed = $false
foreach ($d in $add) {
  if ($expanded -notcontains $d.TrimEnd('\\')) { $new += $d; $changed = $true }
}
if ($changed) {
  $k.SetValue('Path', ($new -join ';'), [Microsoft.Win32.RegistryValueKind]::ExpandString)
  Write-Output 'changed'
}
$k.Close()`;

  const r = run("powershell", ["-NoProfile", "-Command", ps]);
  if (r.code !== 0 || !r.stdout.includes("changed")) return [];
  return dirs;
}

function ensurePersistentPathUnix(): string[] {
  const h = home();
  const block = renderUnixBlock(expectedBinDirs(), h);
  let rcs = candidateRcFiles(h).filter((f) => existsSync(f));
  if (rcs.length === 0) rcs = [join(h, ".profile")];

  const patched: string[] = [];
  for (const rc of rcs) {
    let before = "";
    try {
      before = readFileSync(rc, "utf-8");
    } catch {
      /* new file */
    }
    const after = upsertShellBlock(before, block);
    if (after !== before) {
      ensureDir(join(rc, ".."));
      writeFileSync(rc, after, "utf-8");
      patched.push(rc);
    }
  }
  return patched;
}

/** Build the marked block for Unix shell rc files. */
export function renderUnixBlock(dirs: string[], homeDir: string): string {
  const rel = dirs.map((d) =>
    d.startsWith(homeDir) ? `"$HOME${d.slice(homeDir.length)}"` : `"${d}"`,
  );
  return [
    MARK_START,
    "# Adds toksave tool bin dirs to PATH (rtk, bun, cargo).",
    `for d in ${rel.join(" ")}; do`,
    '  [ -d "$d" ] && case ":$PATH:" in *":$d:"*) ;; *) PATH="$d:$PATH" ;; esac',
    "done",
    "export PATH",
    MARK_END,
    "",
  ].join("\n");
}

function candidateRcFiles(homeDir: string): string[] {
  const shell = process.env.SHELL ?? "";
  if (shell.endsWith("zsh")) return [join(homeDir, ".zshrc")];
  if (shell.endsWith("bash")) {
    return [join(homeDir, ".bashrc"), join(homeDir, ".bash_profile")];
  }
  return [join(homeDir, ".zshrc"), join(homeDir, ".bashrc"), join(homeDir, ".profile")];
}

/** Upsert a marked block in shell rc content. */
export function upsertShellBlock(src: string, block: string): string {
  const re = new RegExp(`${escapeRegExp(MARK_START)}[\\s\\S]*?${escapeRegExp(MARK_END)}\\n?`);
  if (re.test(src)) return src.replace(re, block);
  const sep = src.length === 0 || src.endsWith("\n") ? "" : "\n";
  return `${src}${sep}\n${block}`;
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Heal PATH: prepend expected dirs to process PATH, then persist to shell rc / registry.
 * Skipped in test mode.
 */
export function selfHealPath(): { added: string[]; patched: string[] } {
  if (process.env.NODE_ENV === "test") return { added: [], patched: [] };
  const added = ensureProcessPath();
  const patched = ensurePersistentPath();
  return { added, patched };
}
