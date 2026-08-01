use crate::cli::ParsedCli;
use crate::registry::{detect_agent, verify_tool, ALL_AGENTS, ALL_TOOLS};
use crate::tools::{tool_health_check, tool_installed_version, tool_latest_version, tool_repair};
use crate::util::colors;
use crate::util::health::{HealthStatus, Severity};
use crate::util::probe::probe_agent;
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
                "not installed".dimmed()
            );
            continue;
        }
        let mut missing: Vec<String> = vec![];
        for tool in ALL_TOOLS {
            match verify_tool(agent.id, tool.id) {
                Some(true) => {}
                _ => missing.push(tool.label.to_string()),
            }
        }
        if missing.is_empty() {
            println!(
                "  {} {}{}",
                colors::CHECK.green(),
                label,
                "all tools wired".dimmed()
            );
        } else {
            let missing_str = format!("missing: {}", missing.join(", "));
            println!(
                "  {} {}{}",
                colors::WARN.yellow(),
                label,
                missing_str.yellow()
            );
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
                            "  {} {}{}",
                            "↑ ".yellow(),
                            format!("{label}{inst_str}").dimmed(),
                            format!(" → {lat_str}").green()
                        );
                    }
                    _ => println!(
                        "  {} {}{}",
                        colors::CHECK.green(),
                        label.dimmed(),
                        inst_str.dimmed()
                    ),
                }
            } else {
                println!(
                    "  {} {}{}",
                    colors::BULLET.dimmed(),
                    label.dimmed(),
                    "not installed".dimmed()
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
            if h.healthy {
                None
            } else {
                Some((t, h))
            }
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
