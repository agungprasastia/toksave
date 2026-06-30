import { spawnSync } from "child_process";

export interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

/** Run a command and capture output. */
export function run(cmd: string, args: string[]): RunResult {
  try {
    const result = spawnSync(cmd, args, {
      encoding: "utf-8",
      timeout: 120_000,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return {
      code: result.status ?? -1,
      stdout: (result.stdout ?? "").trim(),
      stderr: (result.stderr ?? "").trim(),
    };
  } catch {
    return { code: -1, stdout: "", stderr: `Failed to execute: ${cmd}` };
  }
}

/** Run a command and return true if it succeeded. */
export function runOk(cmd: string, args: string[]): boolean {
  return run(cmd, args).code === 0;
}

/** Run a command and return stdout if successful. */
export function runStdout(cmd: string, args: string[]): string | null {
  const r = run(cmd, args);
  return r.code === 0 ? r.stdout : null;
}

/** Get the npm command name (npm or npm.cmd on Windows). */
export function npmCmd(): string {
  return process.platform === "win32" ? "npm.cmd" : "npm";
}

/** Get the npx command name. */
export function npxCmd(): string {
  return process.platform === "win32" ? "npx.cmd" : "npx";
}
