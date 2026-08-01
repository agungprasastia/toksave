use crate::cli::ParsedCli;
use crate::registry::{
    agent_info, detect_agent, install_tool, tool_info, tool_installed_version, verify_tool,
    wire_tool, AgentId, ToolId, ALL_TOOLS,
};
use crate::util::colors;
use crate::util::exec::run_stdout;
use crate::util::manifest::record_wire;
use colored::Colorize;
use std::sync::{Arc, Mutex};

/// Shared progress handle: tool installs report phases back through
/// `RunOpts.report`, which re-enters the spinner via the mutex.
struct Prog(Arc<Mutex<crate::util::ui::Progress>>);

impl Prog {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(crate::util::ui::Progress::new())))
    }
    fn start_section(&self, name: &str) {
        self.0.lock().unwrap().start_section(name);
    }
    fn start(&self, label: &str) {
        self.0.lock().unwrap().start(label);
    }
    fn stop(&self, message: &str) {
        self.0.lock().unwrap().stop(message);
    }
    fn bar(&self) -> Option<indicatif::ProgressBar> {
        self.0.lock().unwrap().bar()
    }
    fn report_sink(&self) -> Option<crate::registry::ReportSink> {
        let inner = Arc::clone(&self.0);
        Some(Arc::new(move |phase, frac| {
            inner.lock().unwrap().phase(phase, frac);
        }))
    }
}

