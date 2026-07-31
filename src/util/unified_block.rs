use crate::content::agent_instructions;
use crate::util::errors::Result;
use crate::util::paths::{
    antigravity_paths, claude_paths, codex_paths, copilot_paths, devin_paths, droid_paths,
    opencode_paths, read_file, warp_paths, write_file,
};
use std::fs;
use std::path::PathBuf;

pub fn instruction_path(agent: &str) -> Option<PathBuf> {
    match agent {
        "claude" => Some(claude_paths().agents_md),
        "opencode" => Some(opencode_paths().agents_md),
        "codex" => Some(codex_paths().instructions),
        "antigravity" => Some(antigravity_paths().dir.join("AGENTS.md")),
        "copilot" => Some(copilot_paths().instructions),
        "droid" => Some(droid_paths().dir.join("instructions.md")),
        "devin" => Some(devin_paths().dir.join("instructions.md")),
        "warp" => Some(warp_paths().dir.join("instructions.md")),
        _ => None,
    }
}

pub fn write_owner(agent: &str, owner: &str) -> Result<bool> {
    let Some(path) = instruction_path(agent) else {
        return Ok(false);
    };
    let _cur = read_file(&path).unwrap_or_default();
    // Simplified unified block rendering using managed content agent_instructions
    let mut header = format!("<!-- TOKSAVE:{owner}:START -->\n");
    header.push_str(agent_instructions::agent_instructions());
    header.push_str(&format!("\n<!-- TOKSAVE:{owner}:END -->\n"));
    write_file(&path, &header)?;
    Ok(true)
}

pub fn remove_owner(agent: &str, owner: &str) -> Result<bool> {
    let Some(path) = instruction_path(agent) else {
        return Ok(false);
    };
    if path.exists() {
        let content = read_file(&path).unwrap_or_default();
        let start_tag = format!("<!-- TOKSAVE:{owner}:START -->");
        let end_tag = format!("<!-- TOKSAVE:{owner}:END -->");
        if content.contains(&start_tag) {
            let mut new_content = String::new();
            let mut skipping = false;
            for line in content.lines() {
                if line.contains(&start_tag) {
                    skipping = true;
                    continue;
                }
                if line.contains(&end_tag) {
                    skipping = false;
                    continue;
                }
                if !skipping {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            if new_content.trim().is_empty() {
                let _ = fs::remove_file(&path);
            } else {
                write_file(&path, &new_content)?;
            }
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn has_owner(agent: &str, owner: &str) -> bool {
    let Some(path) = instruction_path(agent) else {
        return false;
    };
    if let Some(content) = read_file(&path) {
        let tag = format!("TOKSAVE:{owner}");
        return content.contains(&tag) || content.contains(owner);
    }
    false
}
