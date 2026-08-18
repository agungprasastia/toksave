use crate::cli::ParsedCli;
use crate::registry::{
    ALL_AGENTS, ALL_TOOLS, ToolId, detect_agent, tool_info, unwire_tool, verify_tool, wire_tool,
};
use crate::tools::{tool_health_check, tool_installed_version, tool_latest_version, tool_repair};
use crate::util::colors;
use crate::util::health::{HealthStatus, Severity};
use crate::util::manifest::record_wire;
use crate::util::probe::{RuntimeIssue, probe_agent};
use colored::Colorize;

pub async fn run_doctor(parsed: &ParsedCli, offline: bool, fix: bool) -> i32 {
    colors::banner("toksave doctor", "quick health check");

    let pad = ALL_AGENTS
        .iter()
        .map(|a| a.label.len())
        .chain(ALL_TOOLS.iter().map(|t| t.label.len()))
        .max()
        .unwrap_or(16)
        .max(16)
        + 2;

    // Per-agent wiring status
    for agent in ALL_AGENTS {
        let det = detect_agent(agent.id);
        let label = format!("{:<width$}", agent.label, width = pad);
        if !det.installed {
            println!(
                "  {} {}{}",
                colors::BULLET.dimmed(),
                label,
                "not installed".red()
            );
            continue;
        }
        let mut missing_ids: Vec<ToolId> = ALL_TOOLS
            .iter()
            .filter(|t| verify_tool(agent.id, t.id) != Some(true))
            .map(|t| t.id)
            .collect();

        if fix {
            for issue in probe_agent(agent.id) {
                if !probe_issue_is_stale_wiring(&issue) {
                    continue;
                }
                match issue.kind {
                    "mcp" => {
                        push_unique(&mut missing_ids, ToolId::Codegraph);
                        push_unique(&mut missing_ids, ToolId::ContextMode);
                    }
                    "hook" => push_unique(&mut missing_ids, ToolId::Rtk),
                    _ => {}
                }
            }
        }

        if fix && !missing_ids.is_empty() {
            missing_ids = repair_agent_wiring(agent.id, &missing_ids, parsed).await;
        }

        if missing_ids.is_empty() {
            println!(
                "  {} {}{}",
                colors::CHECK.green(),
                label,
                "all tools wired".green()
            );
            if agent.id == crate::registry::AgentId::Claude
                && verify_tool(agent.id, ToolId::Rtk) == Some(true)
            {
                println!(
                    "      {}",
                    "official `rtk init --show` reports the Claude hook missing when only toksave's wrapper is present — keep only the toksave hook (`rtk init -g --no-patch`)".dimmed()
                );
            }
        } else {
            let missing_str = format!(
                "missing: {}",
                missing_ids
                    .iter()
                    .map(|id| tool_info(*id).label)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "  {} {}{}",
                colors::WARN.yellow(),
                label,
                missing_str.yellow()
            );
            if !fix {
                println!(
                    "      {}",
                    "run `toksave doctor --fix` to repair wiring".dimmed()
                );
            }
        }

        // Runtime probe: wired hook/MCP commands must resolve and run.
        for issue in probe_agent(agent.id) {
            println!(
                "      {} {}{}",
                colors::WARN.yellow(),
                issue.kind,
                format!(" — {}", issue.detail).yellow()
            );
        }
    }

    // Tool versions (skip when offline)
    if !offline {
        println!();
        let mut outdated = 0usize;
        for tool in ALL_TOOLS {
            let installed = tool_installed_version(tool.id);
            let latest = tool_latest_version(tool.id).await;
            let label = format!("{:<width$}", tool.label, width = pad);
            if tool.instruction_only {
                println!(
                    "  {} {}{}",
                    colors::CHECK.green(),
                    label.dimmed(),
                    "instruction-only".dimmed()
                );
            } else if let Some(inst) = installed {
                let inst_str = if inst.starts_with('v') {
                    inst.clone()
                } else {
                    format!("v{inst}")
                };
                match latest {
                    Some(lat) if !version_up_to_date(&inst, &lat) => {
                        outdated += 1;
                        let lat_str = if lat.starts_with('v') {
                            lat.clone()
                        } else {
                            format!("v{lat}")
                        };
                        println!(
                            "  {} {}{}{}",
                            "↑ ".yellow(),
                            label.dimmed(),
                            inst_str.yellow(),
                            format!(" → {lat_str}").green()
                        );
                    }
                    _ => println!(
                        "  {} {}{}",
                        colors::CHECK.green(),
                        label.dimmed(),
                        inst_str.green()
                    ),
                }
            } else {
                println!(
                    "  {} {}{}",
                    colors::BULLET.dimmed(),
                    label.dimmed(),
                    "not installed".red()
                );
            }
        }
        println!();
        if outdated > 0 {
            colors::warn(&format!(
                "{outdated} update(s) available — run `toksave update`"
            ));
        } else {
            colors::ok("All up to date.");
        }
    }

    // Tool health
    let unhealthy: Vec<_> = ALL_TOOLS
        .iter()
        .filter_map(|t| {
            let h = tool_health_check(t.id);
            if h.healthy { None } else { Some((t, h)) }
        })
        .collect();

    if !unhealthy.is_empty() {
        println!();
        for (tool, health) in &unhealthy {
            let label = format!("{:<width$}", tool.label, width = pad);
            println!(
                "  {} {}{}",
                colors::WARN.yellow(),
                label,
                "unhealthy".yellow()
            );
            print_health_issues(health);
            if fix {
                let result = tool_repair(tool.id, &parsed.opts).await;
                let icon = if result.success {
                    colors::CHECK.green()
                } else {
                    colors::CROSS.red()
                };
                let repair_label = format!("{:<width$}", tool.label, width = pad);
                println!("  {} {}{}", icon, repair_label, result.message);
                if let Some(after) = &result.health_after_repair {
                    let status = if after.healthy {
                        "healthy".green()
                    } else {
                        "unhealthy".yellow()
                    };
                    let after_label = format!("{:<width$}", tool.label, width = pad);
                    println!(
                        "  {} {}after repair: {}",
                        colors::BULLET.dimmed(),
                        after_label,
                        status
                    );
                    print_health_issues(after);
                }
            }
        }
        if !fix {
            println!();
            colors::info("Run `toksave doctor --fix` to repair unhealthy tools.");
        }
    }

    // Suggest init if any installed agent has unwired tools
    let broken = ALL_AGENTS.iter().any(|a| {
        detect_agent(a.id).installed
            && ALL_TOOLS
                .iter()
                .any(|t| verify_tool(a.id, t.id) == Some(false))
    });
    if broken {
        println!();
        colors::info("Run `toksave` to fix.");
    }

    println!();
    0
}

