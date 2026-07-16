import { spawn } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const PROJECT_MARKERS = [
  ".git",
  "package.json",
  "go.mod",
  "Cargo.toml",
  "pyproject.toml",
  "pom.xml",
  "build.gradle",
  "tsconfig.json",
  "requirements.txt",
];

function looksLikeProject(dir: string): boolean {
  for (const m of PROJECT_MARKERS) {
    if (existsSync(join(dir, m))) return true;
  }
  return false;
}

function findProjectDir(dir: string): string {
  let cur = dir;
  for (let i = 0; i < 20; i++) {
    if (looksLikeProject(cur)) return cur;
    const parent = join(cur, "..");
    const resolvedParent = require("node:path").resolve(parent);
    if (resolvedParent === cur) break;
    cur = resolvedParent;
  }
  return dir;
}

function resolveHookProjectDir(input: string): string {
  if (input) {
    try {
      const req = JSON.parse(input) as { workspacePaths?: string[]; cwd?: string };
      if (req.workspacePaths?.[0]) return findProjectDir(req.workspacePaths[0]!);
      if (req.cwd) return findProjectDir(req.cwd);
    } catch {}
  }
  try {
    return findProjectDir(process.cwd());
  } catch {
    return "";
  }
}

function resolveCodegraphBin(): string {
  // Try direct which
  try {
    const { execSync } = require("node:child_process");
    const whichCmd = process.platform === "win32" ? "where" : "which";
    const out = execSync(`${whichCmd} codegraph`, { encoding: "utf-8", timeout: 3000 })
      .trim()
      .split("\n")[0];
    if (out) return out.trim();
  } catch {}
  return "";
}

export function runCodegraphIndexHook(): number {
  let input = "";
  try {
    // Read stdin if piped
    if (!process.stdin.isTTY) {
      input = readFileSync(0, "utf-8");
    }
  } catch {}

  const dir = resolveHookProjectDir(input);
  if (!dir) return 0;

  // Only index if looks like project (for auto mode silent fail)
  if (!looksLikeProject(dir) && !existsSync(join(dir, ".codegraph"))) {
    // Still try findProjectDir result might be parent; check again
    const proj = findProjectDir(dir);
    if (!looksLikeProject(proj)) return 0;
  }

  const bin = resolveCodegraphBin();
  if (!bin) return 0;

  try {
    if (existsSync(join(dir, ".codegraph"))) {
      const child = spawn(bin, ["sync"], { cwd: dir, detached: true, stdio: "ignore" });
      child.unref();
    } else {
      const child = spawn(bin, ["init"], { cwd: dir, detached: true, stdio: "ignore" });
      child.unref();
    }
  } catch {}

  return 0;
}
