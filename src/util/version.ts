import * as semver from "semver";

/** Build-time version from package.json. */
export function toksaveVersion(): string {
  return "0.3.0"; // Updated at build time
}

/** Compare two semver strings. Returns -1, 0, or 1. */
export function semverCmp(a: string, b: string): number {
  const va = semver.coerce(a.replace(/^v/, ""));
  const vb = semver.coerce(b.replace(/^v/, ""));
  if (!va || !vb) return 0;
  return semver.compare(va, vb);
}

/** True if local >= latest. */
export function isUpToDate(local: string, latest: string): boolean {
  return semverCmp(local, latest) >= 0;
}
