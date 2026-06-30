import { spawn } from "node:child_process";
import { existsSync } from "node:fs";

function isNodeShebangScript(filePath: string): boolean {
  try {
    const buf = Buffer.alloc(32);
    const fd = require("node:fs").openSync(filePath, "r");
    require("node:fs").readSync(fd, buf, 0, 32, 0);
    require("node:fs").closeSync(fd);
    return buf.toString().startsWith("#!/usr/bin/env node");
  } catch {
    return false;
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
    let cmdArgs = args.slice(1);

    if (existsSync(exe) && isNodeShebangScript(exe)) {
      cmdArgs = [exe, ...cmdArgs];
      exe = process.execPath; // Use the current node binary
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
