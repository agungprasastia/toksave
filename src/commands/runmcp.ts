import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
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
      } catch {
        /* ignore */
      }
    }
  }
}

export function runMcp(): Promise<number> {
  return new Promise((resolve) => {
    const args = process.argv.slice(3); // process.argv = ["node", "toksave", "runmcp", "path/to/script", ...]
    if (args.length === 0) {
      console.error("Usage: toksave runmcp <script_path> [args...]");
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
