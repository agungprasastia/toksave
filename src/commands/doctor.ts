import pc from "picocolors";
import { readJsonFile } from "../config/json.js";
import * as toml from "../config/toml.js";
import type { AgentId, RunOpts, ToolId } from "../registry.js";
import {
  ALL_AGENTS,
  ALL_TOOLS,
  detectAgent,
  toolHealthCheck,
  toolInstalledVersion,
  toolLatestVersion,
  toolRepair,
  verifyTool,
} from "../registry.js";
import * as colors from "../util/colors.js";
import type { HealthStatus } from "../util/health.js";
import { mcpSpawnHealthy } from "../util/mcpspawn.js";
import { formatPathFixResult, selfHealPath } from "../util/pathfix.js";
import * as paths from "../util/paths.js";
import { isUpToDate } from "../util/version.js";

function checkAgentMcpHealth(agent: AgentId, tool: ToolId): { healthy: boolean; reason?: string } {
  try {
    switch (agent) {
      case "claude": {
        const cfg = (readJsonFile(paths.claudePaths().globalJson) as Record<string, unknown>) ?? {};
        const srv = (cfg.mcpServers as Record<string, { command?: string; args?: string[] }>) ?? {};
        const entry = srv[tool];
        if (!entry) return { healthy: false, reason: "not configured" };
        return mcpSpawnHealthy(entry.command ?? "", entry.args ?? []);
      }
      case "opencode": {
        const cfg = (readJsonFile(paths.opencodePaths().config) as Record<string, unknown>) ?? {};
        const mcp =
          (cfg.mcp as Record<string, { type?: string; command?: string | string[] }>) ?? {};
        const entry = mcp[tool];
        if (!entry) {
          if (tool === "context-mode") {
            const plugins = (cfg.plugin as string[]) ?? [];
            if (
              plugins.some(
                (p) =>
                  typeof p === "string" && (p === "context-mode" || p.startsWith("context-mode@")),
              )
            ) {
              return { healthy: true };
            }
          }
          return { healthy: false, reason: "not configured" };
        }
        if (Array.isArray(entry.command)) {
          return mcpSpawnHealthy(entry.command[0] ?? "", entry.command.slice(1));
        }
        return mcpSpawnHealthy(entry.command ?? "");
      }
      case "codex": {
        const doc = toml.readTomlFile(paths.codexPaths().config);
        const mcpServers =
          (doc.mcp_servers as Record<string, { command?: string; args?: string[] }>) ?? {};
        const entry = mcpServers[tool];
        if (!entry) return { healthy: false, reason: "not configured" };
        return mcpSpawnHealthy(entry.command ?? "", entry.args ?? []);
      }
      case "copilot": {
        const cfg = (readJsonFile(paths.copilotPaths().mcpConfig) as Record<string, unknown>) ?? {};
        const srv = (cfg.mcpServers as Record<string, { command?: string; args?: string[] }>) ?? {};
        const entry = srv[tool];
        if (!entry) return { healthy: false, reason: "not configured" };
        return mcpSpawnHealthy(entry.command ?? "", entry.args ?? []);
      }
      case "droid": {
        const cfg = (readJsonFile(paths.droidPaths().mcpConfig) as Record<string, unknown>) ?? {};
        const srv = (cfg.mcpServers as Record<string, { command?: string; args?: string[] }>) ?? {};
        const entry = srv[tool];
        if (!entry) return { healthy: false, reason: "not configured" };
        return mcpSpawnHealthy(entry.command ?? "", entry.args ?? []);
      }
      case "devin": {
        const cfg = (readJsonFile(paths.devinPaths().mcpConfig) as Record<string, unknown>) ?? {};
        const srv = (cfg.mcpServers as Record<string, { command?: string; args?: string[] }>) ?? {};
        const entry = srv[tool];
        if (!entry) return { healthy: false, reason: "not configured" };
        return mcpSpawnHealthy(entry.command ?? "", entry.args ?? []);
      }
      case "antigravity": {
        for (const f of paths.antigravityMcpFiles()) {
          const cfg = (readJsonFile(f) as Record<string, unknown>) ?? {};
          const srv =
            (cfg.mcpServers as Record<string, { command?: string; args?: string[] }>) ?? {};
          const entry = srv[tool];
          if (!entry)
            return { healthy: false, reason: "not configured in all antigravity targets" };
          const h = mcpSpawnHealthy(entry.command ?? "", entry.args ?? []);
          if (!h.healthy) return h;
        }
        return { healthy: true };
      }
    }
  } catch (err) {
    return { healthy: false, reason: err instanceof Error ? err.message : String(err) };
  }
}

