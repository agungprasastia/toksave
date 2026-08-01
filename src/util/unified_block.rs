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

/// Parse the owner list out of a block header like
/// `<!-- TOKSAVE:codegraph,principles:START -->`.
fn parse_owners(header: &str) -> Vec<String> {
    header
        .split("TOKSAVE:")
        .nth(1)
        .and_then(|s| s.split(":START").next())
        .map(|o| {
            o.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// One managed block per instruction file, carrying every wired tool in the
/// header (`TOKSAVE:a,b:START`). Writes upsert the block body and consolidate
/// the owner list; surrounding user content is preserved.
pub fn write_owner(agent: &str, owner: &str) -> Result<bool> {
    let Some(path) = instruction_path(agent) else {
        return Ok(false);
    };
    let existing = read_file(&path).unwrap_or_default();
    let block = agent_instructions::agent_instructions();
    let lines: Vec<&str> = existing.lines().collect();
    let start_idx = lines
        .iter()
        .position(|l| l.contains("TOKSAVE:") && l.contains(":START -->"));
    let end_idx = lines
        .iter()
        .rposition(|l| l.contains("TOKSAVE:") && l.contains(":END -->"));
    let mut out: Vec<String> = vec![];

    match (start_idx, end_idx) {
        (Some(s), Some(e)) if s <= e => {
            let mut owners = parse_owners(lines[s]);
            if !owners.iter().any(|o| o == owner) {
                owners.push(owner.to_string());
            }
            out.extend(lines[..s].iter().map(|l| l.to_string()));
            out.push(format!("<!-- TOKSAVE:{}:START -->", owners.join(",")));
            out.push(block.to_string());
            out.push("<!-- TOKSAVE:END -->".to_string());
            out.extend(lines[e + 1..].iter().map(|l| l.to_string()));
        }
        _ => {
            if !existing.trim().is_empty() {
                out.push(String::new());
            }
            out.push(format!("<!-- TOKSAVE:{owner}:START -->"));
            out.push(block.to_string());
            out.push("<!-- TOKSAVE:END -->".to_string());
        }
    }
    let content = ensure_separators(&strip_legacy_fences(&out.join("\n")));
    write_file(&path, &content)?;
    Ok(true)
}

pub fn remove_owner(agent: &str, owner: &str) -> Result<bool> {
    let Some(path) = instruction_path(agent) else {
        return Ok(false);
    };
    if path.exists() {
        let content = read_file(&path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let Some(start_idx) = lines
            .iter()
            .position(|l| l.contains("TOKSAVE:") && l.contains(":START -->"))
        else {
            return Ok(false);
        };
        let Some(end_idx) = lines
            .iter()
            .rposition(|l| l.contains("TOKSAVE:") && l.contains(":END -->"))
        else {
            return Ok(false);
        };
        let mut owners = parse_owners(lines[start_idx]);
        owners.retain(|o| o != owner);
        if owners.is_empty() {
            let mut new_lines: Vec<&str> = lines[..start_idx].to_vec();
            new_lines.extend_from_slice(&lines[end_idx + 1..]);
            let content = ensure_separators(&strip_legacy_fences(&new_lines.join("\n")));
            if content.trim().is_empty() {
                let _ = fs::remove_file(&path);
            } else {
                write_file(&path, &content)?;
            }
        } else {
            let header = format!("<!-- TOKSAVE:{}:START -->", owners.join(","));
            let mut new_lines: Vec<String> =
                lines[..start_idx].iter().map(|l| l.to_string()).collect();
            new_lines.push(header);
            new_lines.extend(lines[start_idx + 1..].iter().map(|l| l.to_string()));
            let content = ensure_separators(&strip_legacy_fences(&new_lines.join("\n")));
            write_file(&path, &content)?;
        }
        return Ok(true);
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

/// Fence pairs left behind by earlier tooling (tokless-era markers). Blocks
/// between a start/end pair are dropped entirely.
const LEGACY_FENCES: &[(&str, &str)] = &[
    ("<!-- caveman-begin -->", "<!-- caveman-end -->"),
    ("<!-- CODEGRAPH_START -->", "<!-- CODEGRAPH_END -->"),
    ("<!-- CONTEXT-MODE_START -->", "<!-- CONTEXT-MODE_END -->"),
];

/// Remove legacy fence blocks and stray `tokless:owners` config lines.
pub fn strip_legacy_fences(content: &str) -> String {
    if !(content.contains("caveman-begin")
        || content.contains("CODEGRAPH_START")
        || content.contains("CONTEXT-MODE_START")
        || content.contains("tokless:owners"))
    {
        return content.to_string();
    }
    let mut out: Vec<&str> = vec![];
    let mut skipping = false;
    let mut end_marker: Option<&str> = None;
    for line in content.lines() {
        if skipping {
            if let Some(end) = end_marker {
                if line.contains(end) {
                    skipping = false;
                    end_marker = None;
                }
            }
            continue;
        }
        if line.contains("tokless:owners") {
            continue;
        }
        for (start, end) in LEGACY_FENCES {
            if line.contains(start) {
                skipping = true;
                end_marker = Some(end);
                break;
            }
        }
        if !skipping {
            out.push(line);
        }
    }
    out.join("\n")
}

/// Ensure exactly one blank line follows each instruction marker line, so
/// managed blocks stay separated from surrounding content (port of tokless
/// EnsureInstructionSeparators, line-based).
pub fn ensure_separators(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<String> = vec![];
    for (i, line) in lines.iter().enumerate() {
        out.push(line.to_string());
        let is_marker = line.contains("<!--") && line.contains("-->");
        let next_is_blank = lines
            .get(i + 1)
            .map(|l| l.trim().is_empty())
            .unwrap_or(true);
        if is_marker && !next_is_blank {
            out.push(String::new());
        }
    }
    let mut out = out.join("\n");
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_legacy_fences_removes_all_pairs() {
        let src = concat!(
            "head\n<!-- caveman-begin -->\ndeleted\n<!-- caveman-end -->\n",
            "<!-- CODEGRAPH_START -->\nmore\n<!-- CODEGRAPH_END -->\n",
            "<!-- CONTEXT-MODE_START -->\nmore\n<!-- CONTEXT-MODE_END -->\n",
            "<!-- tokless:owners=claude -->\ntail"
        );
        assert_eq!(strip_legacy_fences(src), "head\ntail");
    }

    #[test]
    fn strip_legacy_fences_noop_without_markers() {
        let src = "head\n<!-- TOKSAVE:x:START -->\nbody\n<!-- TOKSAVE:x:END -->";
        assert_eq!(strip_legacy_fences(src), src);
    }

    #[test]
    fn ensure_separators_blank_between_blocks() {
        let src = "<!-- A:END -->\n<!-- B:START -->\nbody";
        assert_eq!(
            ensure_separators(src),
            "<!-- A:END -->\n\n<!-- B:START -->\n\nbody"
        );
    }

    #[test]
    fn ensure_separators_keeps_existing_blank_and_trims_tail() {
        let src = "<!-- A:END -->\n\nbody\n<!-- B:END -->";
        assert_eq!(
            ensure_separators(src),
            "<!-- A:END -->\n\nbody\n<!-- B:END -->"
        );
    }

    #[test]
    fn parse_owners_splits_header_list() {
        assert_eq!(
            parse_owners("<!-- TOKSAVE:codegraph, principles:START -->"),
            vec!["codegraph", "principles"]
        );
        assert_eq!(
            parse_owners("<!-- TOKSAVE:claude:START -->"),
            vec!["claude"]
        );
        assert!(parse_owners("").is_empty());
    }

    #[test]
    fn write_owner_consolidates_owners_and_keeps_surroundings() {
        let tmp = std::env::temp_dir().join("toksave-block-test");
        let claude_dir = tmp.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let path = claude_dir.join("AGENTS.md");
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", &tmp);
        let block = agent_instructions::agent_instructions();
        let mut first = String::from("# My file\n\nuser content\n\n");
        first.push_str("<!-- TOKSAVE:codegraph:START -->\n");
        first.push_str(block);
        first.push_str("\n<!-- TOKSAVE:END -->\n");
        write_file(&path, &first).unwrap();

        write_owner("claude", "principles").unwrap();
        let out = read_file(&path).unwrap();
        assert!(out.starts_with("# My file\n\nuser content"));
        assert!(out.contains("TOKSAVE:codegraph,principles:START"));
        assert!(out.contains("<!-- TOKSAVE:END -->"));

        remove_owner("claude", "codegraph").unwrap();
        let out = read_file(&path).unwrap();
        assert!(out.contains("TOKSAVE:principles:START"));
        assert!(!out.contains("codegraph,"));

        remove_owner("claude", "principles").unwrap();
        let out = read_file(&path).unwrap();
        assert!(!out.contains("TOKSAVE:"));
        assert!(out.contains("user content"));

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        } else {
            std::env::remove_var("HOME");
        }
        std::fs::remove_dir_all(&tmp).ok();
    }
}
