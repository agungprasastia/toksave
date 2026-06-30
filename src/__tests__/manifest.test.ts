import { describe, expect, test } from "bun:test";
import { readManifest, recordWire, removeWire, wasWiredByUs } from "../util/manifest.js";

// Override cache dir for testing
const _originalEnv = process.env.HOME;

describe("Manifest", () => {
  test("readManifest returns empty for fresh state", () => {
    const m = readManifest();
    expect(m.entries).toBeDefined();
    expect(Array.isArray(m.entries)).toBe(true);
  });

  test("recordWire + wasWiredByUs", () => {
    recordWire("claude", "rtk", "0.43.0");
    expect(wasWiredByUs("claude", "rtk")).toBe(true);
    expect(wasWiredByUs("claude", "caveman")).toBe(false);
  });

  test("recordWire replaces existing entry", () => {
    recordWire("claude", "rtk", "0.42.0");
    recordWire("claude", "rtk", "0.43.0");
    const m = readManifest();
    const matches = m.entries.filter((e) => e.agent === "claude" && e.tool === "rtk");
    expect(matches).toHaveLength(1);
    expect(matches[0].version).toBe("0.43.0");
  });

  test("removeWire clears entry", () => {
    recordWire("opencode", "codegraph");
    removeWire("opencode", "codegraph");
    expect(wasWiredByUs("opencode", "codegraph")).toBe(false);
  });
});
