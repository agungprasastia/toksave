//! `toksave index [--auto]` — build per-project CodeGraph index in the
//! current (or detected) project dir. Mirrors TS src/commands/build-index.ts.

use crate::commands::hooks::agy::{find_project_dir, looks_like_project};
use crate::tools::codegraph;
use crate::util::colors;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};

/// Index in current working dir.
pub fn run_index(auto: bool) -> i32 {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    run_index_with(auto, &cwd)
}

/// Index the given dir. Separated from cwd for testability.
pub fn run_index_with(auto: bool, start: &Path) -> i32 {
    let cwd = start.to_path_buf();

    let dir = if auto {
        let d = find_project_dir(&cwd);
        if !looks_like_project(&d) {
            return 0; // silently skip
        }
        d
    } else {
        println!();
        println!(
            "  {}{}",
            "toksave index".bold().cyan(),
            format!("  build per-project indexes in {}", cwd.display()).dimmed()
        );
        println!();
        cwd
    };

    if env::var("TOKSAVE_TEST").is_err() && codegraph::installed_version().is_none() {
        if !auto {
            println!(
                "  {} CodeGraph  {}",
                colors::BULLET.dimmed(),
                "not installed — run toksave first".dimmed()
            );
        }
        return 1;
    }

    let on_progress = |m: &str| {
        if !auto {
            println!("  {m}");
        }
    };
    let opts = codegraph::IndexOptions {
        verbose: false,
        skip_patterns: vec![],
        on_progress: if auto { None } else { Some(&on_progress) },
    };

    match codegraph::index_project(&dir, if auto { None } else { Some(&opts) }) {
        Ok(result) if result.success => {
            if !auto {
                println!(
                    "  {} CodeGraph  {}",
                    colors::CHECK.green(),
                    "indexed".dimmed()
                );
                println!();
                println!("  {}{}", colors::CHECK.green(), "Project indexed.".green());
                println!();
            }
            0
        }
        _ => {
            if !auto {
                println!(
                    "  {} CodeGraph  {}",
                    colors::CROSS.red(),
                    "failed to index".dimmed()
                );
                println!();
            }
            1
        }
    }
}
