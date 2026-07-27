import { spawn } from "node:child_process";
import { closeSync, existsSync, openSync, readSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join, resolve as resolvePath } from "node:path";
import { findBinary, findBinaryIn, resolveNode } from "../util/detect.js";
import { resolveMcpExecutable } from "../util/mcpspawn.js";

export function isNodeShebangScript(filePath: string): boolean {
  let fd: number | null = null;
  try {
    const buf = Buffer.alloc(64);
    fd = openSync(filePath, "r");
    readSync(fd, buf, 0, 64, 0);
    const head = buf.toString("utf8");
    return head.startsWith("#!/usr/bin/env node") || /^#!.*\bnode(?:\.exe)?\b/m.test(head);
  } catch {
    return false;
  } finally {
    if (fd !== null) {
      try {
        closeSync(fd);
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
    const resolved = resolvePath(parent);
    if (resolved === cur) break;
    cur = resolved;
  }
  return cwd;
}

/** Expand PATH so GUI-launched agents (minimal env) still find node/npm tools. */
export function ensureToolPath(): string {
  const parts = (process.env.PATH ?? "")
    .split(process.platform === "win32" ? ";" : ":")
    .filter(Boolean);
  const extras: string[] = [];
  const home = homedir();
  extras.push(join(home, ".local", "bin"), join(home, ".bun", "bin"), join(home, ".cargo", "bin"));
  if (process.platform !== "win32") {
    extras.push("/usr/local/bin", "/opt/homebrew/bin");
  }
  const miseNodeRoot = join(home, ".local", "share", "mise", "installs", "node");
  if (existsSync(miseNodeRoot)) {
    try {
      const { readdirSync } = require("node:fs") as typeof import("node:fs");
      const versions = readdirSync(miseNodeRoot)
        .map((v) => join(miseNodeRoot, v, "bin"))
        .filter((p) => existsSync(join(p, process.platform === "win32" ? "node.exe" : "node")))
        .sort()
        .reverse();
      extras.push(...versions);
    } catch {}
  }
  const miseShims = join(home, ".local", "share", "mise", "shims");
  if (existsSync(miseShims)) extras.push(miseShims);

  for (const dir of extras) {
    if (dir && !parts.includes(dir)) parts.unshift(dir);
  }
  const next = parts.join(process.platform === "win32" ? ";" : ":");
  process.env.PATH = next;
  return next;
}

function preIndexIfNeeded(cwd: string): void {
  // Fire-and-forget codegraph sync/init if .codegraph exists — only if binary present
  try {
    const codegraphBin = resolveMcpExecutable("codegraph") ?? findBinary("codegraph") ?? "";
    if (!codegraphBin) return;
    if (!existsSync(join(cwd, ".codegraph"))) return;
    const child = spawn(codegraphBin, ["sync"], { cwd, detached: true, stdio: "ignore" });
    child.unref();
  } catch {}
}

export function runMcp(): Promise<number> {
  return new Promise((resolve) => {
    ensureToolPath();

    let args = process.argv.slice(3);
    if (args.length === 0) {
      console.error("Usage: toksave runmcp [--agent <id>] <script_path|tool> [args...]");
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
      console.error("Usage: toksave runmcp [--agent <id>] <script_path|tool> [args...]");
      return resolve(1);
    }

    const requested = args[0];
    if (!requested) {
      console.error("Error: No executable specified for MCP server.");
      return resolve(1);
    }

    // Agent configs pass bare tool names (codegraph, context-mode). Resolve them.
    let exe = resolveMcpExecutable(requested) ?? requested;
    let cmdArgs = args.slice(1);

    if (!isAbsolute(exe) && !exe.includes("/") && !exe.includes("\\")) {
      const found = findBinary(exe) ?? findBinaryIn(exe, []);
      if (found) exe = found;
    }

    if (!existsSync(exe) && !findBinary(exe)) {
      console.error(`Error: MCP executable not found: ${requested}`);
      console.error(
        "Install the tool (e.g. npm i -g @colbymchenry/codegraph context-mode) and ensure PATH includes its bin dir.",
      );
      return resolve(1);
    }

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

    child.on("error", (err) => {
      console.error(`Error: failed to start MCP server: ${err.message}`);
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
