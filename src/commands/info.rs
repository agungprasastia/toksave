use crate::registry::{ALL_AGENTS, ALL_TOOLS, detect_agent, tool_installed_version};
use crate::util::colors;
use colored::Colorize;

pub fn run_info() -> i32 {
    colors::banner("toksave info", "install + paths");

    // ── TokSave block ──
    println!("{}{}", "┌─ ".dimmed(), "TokSave".bold().magenta());
    let ver = crate::util::version::toksave_version();
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let (method, _) = read_install_info();
    let platform = format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH);

    println!("{}  {:<10}{}", "│".dimmed(), "version".cyan(), ver.bold());
    println!("{}  {:<10}{}", "│".dimmed(), "binary".cyan(), exe.dimmed());
    println!(
        "{}  {:<10}{}",
        "│".dimmed(),
        "install".cyan(),
        method.dimmed()
    );
    println!(
        "{}  {:<10}{}",
        "│".dimmed(),
        "platform".cyan(),
        platform.blue()
    );
    println!("{}", "│".dimmed());

    // ── Agents block ──
    println!("{}{}", "├─ ".dimmed(), "Agents".bold().magenta());
    let mut detected_any = false;
    for a in ALL_AGENTS {
        let det = detect_agent(a.id);
        if det.installed {
            detected_any = true;
            let path_str = if det.source.is_empty() {
                "—".dimmed().to_string()
            } else {
                det.source.dimmed().to_string()
            };
            println!(
                "{}  {} {:<16}{}",
                "│".dimmed(),
                "✔".green(),
                a.label.bold(),
                path_str
            );
        }
    }
    if !detected_any {
        println!(
            "{}  {}",
            "│".dimmed(),
            "none detected on this machine".dimmed()
        );
    }
    println!("{}", "│".dimmed());

    // ── Tools block ──
    println!("{}{}", "├─ ".dimmed(), "Tools".bold().magenta());
    for t in ALL_TOOLS {
        if t.instruction_only {
            continue;
        }
        if let Some(v) = tool_installed_version(t.id) {
            println!(
                "{}  {} {:<14}{:<10}",
                "│".dimmed(),
                "✔".green(),
                t.label.bold(),
                format!("v{v}").cyan()
            );
        } else {
            println!(
                "{}  {} {:<14}{}",
                "│".dimmed(),
                "·".dimmed(),
                t.label.dimmed(),
                "not installed".dimmed()
            );
        }
    }
    println!("{}", "│".dimmed());

    // ── State block ──
    println!("{}{}", "├─ ".dimmed(), "State".bold().magenta());
    let manifest_path = crate::util::paths::cache_dir().join("manifest.json");
    if manifest_path.exists() {
        println!(
            "{}  {} {:<16}{}",
            "│".dimmed(),
            "✔".green(),
            "manifest".bold(),
            manifest_path.display().to_string().dimmed()
        );
    } else {
        println!(
            "{}  {} {:<16}{}",
            "│".dimmed(),
            "·".dimmed(),
            "manifest".dimmed(),
            "not created yet".dimmed()
        );
    }
    println!("{}", "│".dimmed());

    // ── Tip ──
    println!(
        "{}  {} {:<8}{}{}",
        "│".dimmed(),
        "ℹ".cyan(),
        "tip".cyan(),
        "run ".dimmed(),
        "toksave doctor".cyan().bold()
    );
    println!("{}", "│".dimmed());

    println!();
    0
}

fn read_install_info() -> (String, Option<String>) {
    let path = install_json_path();
    if let Ok(data) = std::fs::read_to_string(&path)
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&data)
    {
        let method = v["method"].as_str().unwrap_or("install script").to_string();
        let at = v["at"].as_str().map(|s| s.to_string());
        return (method, at);
    }
    ("manual binary".to_string(), None)
}

fn install_json_path() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return std::path::PathBuf::from(local_app_data)
                .join("toksave")
                .join("install.json");
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = dirs::home_dir() {
            return home
                .join(".local")
                .join("share")
                .join("toksave")
                .join("install.json");
        }
    }
    crate::util::paths::cache_dir().join("install.json")
}
