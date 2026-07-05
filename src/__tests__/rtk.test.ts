import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import * as rtk from "../tools/rtk.js";
import * as detect from "../util/detect.js";
import * as exec from "../util/exec.js";
import { ensureDir, localBin } from "../util/paths.js";

let tmp = "";
let oldHome: string | undefined;
let oldLocalAppData: string | undefined;
let runStdoutSpy: any;
let runSpy: any;

beforeEach(() => {
  oldHome = process.env.HOME;
  oldLocalAppData = process.env.LOCALAPPDATA;

  tmp = mkdtempSync(join(tmpdir(), "toksave-rtk-test-"));
  process.env.HOME = join(tmp, "home");
  process.env.LOCALAPPDATA = join(tmp, "AppData", "Local");

  // Mock exec methods using spyOn to avoid ESM caching issues on CI
  runStdoutSpy = spyOn(exec, "runStdout").mockReturnValue("0.1.0");
  runSpy = spyOn(exec, "run").mockReturnValue({ code: 0, stdout: "", stderr: "" });
});

afterEach(() => {
  restoreEnv("HOME", oldHome);
  restoreEnv("LOCALAPPDATA", oldLocalAppData);
  rmSync(tmp, { recursive: true, force: true });

  if (runStdoutSpy) runStdoutSpy.mockRestore();
  if (runSpy) runSpy.mockRestore();
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
    // Spy on isOnPath to return false only for "rtk" during this test
    const spy = spyOn(detect, "isOnPath").mockImplementation((name) => {
      if (name === "rtk") return false;
      return true; // pretend other things are on path if needed
    });

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

    spy.mockRestore();
  });

  test("isInstalledButUnreachable is false when not installed at all", () => {
    const spy = spyOn(detect, "isOnPath").mockReturnValue(false);
    expect(rtk.isInstalledButUnreachable()).toBe(false);
    spy.mockRestore();
  });
});
