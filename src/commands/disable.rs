use crate::cli::ParsedCli;
use crate::registry::{
    agent_info, detect_agent, tool_info, unwire_tool, AgentId, ToolId, ALL_AGENTS, ALL_TOOLS,
};
use crate::util::colors;
use crate::util::manifest::mark_disabled;

/// Disable = surgical uninstall: unwire from agents, keep binaries, keep manifest
/// entries (marked `disabled`). No cache purge, no rtk/npm binary removal.
pub async fn run_disable(parsed: &ParsedCli) -> i32 {
    colors::banner("toksave", "disable");

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
    } else {
        // Non-interactive assumed (CLI binary has no TTY prompt); mirror --yes/CI behavior.
        detected.clone()
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

    // ── Unwire + mark disabled ──
    let mut prog = crate::util::ui::Progress::new();
    for agent_id in &agent_ids {
        let info = agent_info(*agent_id);
        prog.start(&format!("Disabling in {}", info.label));
        for tool_id in &tools {
            if !parsed.opts.dry_run {
                let _ = unwire_tool(*agent_id, *tool_id, &parsed.opts).await;
                let _ = mark_disabled(
                    &format!("{:?}", agent_id).to_lowercase(),
                    tool_name(*tool_id),
                );
            }
        }
        prog.stop(&format!("{} {}", colors::CHECK, info.label));
    }

    // ── Summary ──
    println!();
    let agent_labels: Vec<&str> = agent_ids.iter().map(|id| agent_info(*id).label).collect();
    let tool_labels: Vec<&str> = tools.iter().map(|id| tool_info(*id).label).collect();
    colors::ok(&format!(
        "Disabled {} from {}.",
        tool_labels.join(", "),
        agent_labels.join(", ")
    ));
    println!();
    0
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
