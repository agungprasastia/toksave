import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { cacheDir } from "./paths.js";

export interface ManifestEntry {
  agent: string;
  tool: string;
  wiredAt: string;
  version?: string;
}

export interface Manifest {
  entries: ManifestEntry[];
}

function manifestPath(): string {
  return join(cacheDir(), "manifest.json");
}

function lockPath(): string {
  return `${manifestPath()}.lock`;
}

function readManifestFile(): Manifest {
  const p = manifestPath();
  if (!existsSync(p)) return { entries: [] };
  try {
    return JSON.parse(readFileSync(p, "utf-8"));
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    console.warn(`Warning: Failed to parse manifest at ${p}: ${msg}`);
    return { entries: [] };
  }
}

function sleepSync(ms: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function withManifestLock<T>(fn: () => T): T {
  const lock = lockPath();
  const dir = dirname(lock);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

  const started = Date.now();
  while (true) {
    try {
      mkdirSync(lock);
      break;
    } catch {
      try {
        const ageMs = Date.now() - statSync(lock).mtimeMs;
        if (ageMs > 30_000) rmSync(lock, { recursive: true, force: true });
      } catch {
        /* retry */
      }
      if (Date.now() - started > 5_000) {
        throw new Error(`Timed out waiting for manifest lock: ${lock}`);
      }
      sleepSync(50);
    }
  }

  try {
    return fn();
  } finally {
    try {
      rmSync(lock, { recursive: true, force: true });
    } catch {
      /* ignore */
    }
  }
}

/** Read the manifest, returning empty if not found. */
export function readManifest(): Manifest {
  return readManifestFile();
}

/** Write the manifest to disk. */
export function writeManifest(m: Manifest): void {
  const p = manifestPath();
  const dir = dirname(p);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  const tmp = `${p}.${process.pid}.tmp`;
  writeFileSync(tmp, JSON.stringify(m, null, 2), "utf-8");
  renameSync(tmp, p);
}

/** Record that a tool was wired into an agent. */
export function recordWire(agent: string, tool: string, version?: string): void {
  withManifestLock(() => {
    const m = readManifestFile();
    // Remove existing entry for this agent+tool combo
    m.entries = m.entries.filter((e) => !(e.agent === agent && e.tool === tool));
    m.entries.push({
      agent,
      tool,
      wiredAt: new Date().toISOString(),
      version,
    });
    writeManifest(m);
  });
}

/** Remove a wiring record. */
export function removeWire(agent: string, tool: string): void {
  withManifestLock(() => {
    const m = readManifestFile();
    m.entries = m.entries.filter((e) => !(e.agent === agent && e.tool === tool));
    writeManifest(m);
  });
}

/** Check if a tool was wired by toksave. */
export function wasWiredByUs(agent: string, tool: string): boolean {
  const m = readManifest();
  return m.entries.some((e) => e.agent === agent && e.tool === tool);
}

/** Clear the entire manifest. */
export function clearManifest(): void {
  try {
    withManifestLock(() => writeManifest({ entries: [] }));
  } catch {
    /* ignore */
  }
}
