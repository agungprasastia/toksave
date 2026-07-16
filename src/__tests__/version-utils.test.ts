import { describe, expect, it } from "bun:test";
import { countOutdated, semverCompare, semverGte } from "../util/version.js";

describe("semverCompare", () => {
  it("compares correctly", () => {
    expect(semverCompare("1.0.0", "1.1.0")).toBe(-1);
    expect(semverCompare("1.1.0", "1.0.0")).toBe(1);
    expect(semverCompare("1.0.0", "1.0.0")).toBe(0);
  });

  it("handles v prefix", () => {
    expect(semverCompare("v1.0.0", "1.0.0")).toBe(0);
    expect(semverCompare("1.0.0", "v1.1.0")).toBe(-1);
  });
});

describe("semverGte", () => {
  it("returns true when >= ", () => {
    expect(semverGte("1.1.0", "1.0.0")).toBe(true);
    expect(semverGte("1.0.0", "1.0.0")).toBe(true);
    expect(semverGte("1.0.0", "1.1.0")).toBe(false);
  });
});

describe("countOutdated", () => {
  it("counts tools where installed < latest", () => {
    const v = {
      rtk: { installed: "0.42.0", latest: "0.43.0", present: true },
      caveman: { installed: "1.0.0", latest: "1.0.0", present: true },
      codegraph: { installed: null, latest: "1.1.0", present: false },
    };
    expect(countOutdated(v)).toBe(1);
  });

  it("returns 0 when all up-to-date", () => {
    const v = {
      rtk: { installed: "0.43.0", latest: "0.43.0", present: true },
    };
    expect(countOutdated(v)).toBe(0);
  });

  it("returns 0 when empty", () => {
    expect(countOutdated({})).toBe(0);
  });
});
