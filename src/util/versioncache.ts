import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { cacheDir, ensureDir } from "./paths.js";

interface CacheShape {
  ts: number;
  map: Record<string, string>;
}

const CACHE_TTL_MS = 6 * 60 * 60 * 1000; // 6 hours

function cachePath(): string {
  return join(cacheDir(), "versions.json");
}

function loadCache(): { data: CacheShape | null; fresh: boolean } {
  const p = cachePath();
  if (!existsSync(p)) return { data: null, fresh: false };
  try {
    const obj: CacheShape = JSON.parse(readFileSync(p, "utf-8"));
    const fresh = Date.now() - obj.ts <= CACHE_TTL_MS;
    return { data: obj, fresh };
  } catch {
    return { data: null, fresh: false };
  }
}

function saveCache(map: Record<string, string>): void {
  const p = cachePath();
  ensureDir(join(p, ".."));
  writeFileSync(p, JSON.stringify({ ts: Date.now(), map } satisfies CacheShape, null, 2), "utf-8");
}

/**
 * Get cached latest version for a tool.
 * Returns { version, hit } where hit=true means cache was used.
 */
export function getCachedLatest(toolId: string): { version: string | null; hit: boolean } {
  const { data, fresh } = loadCache();
  if (data && fresh && data.map[toolId]) {
    return { version: data.map[toolId], hit: true };
  }
  return { version: null, hit: false };
}

/**
 * Get cached latest version, falling back to stale value if available.
 * Use when fetch fails — stale is better than nothing.
 */
export function getStaleFallback(toolId: string): string | null {
  const { data } = loadCache();
  return data?.map[toolId] ?? null;
}

/**
 * Save a fetched latest version to cache (merges with existing entries).
 */
export function setCachedLatest(toolId: string, version: string): void {
  const { data } = loadCache();
  const map = data?.map ?? {};
  map[toolId] = version;
  saveCache(map);
}

/** Remove the cache file. */
export function bustCache(): void {
  const p = cachePath();
  try {
    const { unlinkSync } = require("node:fs");
    unlinkSync(p);
  } catch {
    /* ignore */
  }
}
