import { npmCmd, run, runStdout } from "./exec.js";

/** Install an npm package globally. */
export function installGlobal(pkg: string): boolean {
  const r = run(npmCmd(), ["install", "-g", pkg]);
  if (r.code === 0) return true;
  throw new Error(`npm install -g ${pkg} failed: ${r.stderr}`);
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
      headers: { "User-Agent": "toksave" },
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
  const major = Number.parseInt(v.split(".")[0], 10);
  return !Number.isNaN(major) && major >= minMajor;
}
