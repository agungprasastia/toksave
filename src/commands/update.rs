use crate::cli::ParsedCli;
use crate::registry::{
    ALL_AGENTS, ALL_TOOLS, ToolId, detect_agent, install_tool, tool_info, tool_installed_version,
    verify_tool, wire_tool,
};
use crate::tools::tool_latest_version;
use crate::util::colors;
use crate::util::manifest::record_wire;
use colored::Colorize;
use std::collections::HashMap;

pub async fn run_update(parsed: &ParsedCli) -> i32 {
    colors::banner("toksave update", "refresh tools to latest");

    // ── Resolve target tools ──
    let tools: Vec<ToolId> = ALL_TOOLS
        .iter()
        .filter(|t| parsed.tools.is_empty() || parsed.tools.contains(&t.id))
        .map(|t| t.id)
        .collect();

    let pad = tools
        .iter()
        .map(|t| tool_info(*t).label.len() + 2)
        .max()
        .unwrap_or(18)
        .max(18);

    // ── Probe latest versions in parallel ──
    let mut set = tokio::task::JoinSet::new();
    for t in &tools {
        let id = *t;
        set.spawn(async move { (id, tool_latest_version(id).await) });
    }
    let mut latest_versions: HashMap<ToolId, Option<String>> = HashMap::new();
    while let Some(Ok((id, latest))) = set.join_next().await {
        latest_versions.insert(id, latest);
    }

    // ── Determine changed tools ──
    let mut changed: Vec<ToolId> = vec![];
    for t in &tools {
        let info = tool_info(*t);
        let installed = tool_installed_version(*t);
        let latest = latest_versions.get(t).cloned().flatten();
        let label = format!("{:<width$}", info.label, width = pad);
        if info.instruction_only {
            println!(
                "  {} {}{}",
                colors::CHECK.green(),
                label,
                "instruction-only (up to date)".dimmed()
            );
            continue;
        }
        let inst_str = installed
            .as_deref()
            .map(display_version)
            .unwrap_or_else(|| "not on PATH".to_string());
        let lat_str = latest
            .as_deref()
            .map(display_version)
            .unwrap_or_else(|| "?".to_string());
        let needs_upgrade = installed.is_some()
            && latest.is_some()
            && !version_up_to_date(installed.as_deref().unwrap(), latest.as_deref().unwrap());
        let needs_install = installed.is_none() && latest.is_some();

        if needs_upgrade {
            changed.push(*t);
            println!(
                "  {} {}{}{}",
                "↑".yellow(),
                label,
                format!("{inst_str} ").yellow(),
                format!("→ {lat_str} → upgrade").green()
            );
        } else if needs_install {
            changed.push(*t);
            println!(
                "  {} {}{}{}",
                "+".yellow(),
                label,
                format!("{inst_str} ").red(),
                format!("→ {lat_str} → install").green()
            );
        } else if installed.is_some() {
            let state = if latest.is_some() {
                "(up to date)".dimmed()
            } else {
                "(latest unknown — could not check)".dimmed()
            };
            println!(
                "  {} {}{}{}",
                colors::CHECK.green(),
                label,
                format!("{inst_str} → {lat_str} ").green(),
                state
            );
        } else {
            println!(
                "  {} {}{}",
                "•".yellow(),
                label,
                format!("{inst_str} → {lat_str} (not installed)").red()
            );
        }
    }
    println!();

    if changed.is_empty() {
        colors::ok("Everything up to date.");
        println!();
        return 0;
    }

    if parsed.opts.dry_run {
        let names: Vec<&str> = changed.iter().map(|id| tool_info(*id).label).collect();
        colors::info(&format!("Would upgrade: {}", names.join(", ")));
        println!();
        return 0;
    }

    // ── Upgrade changed tools in parallel ──
    let upgrade_opts = crate::registry::RunOpts {
        dry_run: false,
        upgrade: true,
        verbose: parsed.opts.verbose,
        yes: parsed.opts.yes,
        report: None,
    };

    let mut prog = crate::util::ui::Progress::new();
    let mut upgraded: Vec<ToolId> = vec![];
    let mut failed: Vec<String> = vec![];
    for id in &changed {
        let info = tool_info(*id);
        prog.start(&format!("Upgrading {}", info.label));
        match install_tool(*id, &upgrade_opts).await {
            Ok(true) => {
                upgraded.push(*id);
                prog.stop(&format!("{} {}", colors::CHECK, info.label));
            }
            Ok(false) => {
                prog.stop(&format!("{} {} — skipped", colors::WARN, info.label));
            }
            Err(e) => {
                failed.push(info.label.to_string());
                let first = e.message.lines().next().unwrap_or("").to_string();
                prog.stop(&format!("{} {} — {}", colors::CROSS, info.label, first));
            }
        }
    }
    prog.done();

    // ── Re-sync wiring (only where already wired) ──
    for tool_id in &upgraded {
        for agent in ALL_AGENTS {
            if !detect_agent(agent.id).installed {
                continue;
            }
            if verify_tool(agent.id, *tool_id) != Some(true) {
                continue;
            }
            if wire_tool(agent.id, *tool_id, &upgrade_opts).await.is_ok() {
                let _ = record_wire(
                    &format!("{:?}", agent.id).to_lowercase(),
                    tool_name(*tool_id),
                    tool_installed_version(*tool_id).as_deref(),
                );
            }
        }
    }
    // ── Re-sync instruction wiring (only where already wired) ──
    for tool in ALL_TOOLS {
        if !tool.instruction_only {
            continue;
        }
        for agent in ALL_AGENTS {
            if !detect_agent(agent.id).installed {
                continue;
            }
            if verify_tool(agent.id, tool.id) != Some(true) {
                continue;
            }
            let _ = wire_tool(agent.id, tool.id, &upgrade_opts).await;
        }
    }

    // ── Summary ──
    if !upgraded.is_empty() {
        let names: Vec<&str> = upgraded.iter().map(|id| tool_info(*id).label).collect();
        colors::ok(&format!("Updated {}.", names.join(", ")));
    }
    for name in &failed {
        colors::warn(&format!("{name} failed to update."));
    }
    println!();

    if failed.is_empty() { 0 } else { 1 }
}

/// Render a version string: keep semantic versions as-is, prefix `v` only when
/// it looks like a plain number/version (so labels like "installed" or
/// "instruction-only" aren't mangled into "vinstalled").
fn display_version(v: &str) -> String {
    let trimmed = v.trim().trim_start_matches('v');
    if trimmed.chars().next().is_some_and(|c| c.is_numeric()) || trimmed.contains('.') {
        format!("v{trimmed}")
    } else {
        v.to_string()
    }
}

fn tool_name(t: ToolId) -> &'static str {
    match t {
        ToolId::Rtk => "rtk",
        ToolId::Caveman => "caveman",
        ToolId::Codegraph => "codegraph",
        ToolId::ContextMode => "context-mode",
        ToolId::Ponytail => "ponytail",
        ToolId::Principles => "principles",
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
