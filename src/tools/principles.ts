import type { RunOpts } from "../registry.js";
import type { HealthStatus, RepairResult } from "../util/health.js";

export async function install(_opts: RunOpts): Promise<boolean> {
  // Instruction-only tool — no install needed
  return true;
}

export function installedVersion(): string | null {
  return null;
}

export async function latestVersion(): Promise<string | null> {
  return null;
}

export function healthCheck(): HealthStatus {
  return { healthy: true, version: null, issues: [] };
}

export async function repair(_opts: RunOpts): Promise<RepairResult> {
  const h = healthCheck();
  return { success: true, message: "Principles is instruction-only", healthAfterRepair: h };
}
