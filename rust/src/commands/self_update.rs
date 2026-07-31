//! `toksave self-update` — update the toksave CLI itself via GitHub
//! Releases + install script. Mirrors TS src/commands/self-update.ts.

use crate::util::colors;
use crate::util::download::fetch_json;
use crate::util::exec::{run as exec_run, run_ok};
use crate::util::version::toksave_version;
use colored::Colorize;

const OWNER: &str = "agungprasastia";
const REPO: &str = "toksave";

fn up_to_date(local: &str, latest: &str) -> bool {
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
    let i = parse(local);
    let l = parse(latest);
    for k in 0..3usize {
        let (a, b) = (
            i.get(k).copied().unwrap_or(0),
            l.get(k).copied().unwrap_or(0),
        );
        if a != b {
            return a > b;
        }
    }
    true
}

/// Run the self-update command.
pub async fn run() -> i32 {
    colors::banner("toksave self-update", "update the toksave CLI itself");

    let local = toksave_version();
    println!("  local:  {local}");

    let latest = match fetch_json(&format!(
        "https://api.github.com/repos/{OWNER}/{REPO}/releases/latest"
    ))
    .await
    .ok()
    .and_then(|j| j.get("tag_name").and_then(|t| t.as_str()).map(String::from))
    .map(|t| t.trim_start_matches('v').to_string())
    {
        Some(t) if !t.is_empty() => t,
        _ => {
            colors::err("Could not reach GitHub Releases. Try again later.");
            colors::warn("Manual update:");
            println!(
                "  {}",
                format!(
                    "curl -fsSL https://raw.githubusercontent.com/{OWNER}/{REPO}/main/scripts/install.sh | bash"
                )
                .cyan()
            );
            return 1;
        }
    };

    println!("  latest: {latest}");

    if up_to_date(local, &latest) {
        colors::ok("Already up to date.");
        return 0;
    }

    println!("  Updating {local} → {latest}…");

    if cfg!(windows) {
        colors::warn("On Windows, run:");
        println!(
            "  {}",
            format!(
                "irm https://raw.githubusercontent.com/{OWNER}/{REPO}/main/scripts/install.ps1 | iex"
            )
            .cyan()
        );
        return 0;
    }

    if run_ok("which", &["curl"]) && run_ok("which", &["bash"]) {
        let url =
            format!("https://raw.githubusercontent.com/{OWNER}/{REPO}/main/scripts/install.sh");
        let r = exec_run("bash", &["-c", &format!("curl -fsSL {url} | bash")]);
        if r.code == 0 {
            colors::ok(&format!("Updated to {latest}. Restart your shell."));
            return 0;
        }
        colors::err("Auto-update failed.");
    }

    colors::warn("Manual update:");
    println!(
        "  {}",
        format!(
            "curl -fsSL https://raw.githubusercontent.com/{OWNER}/{REPO}/main/scripts/install.sh | bash"
        )
        .cyan()
    );
    0
}