pub async fn run_init(parsed: &ParsedCli) -> i32 {
    colors::banner("toksave", "global token-saver for AI agents");

    // ── Step 1: Filter tools ──
    let tools: Vec<ToolId> = ALL_TOOLS
        .iter()
        .filter(|t| parsed.tools.is_empty() || parsed.tools.contains(&t.id))
        .map(|t| t.id)
        .collect();

    // Node dep check for npm-channel tools (port of ensureDeps)
    let has_npm_tools = tools
        .iter()
        .any(|t| tool_info(*t).channel == crate::registry::Channel::Npm);
    let min_node = tools
        .iter()
        .map(|t| tool_info(*t).min_node_major)
        .max()
        .unwrap_or(0);
    let deps_ok = check_deps(has_npm_tools, min_node);

    // ── Step 2: Install tools ──
    let mut installed_tools = std::collections::HashSet::new();
    let prog = Prog::new();
    prog.start_section("Tools");
    for t in &tools {
        let info = tool_info(*t);
        let is_npm = info.channel == crate::registry::Channel::Npm;
        if is_npm && !deps_ok {
            colors::warn(&format!("{} — needs Node.js", info.label));
            continue;
        }
        prog.start(&format!("Installing {}", info.label));
        if let Some(bar) = prog.bar() {
            crate::util::download::set_download_progress_bar(bar);
        }
        let mut opts = parsed.opts.clone();
        opts.report = prog.report_sink();
        let pre_version = tool_installed_version(*t);
        match install_tool(*t, &opts).await {
            Ok(true) => {
                installed_tools.insert(*t);
                let line = if parsed.opts.dry_run {
                    match tool_installed_version(*t).or(pre_version) {
                        Some(_) if info.instruction_only => {
                            format!("{} {} instruction-only", colors::CHECK, info.label)
                        }
                        Some(v) if v == "installed" => {
                            format!("{} {} installed", colors::CHECK, info.label)
                        }
                        Some(v) => format!("{} {} {}", colors::CHECK, info.label, v),
                        None => format!(
                            "{} {} not installed (would install)",
                            colors::WARN,
                            info.label
                        ),
                    }
                } else {
                    format!("{} {}", colors::CHECK, info.label)
                };
                prog.stop(&line);
            }
            Ok(false) => {
                let tail: &str = if parsed.opts.dry_run {
                    " — skipped (dry run)"
                } else {
                    " — skipped"
                };
                prog.stop(&format!("{} {}{}", colors::WARN, info.label, tail));
            }
            Err(e) => {
                let first = e.message.lines().next().unwrap_or("").to_string();
                prog.stop(&format!("{} {} — {}", colors::CROSS, info.label, first));
                for line in e.message.lines().skip(1) {
                    println!("      {line}");
                }
                if let Some(rem) = &e.remediation {
                    println!("      {}", rem.dimmed());
                }
            }
        }
        crate::util::download::clear_download_progress_bar();
    }

    // ── Step 3: Detect agents ──
    let mut detected: Vec<(AgentId, String)> = vec![];
    for a in crate::registry::ALL_AGENTS {
        let d = detect_agent(a.id);
        if d.installed {
            detected.push((a.id, d.source));
        }
    }

    // ── Step 4: Pick agents ──
    let detected_ids: Vec<AgentId> = detected.iter().map(|(id, _)| *id).collect();
    let requested: Vec<AgentId> = if !parsed.agents.is_empty() {
        parsed.agents.clone()
    } else if parsed.opts.yes || !is_interactive() {
        detected_ids
    } else {
        let select_options: Vec<crate::util::ui::SelectOption> = crate::registry::ALL_AGENTS
            .iter()
            .map(|a| {
                let det = detected.iter().find(|(id, _)| *id == a.id);
                crate::util::ui::SelectOption {
                    value: a.id,
                    label: a.label.to_string(),
                    disabled: det.is_none(),
                    hint: det
                        .map(|(_, src)| src.clone())
                        .unwrap_or_else(|| a.homepage.to_string()),
                    selected: false,
                }
            })
            .collect();
        crate::util::ui::multi_select("Select agents to wire toksave into", select_options)
    };

    if requested.is_empty() {
        println!("  Nothing selected.");
        return 0;
    }

    let detected_by_id: std::collections::HashMap<AgentId, String> = detected.into_iter().collect();

    // ── Step 5: Wire tools into agents ──
    let mut failures: Vec<(AgentId, Vec<String>)> = vec![];
    let prog = Prog::new();
    prog.start_section("Agents");
    for agent_id in &requested {
        let Some(_source) = detected_by_id.get(agent_id) else {
            let info = agent_info(*agent_id);
            colors::warn(&format!(
                "{} not installed — install it first: {}",
                info.label, info.homepage
            ));
            continue;
        };
        let info = agent_info(*agent_id);
        let dry_suffix = if parsed.opts.dry_run {
            " (dry run)"
        } else {
            ""
        };
        prog.start(&format!("Wiring {}{}", info.label, dry_suffix));
        let mut failed_tools: Vec<String> = vec![];
        for t in &tools {
            if !installed_tools.contains(t) {
                failed_tools.push(tool_info(*t).label.to_string());
                continue;
            }
            match wire_tool(*agent_id, *t, &parsed.opts).await {
                Ok(true) => {
                    if parsed.opts.dry_run {
                        if tool_installed_version(*t).is_none() {
                            failed_tools.push(tool_info(*t).label.to_string());
                        }
                        continue;
                    }
                    if verify_tool(*agent_id, *t) == Some(false) {
                        failed_tools.push(tool_info(*t).label.to_string());
                        continue;
                    }
                    let _ = record_wire(
                        &format!("{:?}", agent_id).to_lowercase(),
                        &tool_name(*t),
                        tool_installed_version(*t).as_deref(),
                    );
                }
                _ => failed_tools.push(tool_info(*t).label.to_string()),
            }
        }
        if failed_tools.is_empty() {
            let suffix = if parsed.opts.dry_run {
                " (dry run)"
            } else {
                ""
            };
            prog.stop(&format!("{} {}{}", colors::CHECK, info.label, suffix));
        } else {
            let suffix = if parsed.opts.dry_run {
                " (dry run)"
            } else {
                ""
            };
            prog.stop(&format!(
                "{} {}{} — {} not wired",
                colors::WARN,
                info.label,
                suffix,
                failed_tools.join(", ")
            ));
            failures.push((*agent_id, failed_tools));
        }
    }

    // ── Step 6: Summary ──
    let wired: Vec<&str> = requested
        .iter()
        .filter(|id| !failures.iter().any(|(fid, _)| fid == *id))
        .filter(|id| detected_by_id.contains_key(*id))
        .map(|id| agent_info(*id).label)
        .collect();
    if !wired.is_empty() {
        crate::util::ui::green_box(&format!("Equipped {}.", wired.join(", ")));
    }
    for (id, failed) in &failures {
        colors::warn(&format!(
            "{}: {} not wired. Run `toksave doctor` for details.",
            agent_info(*id).label,
            failed.join(", ")
        ));
    }
    print_version_table(&tools);
    println!();

    if failures.is_empty() || parsed.opts.dry_run {
        0
    } else {
        1
    }
}

fn check_deps(need_node: bool, min_node: u32) -> bool {
    if !need_node {
        return true;
    }
    let Some(out) = run_stdout("node", &["--version"]) else {
        return false;
    };
    let v = out.trim_start_matches('v');
    let major: u32 = v
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if major < min_node {
        eprintln!(
            "Node.js {out} detected but >= v{min_node}.x required. Upgrade Node.js at https://nodejs.org"
        );
        return false;
    }
    true
}

fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

fn tool_name(t: ToolId) -> String {
    match t {
        ToolId::Rtk => "rtk".to_string(),
        ToolId::Caveman => "caveman".to_string(),
        ToolId::Codegraph => "codegraph".to_string(),
        ToolId::ContextMode => "context-mode".to_string(),
        ToolId::Ponytail => "ponytail".to_string(),
        ToolId::Principles => "principles".to_string(),
    }
}

fn print_version_table(tools: &[ToolId]) {
    for t in tools {
        let info = tool_info(*t);
        if info.instruction_only {
            println!("  {} {} instruction-only", colors::CHECK, info.label);
            continue;
        }
        match tool_installed_version(*t) {
            Some(v) => println!("  {} {} {}", colors::CHECK, info.label, v),
            None => println!("  {} {} not installed", colors::BULLET, info.label),
        }
    }
}
