import { describe, expect, test } from "bun:test";
import { CAVEMAN_SKILL_MD } from "../content/caveman-skill.js";
import { CTX_RULES_BLOCK, hasCtxRules, removeCtxRules } from "../content/ctx-rules.js";
import { isUpToDate, semverCmp, toksaveVersion } from "../util/version.js";

describe("Version utils", () => {
  test("toksaveVersion returns semver", () => {
    expect(toksaveVersion()).toMatch(/^\d+\.\d+\.\d+/);
  });

  test("semverCmp equal", () => {
    expect(semverCmp("1.0.0", "1.0.0")).toBe(0);
  });

  test("semverCmp less", () => {
    expect(semverCmp("1.0.0", "2.0.0")).toBe(-1);
  });

  test("semverCmp greater", () => {
    expect(semverCmp("2.0.0", "1.0.0")).toBe(1);
  });

  test("isUpToDate true when equal", () => {
    expect(isUpToDate("1.0.0", "1.0.0")).toBe(true);
  });

  test("isUpToDate true when ahead", () => {
    expect(isUpToDate("2.0.0", "1.0.0")).toBe(true);
  });

  test("isUpToDate false when behind", () => {
    expect(isUpToDate("1.0.0", "2.0.0")).toBe(false);
  });

  test("handles v prefix", () => {
    expect(semverCmp("v1.0.0", "1.0.0")).toBe(0);
  });
});

describe("Content: Context-Mode rules", () => {
  test("CTX_RULES_BLOCK contains marker", () => {
    expect(CTX_RULES_BLOCK).toContain("CONTEXT-MODE_START");
    expect(CTX_RULES_BLOCK).toContain("CONTEXT-MODE_END");
  });

  test("hasCtxRules detects marker", () => {
    expect(hasCtxRules("some text CONTEXT-MODE_START more")).toBe(true);
    expect(hasCtxRules("no marker here")).toBe(false);
  });

  test("removeCtxRules strips block", () => {
    const before = "header\n<!-- CONTEXT-MODE_START -->\nrules\n<!-- CONTEXT-MODE_END -->\nfooter";
    const after = removeCtxRules(before);
    expect(after).not.toContain("CONTEXT-MODE_START");
    expect(after).toContain("header");
    expect(after).toContain("footer");
  });
});

describe("Content: Caveman SKILL.md", () => {
  test("contains frontmatter", () => {
    expect(CAVEMAN_SKILL_MD).toContain("name: caveman");
    expect(CAVEMAN_SKILL_MD).toContain("description:");
  });

  test("contains all levels", () => {
    expect(CAVEMAN_SKILL_MD).toContain("### lite");
    expect(CAVEMAN_SKILL_MD).toContain("### full");
    expect(CAVEMAN_SKILL_MD).toContain("### ultra");
  });

  test("contains auto-clarity", () => {
    expect(CAVEMAN_SKILL_MD).toContain("Auto-Clarity");
  });

  test("contains boundaries", () => {
    expect(CAVEMAN_SKILL_MD).toContain("Boundaries");
  });
});
