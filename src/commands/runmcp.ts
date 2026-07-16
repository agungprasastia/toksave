import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { resolveNode } from "../util/detect.js";

export function isNodeShebangScript(filePath: string): boolean {
  let fd: number | null = null;
  try {
    const buf = Buffer.alloc(32);
    fd = require("node:fs").openSync(filePath, "r");
    require("node:fs").readSync(fd, buf, 0, 32, 0);
    return buf.toString().startsWith("#!/usr/bin/env node");
  } catch {
    return false;
  } finally {
    if (fd !== null) {
      try {
        require("node:fs").closeSync(fd);
      } catch {}
    }
  }
}

function parseAgentFlag(args: string[]): { agent: string; rest: string[] } {
  const rest: string[] = [];
  let agent = "";
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--agent" && i + 1 < args.length) {
      agent = args[i + 1]!;
      i++;
    } else {
      rest.push(args[i]!);
    }
  }
  return { agent, rest };
}

function resolveHookProjectDirFromInput(_input: string, cwd: string): string {
  // For codegraph pre-index we need project dir — simplest: cwd with marker walk
  const markers = [".git", "package.json", "go.mod", "Cargo.toml", "pyproject.toml"];
  let cur = cwd;
  for (let i = 0; i < 20; i++) {
    for (const m of markers) {
      if (existsSync(join(cur, m))) return cur;
    }
    const parent = join(cur, "..");
    const resolved = require("node:path").resolve(parent);
    if (resolved === cur) break;
    cur = resolved;
  }
  return cwd;
}

function preIndexIfNeeded(cwd: string): void {
  // Fire-and-forget codegraph sync/init if .codegraph exists or markers present — like tokless RunIndex auto
  // Only if codegraph binary present
  try {
    const codegraphBin = (() => {
      try {
        const { execSync } = require("node:child_process");
        const whichCmd = process.platform === "win32" ? "where" : "which";
        const out = execSync(`${whichCmd} codegraph`, { encoding: "utf-8", timeout: 2000 })
          .trim()
          .split("\n")[0];
        return out?.trim() ?? "";
      } catch {
        return "";
      }
    })();
    if (!codegraphBin) return;
    if (!existsSync(join(cwd, ".codegraph"))) return;
    // Spawn background sync
    const child = spawn(codegraphBin, ["sync"], { cwd, detached: true, stdio: "ignore" });
    child.unref();
  } catch {}
}

export function runMcp(): Promise<number> {
  return new Promise((resolve) => {
    let args = process.argv.slice(3);
    if (args.length === 0) {
      console.error("Usage: toksave runmcp <script_path> [args...]");
      return resolve(1);
    }

    // Parse --agent flag for pre-index (tokless RunIndex auto)
    const parsed = parseAgentFlag(args);
    const _agent = parsed.agent;
    args = parsed.rest;

    // Pre-index: auto build per-project codegraph index if .codegraph exists
    try {
      const cwd = process.cwd();
      preIndexIfNeeded(resolveHookProjectDirFromInput("", cwd));
    } catch {}

    if (args.length === 0) {
      console.error("Usage: toksave runmcp [--agent <id>] <script_path> [args...]");
      return resolve(1);
    }

    let exe = args[0];
    if (!exe) {
      console.error("Error: No executable specified for MCP server.");
      return resolve(1);
    }
    let cmdArgs = args.slice(1);

    if (existsSync(exe) && isNodeShebangScript(exe)) {
      cmdArgs = [exe, ...cmdArgs];
      const systemNode = resolveNode();
      if (!systemNode) {
        console.error("Error: Could not find 'node' executable on PATH to run MCP server.");
        return resolve(1);
      }
      exe = systemNode;
    }

    const child = spawn(exe, cmdArgs, {
      stdio: ["inherit", "pipe", "inherit"],
      env: process.env,
    });

    child.on("error", () => {
      resolve(1);
    });

    child.on("exit", (code) => {
      resolve(code ?? 1);
    });

    // Proxy stdout directly to process.stdout
    if (child.stdout) {
      child.stdout.pipe(process.stdout);
    }
  });
}
