import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import * as rtk from "../tools/rtk.js";
import * as exec from "../util/exec.js";
import { ensureDir, localBin } from "../util/paths.js";

let tmp = "";
let oldHome: string | undefined;
let oldLocalAppData: string | undefined;
let oldPath: string | undefined;
let runStdoutSpy: ReturnType<typeof spyOn>;
let runSpy: ReturnType<typeof spyOn>;

beforeEach(() => {
  oldHome = process.env.HOME;
  oldLocalAppData = process.env.LOCALAPPDATA;
  oldPath = process.env.PATH;

  runStdoutSpy = spyOn(exec, "runStdout").mockReturnValue("0.1.0");
  runSpy = spyOn(exec, "run").mockReturnValue({ code: 0, stdout: "", stderr: "" });

  tmp = mkdtempSync(join(tmpdir(), "toksave-rtk-test-"));
  process.env.HOME = join(tmp, "home");
  process.env.LOCALAPPDATA = join(tmp, "AppData", "Local");

  // Clean PATH of any real rtk binaries to simulate clean environment
  const pathSeparator = process.platform === "win32" ? ";" : ":";
  const pathsList = (process.env.PATH || "").split(pathSeparator);
  const normalized = pathsList.filter((p) => {
    const lower = p.toLowerCase();
    return (
      !lower.includes("toksave") && !lower.endsWith(".local/bin") && !lower.endsWith(".local\\bin")
    );
  });
  process.env.PATH = normalized.join(pathSeparator);
});

afterEach(() => {
  runStdoutSpy.mockRestore();
  runSpy.mockRestore();
  restoreEnv("HOME", oldHome);
  restoreEnv("LOCALAPPDATA", oldLocalAppData);
  restoreEnv("PATH", oldPath);
  rmSync(tmp, { recursive: true, force: true });
});

function restoreEnv(key: string, value: string | undefined): void {
  if (value === undefined) {
    delete process.env[key];
    return;
  }
  process.env[key] = value;
}

describe("RTK detection", () => {
  test("isInstalledButUnreachable is true when binary in localBin but not in PATH", () => {
    const binDir = localBin();

    ensureDir(binDir);
    const exeName = process.platform === "win32" ? "rtk.exe" : "rtk";
    writeFileSync(join(binDir, exeName), "fake binary", "utf-8");

    expect(rtk.isInstalledButUnreachable()).toBe(true);

    const health = rtk.healthCheck();
    expect(health.healthy).toBe(false);
    expect(health.issues.length).toBeGreaterThan(0);
    expect(health.issues[0]?.severity).toBe("error");
    expect(health.issues[0]?.message).toContain("not on your PATH");
    expect(health.issues[0]?.remediation).toContain("Enforcement hooks will fail");
  });

  test("isInstalledButUnreachable is false when not installed at all", () => {
    expect(rtk.isInstalledButUnreachable()).toBe(false);
  });
});
