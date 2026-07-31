import * as semver from "semver";
import pkg from "../../package.json";

// ─── Version info type ──────────────────────────────────────

export interface VersionInfo {
  installed: string | null;
  latest: string | null;
  present: boolean;
}

// ─── TokSave version ────────────────────────────────────────

/** Build-time version from package.json. */
export function toksaveVersion(): string {
  return pkg.version;
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

export function countOutdated(versions: Record<string, VersionInfo>): number {
  let n = 0;
  for (const v of Object.values(versions)) {
    if (v.installed && v.latest && semverCmp(v.installed, v.latest) < 0) {
      n++;
    }
  }
  return n;
}
