import { spawnSync } from "node:child_process";
import { readJsonFile, writeJsonFile } from "../config/json.js";
import type { RunOpts } from "../registry.js";
import { isOnPath } from "../util/detect.js";
import type { HealthStatus, RepairResult } from "../util/health.js";
import * as paths from "../util/paths.js";
import { userAgent } from "../util/version.js";

const PONYTAIL_PKG = "@dietrichgebert/ponytail";

/** Install ponytail via npm global. */
export async function install(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) return true;
  if (process.env.TOKSAVE_TEST === "1") return true;

  if (!opts.upgrade) {
    const v = installedVersion();
    if (v) return true;
  }

  if (!isOnPath("npm")) return false;

  try {
    const res = spawnSync("npm", ["install", "-g", `${PONYTAIL_PKG}@latest`], {
      stdio: "pipe",
      timeout: 5 * 60 * 1000,
    });
    return res.status === 0;
  } catch {
    return false;
  }
}

export function installedVersion(): string | null {
  try {
    const npm = isOnPath("npm") ? "npm" : null;
    if (!npm) {
      // Check if plugin present in opencode config => consider installed
      if (ponytailPluginInstalled()) return "0.0.0";
      return null;
    }
    const res = spawnSync(npm, ["list", "-g", PONYTAIL_PKG, "--depth=0", "--json"], {
      encoding: "utf-8",
      timeout: 10000,
    });
    if (res.status !== 0) {
      if (ponytailPluginInstalled()) return "0.0.0";
      return null;
    }
    const json = JSON.parse(res.stdout);
    const deps = json.dependencies as Record<string, { version?: string }> | undefined;
    const entry = deps?.[PONYTAIL_PKG];
    return entry?.version ?? null;
  } catch {
    if (ponytailPluginInstalled()) return "0.0.0";
    return null;
  }
}

export async function latestVersion(): Promise<string | null> {
  try {
    const resp = await fetch(
      `https://registry.npmjs.org/${encodeURIComponent(PONYTAIL_PKG)}/latest`,
      {
        headers: { "User-Agent": userAgent() },
        signal: AbortSignal.timeout(10000),
      },
    );
    if (!resp.ok) return null;
    const json = (await resp.json()) as { version?: string };
    return json.version ?? null;
  } catch {
    return null;
  }
}

export function healthCheck(): HealthStatus {
  const v = installedVersion();
  if (!v) {
    return {
      healthy: false,
      version: null,
      issues: [
        {
          severity: "error",
          message: "Ponytail not installed",
          remediation: "Run: toksave --tools ponytail",
        },
      ],
    };
  }
  return { healthy: true, version: v, issues: [] };
}

export async function repair(opts: RunOpts): Promise<RepairResult> {
  const before = healthCheck();
  if (before.healthy) {
    return {
      success: true,
      message: "Ponytail already healthy",
      healthAfterRepair: before,
    };
  }
  const ok = await install({ ...opts, upgrade: true });
  const after = healthCheck();
  return {
    success: ok && after.healthy,
    message: ok ? "Ponytail repaired" : "Ponytail repair failed",
    healthAfterRepair: after,
  };
}

// ─── Opencode plugin helpers ─────────────────────────────────

export function registerOpencodePlugin(): void {
  const p = paths.opencodePaths();
  const cfg = (readJsonFile(p.config) as Record<string, unknown>) ?? {
    $schema: "https://opencode.ai/config.json",
  };
  const plugins = (cfg.plugin as unknown[]) ?? [];
  if (
    plugins.some((pl) => typeof pl === "string" && pl.toLowerCase() === PONYTAIL_PKG.toLowerCase())
  ) {
    return;
  }
  // Insert before context-mode if present
  let inserted = false;
  const out: unknown[] = [];
  for (const pl of plugins) {
    if (
      !inserted &&
      typeof pl === "string" &&
      (pl === "context-mode" || pl.startsWith("context-mode@"))
    ) {
      out.push(PONYTAIL_PKG);
      inserted = true;
    }
    out.push(pl);
  }
  if (!inserted) out.push(PONYTAIL_PKG);
  cfg.plugin = out;
  writeJsonFile(p.config, cfg);
}

export function unregisterOpencodePlugin(): void {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) as Record<string, unknown> | null;
  if (!cfg) return;
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return;
  const kept = plugins.filter(
    (pl) => !(typeof pl === "string" && pl.toLowerCase() === PONYTAIL_PKG.toLowerCase()),
  );
  cfg.plugin = kept;
  writeJsonFile(p.config, cfg);
}

export function ponytailPluginInstalled(): boolean {
  const p = paths.opencodePaths();
  const cfg = readJsonFile(p.config) as Record<string, unknown> | null;
  if (!cfg) return false;
  const plugins = cfg.plugin as unknown[] | undefined;
  if (!Array.isArray(plugins)) return false;
  return plugins.some(
    (pl) => typeof pl === "string" && pl.toLowerCase() === PONYTAIL_PKG.toLowerCase(),
  );
}
