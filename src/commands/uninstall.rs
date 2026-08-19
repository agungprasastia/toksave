use crate::cli::ParsedCli;
use crate::registry::{
    ALL_AGENTS, ALL_TOOLS, AgentId, ToolId, agent_info, detect_agent, tool_info, unwire_tool,
    verify_tool,
};
use crate::util::colors;
use crate::util::manifest::remove_wire;
use std::process::Command;

pub async fn run_uninstall(parsed: &ParsedCli) -> i32 {
    colors::banner("toksave", "uninstall");

    // ── Detect installed agents ──
    let detected: Vec<AgentId> = ALL_AGENTS
        .iter()
        .filter(|a| detect_agent(a.id).installed)
        .map(|a| a.id)
        .collect();

    if detected.is_empty() {
        println!("  Nothing wired.");
        println!();
        return 0;
    }

    // ── Pick agents ──
    let agent_ids: Vec<AgentId> = if !parsed.agents.is_empty() {
        let detected_set: std::collections::HashSet<AgentId> = detected.iter().copied().collect();
        parsed
            .agents
            .iter()
            .copied()
            .filter(|id| detected_set.contains(id))
            .collect()
    } else if parsed.opts.yes || !is_interactive() {
        detected.clone()
    } else {
        let select_options: Vec<crate::util::ui::SelectOption> = crate::registry::ALL_AGENTS
            .iter()
            .map(|a| {
                let is_det = detected.contains(&a.id);
                crate::util::ui::SelectOption {
                    value: a.id,
                    label: a.label.to_string(),
                    disabled: !is_det,
                    hint: if is_det {
                        "installed".to_string()
                    } else {
                        a.homepage.to_string()
                    },
                    selected: false,
                }
            })
            .collect();
        crate::util::ui::multi_select("Select agents to uninstall toksave from", select_options)
    };

    if agent_ids.is_empty() {
        println!("  Nothing selected.");
        println!();
        return 0;
    }

    // ── Pick tools ──
    let tools: Vec<ToolId> = if parsed.tools.is_empty() {
        ALL_TOOLS.iter().map(|t| t.id).collect()
    } else {
        parsed.tools.clone()
    };

    // ── Unwire ──
    let mut prog = crate::util::ui::Progress::new();
    let mut residual: Vec<(AgentId, ToolId)> = vec![];
    for agent_id in &agent_ids {
        let info = agent_info(*agent_id);
        prog.start(&format!("Uninstalling from {}", info.label));
        for tool_id in &tools {
            if !parsed.opts.dry_run {
                let _ = unwire_tool(*agent_id, *tool_id, &parsed.opts).await;
                let _ = remove_wire(
                    &format!("{:?}", agent_id).to_lowercase(),
                    tool_name(*tool_id),
                );
                // Warp's Rtk has no hook file to remove (relies on `rtk` on PATH), so
                // verify() always reports Some(true) -- that's not residual wiring.
                let is_warp_rtk = matches!((agent_id, tool_id), (AgentId::Warp, ToolId::Rtk));
                if !is_warp_rtk && verify_tool(*agent_id, *tool_id) == Some(true) {
                    residual.push((*agent_id, *tool_id));
                }
            }
        }
        prog.stop(&format!("{} {}", colors::CHECK, info.label));
    }

    for (agent_id, tool_id) in &residual {
        colors::warn(&format!(
            "{} still appears wired to {} after unwire -- config may need manual cleanup",
            tool_info(*tool_id).label,
            agent_info(*agent_id).label
        ));
    }

    // ── Cleanup cache + purge binaries on full removal ──
    if !parsed.opts.dry_run && tools.len() == ALL_TOOLS.len() && agent_ids.len() == detected.len() {
        let cache = crate::util::paths::cache_dir();
        if cache.exists() {
            let _ = std::fs::remove_dir_all(&cache);
        }
        purge_binaries_if_confirmed(parsed);
    }

    // ── Summary ──
    println!();
    let agent_labels: Vec<&str> = agent_ids.iter().map(|id| agent_info(*id).label).collect();
    let tool_labels: Vec<&str> = tools.iter().map(|id| tool_info(*id).label).collect();
    colors::ok(&format!(
        "Uninstalled {} from {}.",
        tool_labels.join(", "),
        agent_labels.join(", ")
    ));
    println!();
    0
}

fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
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

/// Prompt to purge toksave-installed binaries. Mirrors TS: only asked when
/// interactive; returns false (skip) on non-TTY or user declining.
fn interactive_confirm() -> bool {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return false;
    }
    dialoguer::Confirm::new()
        .with_prompt("Also remove binaries/packages toksave installed (rtk, npm globals)?")
        .default(false)
        .interact()
        .unwrap_or(false)
}

fn purge_binaries_if_confirmed(parsed: &ParsedCli) {
    if parsed.opts.dry_run {
        println!("  [dry-run] would purge toksave-installed binaries + npm globals");
        return;
    }
    if std::env::var("TOKSAVE_TEST").is_ok_and(|v| v == "1") {
        return;
    }
    if !parsed.opts.yes && !interactive_confirm() {
        return;
    }

    // rtk
    let local_bin = crate::util::paths::local_bin();
    let rtk_path = if cfg!(windows) {
        local_bin.join("rtk.exe")
    } else {
        local_bin.join("rtk")
    };
    if rtk_path.exists() {
        let _ = Command::new(&rtk_path)
            .args(["init", "--uninstall"])
            .output();
        let _ = std::fs::remove_file(&rtk_path);
    }

    // npm globals
    for pkg in [
        "context-mode",
        "@colbymchenry/codegraph",
        "@dietrichgebert/ponytail",
    ] {
        let _ = Command::new(crate::util::exec::npm_cmd())
            .args(["uninstall", "-g", pkg])
            .output();
    }
}
