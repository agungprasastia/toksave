import { InstallError } from "./errors.js";
import { npmCmd, run, runStdout } from "./exec.js";
import { userAgent } from "./version.js";

/** Install an npm package globally. */
export function installGlobal(pkg: string): boolean {
  const r = run(npmCmd(), ["install", "-g", pkg]);
  if (r.code === 0) return true;
  throw new InstallError(pkg, {
    message: `Failed to install npm package globally: ${pkg}`,
    cause: r.stderr,
    remediation:
      "Check npm configuration and network connectivity. Try running: npm config get prefix",
  });
}

/** Get installed version of a global npm package. */
export function installedVersion(pkg: string): string | null {
  const r = run(npmCmd(), ["list", "-g", pkg, "--depth=0", "--json"]);
  if (r.code !== 0) return null;
  try {
    const json = JSON.parse(r.stdout);
    return json?.dependencies?.[pkg]?.version ?? null;
  } catch {
    return null;
  }
}

/** Get latest version of an npm package from registry. */
export async function latestVersion(pkg: string): Promise<string | null> {
  try {
    const res = await fetch(`https://registry.npmjs.org/${pkg}/latest`, {
      headers: { "User-Agent": userAgent() },
      signal: AbortSignal.timeout(10000),
    });
    if (!res.ok) return null;
    const json = await res.json();
    return json.version || null;
  } catch {
    return null;
  }
}

/** Check if Node.js is installed and meets minimum version. */
export function checkNode(minMajor: number): boolean {
  const out = runStdout("node", ["--version"]);
  if (!out) return false;
  const v = out.trim().replace(/^v/, "");
  const firstPart = v.split(".")[0];
  if (!firstPart) return false;
  const major = Number.parseInt(firstPart, 10);
  return !Number.isNaN(major) && major >= minMajor;
}
