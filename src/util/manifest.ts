import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
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

/** Read the manifest, returning empty if not found. */
export function readManifest(): Manifest {
  const p = manifestPath();
  if (!existsSync(p)) return { entries: [] };
  try {
    return JSON.parse(readFileSync(p, "utf-8"));
  } catch {
    return { entries: [] };
  }
}

/** Write the manifest to disk. */
export function writeManifest(m: Manifest): void {
  const p = manifestPath();
  const dir = dirname(p);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  writeFileSync(p, JSON.stringify(m, null, 2), "utf-8");
}

/** Record that a tool was wired into an agent. */
export function recordWire(agent: string, tool: string, version?: string): void {
  const m = readManifest();
  // Remove existing entry for this agent+tool combo
  m.entries = m.entries.filter((e) => !(e.agent === agent && e.tool === tool));
  m.entries.push({
    agent,
    tool,
    wiredAt: new Date().toISOString(),
    version,
  });
  writeManifest(m);
}

/** Remove a wiring record. */
export function removeWire(agent: string, tool: string): void {
  const m = readManifest();
  m.entries = m.entries.filter((e) => !(e.agent === agent && e.tool === tool));
  writeManifest(m);
}

/** Check if a tool was wired by toksave. */
export function wasWiredByUs(agent: string, tool: string): boolean {
  const m = readManifest();
  return m.entries.some((e) => e.agent === agent && e.tool === tool);
}

/** Clear the entire manifest. */
export function clearManifest(): void {
  const p = manifestPath();
  try {
    writeFileSync(p, JSON.stringify({ entries: [] }, null, 2));
  } catch {
    /* ignore */
  }
}