/** Run the doctor command: health check. */
export async function run(offline: boolean, fix: boolean, opts: RunOpts): Promise<number> {
  colors.banner("toksave doctor", "quick health check");

  const PAD = Math.max(
    18,
    ...ALL_AGENTS.map((a) => a.label.length + 2),
    ...ALL_TOOLS.map((t) => t.label.length + 2),
  );

  // ── Per-agent wiring status ─────────────────────────────
  for (const agent of ALL_AGENTS) {
    const det = detectAgent(agent.id);
    if (!det.installed) {
      colors.raw(
        `  ${pc.dim(colors.BULLET)} ${colors.pad(agent.label, PAD)}${pc.dim("not installed")}`,
      );
      continue;
    }

    const missing: string[] = [];
    for (const tool of ALL_TOOLS) {
      const verified = verifyTool(agent.id, tool.id);
      if (verified === false || verified === null) {
        missing.push(tool.label);
      } else if (tool.id === "codegraph" || tool.id === "context-mode") {
        // Deep verification: presence in config is not enough if binary won't launch
        const mcpStatus = checkAgentMcpHealth(agent.id, tool.id);
        if (!mcpStatus.healthy) {
          missing.push(`${tool.label} (${mcpStatus.reason ?? "unexecutable"})`);
        }
      }
    }

    if (missing.length === 0) {
      colors.raw(
        `  ${pc.green(colors.CHECK)} ${colors.pad(agent.label, PAD)}${pc.dim("all tools wired")}`,
      );
    } else {
      colors.raw(
        `  ${pc.yellow(colors.WARN)} ${colors.pad(agent.label, PAD)}${pc.yellow(`missing: ${missing.join(", ")}`)}`,
      );
    }
  }

  // ── Tool versions ───────────────────────────────────────
  if (!offline) {
    console.log();
    let outdated = 0;

    for (const tool of ALL_TOOLS) {
      const installed = toolInstalledVersion(tool.id);
      const latest = await toolLatestVersion(tool.id);
      const label = colors.pad(tool.label, PAD);

      if (tool.instructionOnly) {
        colors.raw(`  ${pc.green(colors.CHECK)} ${pc.dim(label)}${pc.dim("instruction-only")}`);
      } else if (installed && latest) {
        const instStr = installed.startsWith("v") ? installed : `v${installed}`;
        const latestStr = latest.startsWith("v") ? latest : `v${latest}`;
        if (isUpToDate(installed, latest)) {
          colors.raw(`  ${pc.green(colors.CHECK)} ${pc.dim(label)}${pc.dim(instStr)}`);
        } else {
          outdated++;
          colors.raw(
            `  ${pc.yellow(colors.ARROW_UP)} ${pc.dim(`${label}${instStr}`)}${pc.green(` → ${latestStr}`)}`,
          );
        }
      } else if (installed) {
        const instStr =
          installed === "skill" || installed === "instruction-only"
            ? installed
            : installed.startsWith("v")
              ? installed
              : `v${installed}`;
        colors.raw(`  ${pc.green(colors.CHECK)} ${pc.dim(label)}${pc.dim(instStr)}`);
      } else {
        colors.raw(`  ${pc.dim(colors.BULLET)} ${pc.dim(label)}${pc.dim("not installed")}`);
      }
    }

    console.log();
    if (outdated > 0) {
      colors.warn(`${outdated} update(s) available — run \`toksave update\``);
    } else {
      colors.ok("All up to date.");
    }
  }

  // ── PATH auto-fix ────────────────────────────────────────
  if (fix) {
    const msg = formatPathFixResult(selfHealPath());
    if (msg) colors.ok(msg);
  }

  // ── Tool health ─────────────────────────────────────────
  const unhealthy = ALL_TOOLS.map((tool) => ({ tool, health: toolHealthCheck(tool.id) })).filter(
    ({ health }) => !health.healthy,
  );

  if (unhealthy.length > 0) {
    console.log();
    for (const { tool, health } of unhealthy) {
      const label = colors.pad(tool.label, PAD);
      colors.raw(`  ${pc.yellow(colors.WARN)} ${label}${pc.yellow("unhealthy")}`);
      printHealthIssues(health);

      if (fix) {
        const result = await toolRepair(tool.id, opts);
        const icon = result.success ? pc.green(colors.CHECK) : pc.red(colors.CROSS);
        colors.raw(`  ${icon} ${colors.pad(tool.label, PAD)}${result.message}`);
        if (result.healthAfterRepair) {
          const status = result.healthAfterRepair.healthy
            ? pc.green("healthy")
            : pc.yellow("unhealthy");
          colors.raw(
            `  ${pc.dim(colors.BULLET)} ${colors.pad(tool.label, PAD)}after repair: ${status}`,
          );
          printHealthIssues(result.healthAfterRepair);
        }
      }
    }

    if (!fix) {
      console.log();
      colors.info("Run `toksave doctor --fix` to repair unhealthy tools.");
    }
  }

  // ── Fix suggestion ──────────────────────────────────────
  const broken = ALL_AGENTS.some((a) => {
    const det = detectAgent(a.id);
    if (!det.installed) return false;
    return ALL_TOOLS.some((t) => verifyTool(a.id, t.id) === false);
  });

  if (broken) {
    console.log();
    colors.info("Run `toksave` to fix.");
  }

  printRepoFooter();
  console.log();
  return 0;
}

function printRepoFooter(): void {
  if (process.env.TOKSAVE_TEST === "1") return;
  const repoURL = "https://github.com/agungprasastia/toksave";
  const issuesURL = `${repoURL}/issues`;
  console.log();
  console.log(`  ${pc.dim("─".repeat(52))}`);
  console.log(`  ${pc.dim("If toksave helps, please star it here: ")}${pc.cyan(repoURL)}`);
  console.log(
    `  ${pc.dim("If you hit any issue or have ideas, please raise it here: ")}${pc.cyan(issuesURL)}`,
  );
}

function printHealthIssues(health: HealthStatus): void {
  for (const issue of health.issues) {
    const icon = issue.severity === "error" ? pc.red(colors.CROSS) : pc.yellow(colors.WARN);
    colors.raw(`    ${icon} ${issue.message}`);
    if (issue.remediation) colors.raw(`      ${pc.dim(issue.remediation)}`);
  }
}
