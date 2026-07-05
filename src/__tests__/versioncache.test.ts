import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  bustCache,
  getCachedLatest,
  getStaleFallback,
  setCachedLatest,
} from "../util/versioncache.js";

let tmp = "";
let oldCacheDir: string | undefined;

beforeEach(() => {
  oldCacheDir = process.env.TOKSAVE_CACHE_DIR;
  tmp = mkdtempSync(join(tmpdir(), "toksave-versioncache-test-"));
  process.env.TOKSAVE_CACHE_DIR = tmp;
});

afterEach(() => {
  if (oldCacheDir === undefined) delete process.env.TOKSAVE_CACHE_DIR;
  else process.env.TOKSAVE_CACHE_DIR = oldCacheDir;
  rmSync(tmp, { recursive: true, force: true });
});

describe("version cache", () => {
  test("getCachedLatest returns miss when no cache exists", () => {
    const result = getCachedLatest("rtk");
    expect(result.hit).toBe(false);
    expect(result.version).toBeNull();
  });

  test("setCachedLatest writes and getCachedLatest reads back", () => {
    setCachedLatest("rtk", "0.43.0");
    const result = getCachedLatest("rtk");
    expect(result.hit).toBe(true);
    expect(result.version).toBe("0.43.0");
  });

  test("cache merges multiple tools", () => {
    setCachedLatest("rtk", "0.43.0");
    setCachedLatest("codegraph", "1.1.6");
    expect(getCachedLatest("rtk").version).toBe("0.43.0");
    expect(getCachedLatest("codegraph").version).toBe("1.1.6");
  });

  test("stale cache returns miss on getCachedLatest but value on getStaleFallback", () => {
    // Write cache with old timestamp
    const cachePath = join(tmp, "versions.json");
    const staleTs = Date.now() - 7 * 60 * 60 * 1000; // 7 hours ago
    writeFileSync(cachePath, JSON.stringify({ ts: staleTs, map: { rtk: "0.42.0" } }), "utf-8");

    const cached = getCachedLatest("rtk");
    expect(cached.hit).toBe(false);

    const fallback = getStaleFallback("rtk");
    expect(fallback).toBe("0.42.0");
  });

  test("fresh cache (under 6h) returns hit", () => {
    const cachePath = join(tmp, "versions.json");
    const freshTs = Date.now() - 5 * 60 * 60 * 1000; // 5 hours ago
    writeFileSync(cachePath, JSON.stringify({ ts: freshTs, map: { rtk: "0.43.0" } }), "utf-8");

    const cached = getCachedLatest("rtk");
    expect(cached.hit).toBe(true);
    expect(cached.version).toBe("0.43.0");
  });

  test("bustCache removes cache file", () => {
    setCachedLatest("rtk", "0.43.0");
    expect(getCachedLatest("rtk").hit).toBe(true);

    bustCache();
    expect(getCachedLatest("rtk").hit).toBe(false);
  });

  test("corrupted cache file returns miss without crash", () => {
    const cachePath = join(tmp, "versions.json");
    writeFileSync(cachePath, "not json", "utf-8");

    const cached = getCachedLatest("rtk");
    expect(cached.hit).toBe(false);

    const fallback = getStaleFallback("rtk");
    expect(fallback).toBeNull();
  });
});
