import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import * as caveman from "../tools/caveman.js";
import * as paths from "../util/paths.js";

let tmp = "";
let oldHome: string | undefined;
let oldLocalAppData: string | undefined;

beforeEach(() => {
  oldHome = process.env.HOME;
  oldLocalAppData = process.env.LOCALAPPDATA;

  tmp = mkdtempSync(join(tmpdir(), "toksave-caveman-test-"));
  process.env.HOME = join(tmp, "home");
  process.env.LOCALAPPDATA = join(tmp, "AppData", "Local");
});

afterEach(() => {
  if (oldHome === undefined) delete process.env.HOME;
  else process.env.HOME = oldHome;
  if (oldLocalAppData === undefined) delete process.env.LOCALAPPDATA;
  else process.env.LOCALAPPDATA = oldLocalAppData;
  rmSync(tmp, { recursive: true, force: true });
});

describe("Caveman healthCheck", () => {
  test("not installed returns unhealthy with error", () => {
    const health = caveman.healthCheck();
    expect(health.healthy).toBe(false);
    expect(health.version).toBeNull();
    expect(health.issues[0]?.severity).toBe("error");
    expect(health.issues[0]?.message).toContain("not found");
  });

  test("installed returns healthy with no issues", () => {
    // Write a SKILL.md so installedVersion() finds it
    const skillDir = join(paths.claudePaths().skillsDir, "caveman");
    mkdirSync(skillDir, { recursive: true });
    writeFileSync(join(skillDir, "SKILL.md"), "---\nname: caveman\n---\n# Caveman", "utf-8");

    const health = caveman.healthCheck();
    expect(health.healthy).toBe(true);
    expect(health.version).not.toBeNull();
    expect(health.issues).toHaveLength(0);
  });
});

describe("Caveman latestVersion", () => {
  test("returns null on network failure without crashing", async () => {
    const fetchSpy = spyOn(globalThis, "fetch").mockRejectedValue(new Error("offline"));
    const result = await caveman.latestVersion();
    expect(result).toBeNull();
    fetchSpy.mockRestore();
  });
});
