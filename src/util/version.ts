import * as semver from "semver";
import type { ToolId } from "../registry.js";
import { ALL_TOOLS, toolInstalledVersion, toolLatestVersion } from "../registry.js";
import { bustCache } from "./versioncache.js";

// ─── Version info type ──────────────────────────────────────

export interface VersionInfo {
  installed: string | null;
  latest: string | null;
  present: boolean;
}

// ─── TokSave version ────────────────────────────────────────

/** Build-time version from package.json. */
export function toksaveVersion(): string {
  return "0.8.1";
}

/** User-Agent string with version for HTTP requests. */
export function userAgent(): string {
  return `toksave/${toksaveVersion()}`;
}

// ─── Semver helpers ──────────────────────────────────────────

/** Compare two semver strings. Returns -1, 0, or 1. */
export function semverCmp(a: string, b: string): number {
  const va = semver.coerce(a.replace(/^v/, ""));
  const vb = semver.coerce(b.replace(/^v/, ""));
  if (!va || !vb) return -1;
  return semver.compare(va, vb);
}

/** True if local >= latest. */
export function isUpToDate(local: string, latest: string): boolean {
  return semverCmp(local, latest) >= 0;
}

export function semverCompare(a: string, b: string): number {
  return semverCmp(a, b);
}

export function semverGte(a: string, b: string): boolean {
  return semverCmp(a, b) >= 0;
}

export function semverLt(a: string, b: string): boolean {
  return semverCmp(a, b) < 0;
}

// ─── Version gathering ──────────────────────────────────────

const trackableTools = () => ALL_TOOLS.filter((t) => !t.notTrackable);

export async function gatherVersions(): Promise<Record<string, VersionInfo>> {
  const tools = trackableTools();
  const latestResults = await Promise.all(
    tools.map(async (t) => {
      const v = await toolLatestVersion(t.id);
      return [t.id, v] as const;
    }),
  );
  const latestMap: Record<string, string | null> = {};
  for (const [id, v] of latestResults) {
    latestMap[id] = v;
  }

  const out: Record<string, VersionInfo> = {};
  for (const t of tools) {
    const installed = toolInstalledVersion(t.id);
    out[t.id] = {
      installed,
      latest: latestMap[t.id] ?? null,
      present: installed !== null,
    };
  }
  return out;
}

export async function gatherVersionsForce(): Promise<Record<string, VersionInfo>> {
  bustCache();
  return gatherVersions();
}

export function countOutdated(versions: Record<string, VersionInfo>): number {
  let n = 0;
  for (const v of Object.values(versions)) {
    if (v.installed && v.latest && semverCmp(v.installed, v.latest) < 0) {
      n++;
    }
  }
  return n;
}

export function installedVersionFor(id: ToolId): string | null {
  return toolInstalledVersion(id);
}

export async function latestVersionFor(id: ToolId): Promise<string | null> {
  return toolLatestVersion(id);
}

export function bustVersionCache(): void {
  bustCache();
}