/// Actually repair broken/missing wiring: unwire then rewire each tool cleanly instead of
/// layering a new wire attempt on top of whatever stale state is already there (the same
/// unwire-before-rewire discipline `toksave uninstall` + a fresh `toksave` already give you
/// manually, just automated here). Tools whose underlying binary/package was never
/// successfully installed are skipped -- rewiring an agent config to point at a tool that
/// isn't there just produces a different kind of broken wiring.
/// Returns the tool ids that are still not wired after the attempt.
async fn repair_agent_wiring(
    agent: crate::registry::AgentId,
    missing_ids: &[ToolId],
    parsed: &ParsedCli,
) -> Vec<ToolId> {
    let mut still_missing = vec![];
    for tool_id in missing_ids {
        let info = tool_info(*tool_id);
        if !info.instruction_only
            && *tool_id != ToolId::Rtk
            && tool_installed_version(*tool_id).is_none()
        {
            let _ = tool_repair(*tool_id, &parsed.opts).await;
        }
        if !info.instruction_only
            && *tool_id != ToolId::Rtk
            && tool_installed_version(*tool_id).is_none()
        {
            still_missing.push(*tool_id);
            if matches!(*tool_id, ToolId::Codegraph | ToolId::ContextMode) {
                println!(
                    "      {} still broken: {} (not installed)",
                    colors::CROSS.red(),
                    info.label
                );
                println!(
                    "      {}",
                    "add the npm global bin and toksave install dir to the user PATH, then restart the agent".dimmed()
                );
                if cfg!(windows) {
                    println!(
                        "      {}",
                        r"Windows: %APPDATA%\npm and %LOCALAPPDATA%\Programs\toksave".dimmed()
                    );
                }
            }
            continue;
        }

        let _ = unwire_tool(agent, *tool_id, &parsed.opts).await;
        let wired = matches!(wire_tool(agent, *tool_id, &parsed.opts).await, Ok(true));
        let repaired = wired && verify_tool(agent, *tool_id) == Some(true);

        if repaired {
            println!("      {} rewired {}", colors::CHECK.green(), info.label);
            let _ = record_wire(
                &format!("{agent:?}").to_lowercase(),
                &tool_wire_name(*tool_id),
                tool_installed_version(*tool_id).as_deref(),
            );
        } else {
            println!("      {} still broken: {}", colors::CROSS.red(), info.label);
            still_missing.push(*tool_id);
        }
    }
    still_missing
}

fn tool_wire_name(t: ToolId) -> String {
    match t {
        ToolId::Rtk => "rtk".to_string(),
        ToolId::Caveman => "caveman".to_string(),
        ToolId::Codegraph => "codegraph".to_string(),
        ToolId::ContextMode => "context-mode".to_string(),
        ToolId::Ponytail => "ponytail".to_string(),
        ToolId::Principles => "principles".to_string(),
    }
}

fn push_unique(ids: &mut Vec<ToolId>, id: ToolId) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn probe_issue_is_stale_wiring(issue: &RuntimeIssue) -> bool {
    issue.detail.contains("binary not found")
        || issue.detail.contains("backslash path")
        || issue.detail.contains("/$bunfs/")
        || issue.detail.contains("mcp target not found")
}

fn print_health_issues(health: &HealthStatus) {
    for issue in &health.issues {
        let icon = match issue.severity {
            Severity::Error => colors::CROSS.red(),
            Severity::Warning => colors::WARN.yellow(),
        };
        println!("    {} {}", icon, issue.message);
        if let Some(rem) = &issue.remediation {
            println!("      {}", rem.dimmed());
        }
    }
}

fn version_up_to_date(installed: &str, latest: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
            })
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let i = parse(installed);
    let l = parse(latest);
    for k in 0..3 {
        let iv = i.get(k).copied().unwrap_or(0);
        let lv = l.get(k).copied().unwrap_or(0);
        if iv != lv {
            return iv > lv;
        }
    }
    true
}
