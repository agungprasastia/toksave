use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{
    add_to_array_if_missing, get_or_create_object, read_json_file, remove_from_array,
    write_json_file,
};
use crate::util::paths::{
    claude_desktop_paths, claude_known_bin_dirs, claude_paths, read_file, toksave_abs, write_file,
};
use std::path::Path;

pub struct ClaudeAgent;

impl Agent for ClaudeAgent {
    fn detect(&self) -> Detection {
        let has_cli = find_binary_in("claude", &claude_known_bin_dirs()).is_some();
        let has_desktop = claude_desktop_paths().iter().any(|p| p.exists());
        if has_cli && has_desktop {
            return Detection {
                installed: true,
                source: "cli+desktop".to_string(),
            };
        }
        if has_cli {
            return Detection {
                installed: true,
                source: "cli".to_string(),
            };
        }
        if has_desktop {
            return Detection {
                installed: true,
                source: "desktop".to_string(),
            };
        }
        // Config-dir fallback only in test mode (mirror TS NODE_ENV==="test")
        if std::env::var("TOKSAVE_TEST").is_ok() && claude_paths().dir.exists() {
            return Detection {
                installed: true,
                source: "config".to_string(),
            };
        }
        Detection {
            installed: false,
            source: String::new(),
        }
    }

    fn wire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool> {
        match tool {
            ToolId::Rtk => {
                if !opts.dry_run {
                    allow_bash_pattern("Bash(rtk *)")?;
                    wire_rtk_hook()?;
                    override_claude_rtk_hook()?;
                }
                Ok(true)
            }
            ToolId::Codegraph => {
                if !opts.dry_run {
                    wire_mcp("codegraph", vec!["runmcp", "codegraph", "serve", "--mcp"])?;
                    crate::util::unified_block::write_owner("claude", "codegraph")?;
                }
                Ok(true)
            }
            ToolId::ContextMode => {
                if !opts.dry_run {
                    wire_mcp("context-mode", vec!["runmcp", "context-mode"])?;
                    crate::util::unified_block::write_owner("claude", "context-mode")?;
                }
                Ok(true)
            }
            ToolId::Caveman => {
                if !opts.dry_run {
                    crate::util::unified_block::write_owner("claude", "caveman")?;
                }
                Ok(true)
            }
            ToolId::Ponytail => {
                if !opts.dry_run {
                    crate::util::unified_block::write_owner("claude", "ponytail")?;
                }
                Ok(true)
            }
            ToolId::Principles => {
                if !opts.dry_run {
                    crate::util::unified_block::write_owner("claude", "principles")?;
                }
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool> {
        match tool {
            ToolId::Rtk => {
                if !opts.dry_run {
                    remove_rtk_hook()?;
                }
                Ok(true)
            }
            ToolId::Codegraph => {
                if !opts.dry_run {
                    remove_mcp("codegraph")?;
                    crate::util::unified_block::remove_owner("claude", "codegraph")?;
                }
                Ok(true)
            }
            ToolId::ContextMode => {
                if !opts.dry_run {
                    remove_mcp("context-mode")?;
                    crate::util::unified_block::remove_owner("claude", "context-mode")?;
                }
                Ok(true)
            }
            ToolId::Caveman => {
                if !opts.dry_run {
                    crate::util::unified_block::remove_owner("claude", "caveman")?;
                }
                Ok(true)
            }
            ToolId::Ponytail => {
                if !opts.dry_run {
                    crate::util::unified_block::remove_owner("claude", "ponytail")?;
                }
                Ok(true)
            }
            ToolId::Principles => {
                if !opts.dry_run {
                    crate::util::unified_block::remove_owner("claude", "principles")?;
                }
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        match tool {
            ToolId::Rtk => Some(has_rtk_hook()),
            ToolId::Codegraph => Some(has_mcp("codegraph")),
            ToolId::ContextMode => Some(has_mcp("context-mode")),
            ToolId::Caveman => Some(crate::util::unified_block::has_owner("claude", "caveman")),
            ToolId::Ponytail => Some(crate::util::unified_block::has_owner("claude", "ponytail")),
            ToolId::Principles => Some(crate::util::unified_block::has_owner(
                "claude",
                "principles",
            )),
        }
    }
}

fn allow_bash_pattern(pattern: &str) -> Result<()> {
    let p = claude_paths();
    let cfg = read_json_file(&p.settings)?.unwrap_or_else(|| serde_json::json!({}));
    let mut cfg = cfg;
    {
        let perms = get_or_create_object(&mut cfg, "permissions");
        let perms = perms.as_object_mut().expect("object");
        let arr = perms
            .entry("allow")
            .or_insert_with(|| serde_json::json!([]));
        let arr = arr.as_array_mut().expect("array");
        add_to_array_if_missing(arr, serde_json::json!(pattern));
    }
    write_json_file(&p.settings, &cfg)
}

fn rtk_hook_command() -> String {
    format!("{} rtk-hook claude", toksave_abs())
}

fn wire_rtk_hook() -> Result<()> {
    let p = claude_paths();
    let cfg = read_json_file(&p.settings)?.unwrap_or_else(|| serde_json::json!({}));
    let mut cfg = cfg;
    let command = rtk_hook_command();
    {
        let hooks = get_or_create_object(&mut cfg, "hooks");
        let hooks = hooks.as_object_mut().expect("object");
        let arr = hooks
            .entry("PreToolUse")
            .or_insert_with(|| serde_json::json!([]));
        let arr = arr.as_array_mut().expect("array");
        let entry = serde_json::json!({
            "matcher": "Bash",
            "hooks": [{ "type": "command", "command": command, "timeout": 10 }]
        });
        if !arr.iter().any(|g| hook_group_has_command(g, &command)) {
            arr.push(entry);
        }
    }
    write_json_file(&p.settings, &cfg)
}

fn hook_group_has_command(group: &serde_json::Value, command: &str) -> bool {
    group
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks
                .iter()
                .any(|h| h.get("command").and_then(|c| c.as_str()) == Some(command))
        })
        .unwrap_or(false)
}

fn remove_rtk_hook() -> Result<()> {
    let p = claude_paths();
    let cfg = read_json_file(&p.settings)?.unwrap_or_else(|| serde_json::json!({}));
    let mut cfg = cfg;
    if let Some(hooks) = cfg.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        if let Some(pre) = hooks.get_mut("PreToolUse").and_then(|p| p.as_array_mut()) {
            let marker = "rtk-hook claude";
            pre.retain(|g| !hook_group_contains_marker(g, marker));
            if pre.is_empty() {
                hooks.remove("PreToolUse");
            }
        }
        if hooks.is_empty() {
            cfg.as_object_mut().expect("object").remove("hooks");
        }
    }
    if let Some(perms) = cfg.get_mut("permissions").and_then(|p| p.as_object_mut()) {
        if let Some(allow) = perms.get_mut("allow").and_then(|a| a.as_array_mut()) {
            remove_from_array(allow, &serde_json::json!("Bash(rtk *)"));
            if allow.is_empty() {
                perms.remove("allow");
            }
        }
        if perms.is_empty() {
            cfg.as_object_mut().expect("object").remove("permissions");
        }
    }
    write_json_file(&p.settings, &cfg)
}

fn hook_group_contains_marker(group: &serde_json::Value, marker: &str) -> bool {
    group
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains(marker))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn has_rtk_hook() -> bool {
    let p = claude_paths();
    let Ok(Some(cfg)) = read_json_file(&p.settings) else {
        return false;
    };
    let marker = "rtk-hook claude";
    cfg.get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().any(|g| hook_group_contains_marker(g, marker)))
        .unwrap_or(false)
}

/// Override rtk's own "rtk hook claude" command with the toksave wrapper, dedupe groups,
/// remove RTK.md, strip @RTK.md refs, and allow Bash(rtk *) (port of overrideClaudeRtkHook).
fn override_claude_rtk_hook() -> Result<()> {
    let p = claude_paths();
    let Some(raw) = read_file(&p.settings) else {
        return Ok(());
    };
    let parsed = serde_json::from_str::<serde_json::Value>(&raw);
    let mut cfg = match parsed {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    let tok = toksave_abs();
    let new_cmd = format!("{tok} rtk-hook claude");
    let mut changed = false;

    if let Some(hooks) = cfg.get_mut("hooks").and_then(|h| h.as_object_mut())
        && let Some(pre) = hooks.get_mut("PreToolUse").and_then(|p| p.as_array_mut())
    {
        for g in pre.iter_mut() {
            let Some(inner) = g.get_mut("hooks").and_then(|h| h.as_array_mut()) else {
                continue;
            };
            for h in inner.iter_mut() {
                // Read command immutably first to avoid borrow conflict with *h = ...
                let should_replace = h
                    .get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("rtk hook claude") && !c.contains("rtk-hook claude"))
                    .unwrap_or(false);
                if should_replace {
                    *h =
                        serde_json::json!({ "type": "command", "command": new_cmd, "timeout": 10 });
                    changed = true;
                }
            }
        }
        // Deduplicate groups with same first hook command
        if pre.len() > 1 {
            let mut seen = std::collections::HashSet::new();
            let mut dedup: Vec<serde_json::Value> = Vec::new();
            for g in pre.iter() {
                let first = first_hook_command(g);
                if seen.insert(first.clone()) {
                    dedup.push(g.clone());
                } else {
                    changed = true;
                }
            }
            *pre = dedup;
        }
    }

    if changed {
        write_json_file(&p.settings, &cfg)?;
    }
    allow_bash_pattern("Bash(rtk *)")?;

    // Remove RTK.md + strip @RTK.md ref from AGENTS.md
    let rtk_md = p.dir.join("RTK.md");
    if rtk_md.exists() {
        let _ = std::fs::remove_file(&rtk_md);
    }
    strip_rtk_ref_from_md(&p.agents_md);
    Ok(())
}

fn first_hook_command(g: &serde_json::Value) -> String {
    g.get("hooks")
        .and_then(|h| h.as_array())
        .and_then(|arr| arr.first())
        .and_then(|h| h.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string()
}

fn strip_rtk_ref_from_md(file_path: &Path) {
    let Some(raw) = read_file(file_path) else {
        return;
    };
    let kept: Vec<&str> = raw
        .split('\n')
        .filter(|l| {
            let t = l.trim();
            !(t.starts_with('@') && t.ends_with("RTK.md"))
        })
        .collect();
    let result = kept.join("\n").trim().to_string();
    if result.is_empty() {
        let _ = std::fs::remove_file(file_path);
        return;
    }
    if result != raw.trim() {
        let _ = write_file(file_path, &format!("{result}\n"));
    }
}

fn wire_mcp(name: &str, args: Vec<&str>) -> Result<()> {
    let p = claude_paths();
    let mut cfg = read_json_file(&p.global_json)?.unwrap_or_else(|| serde_json::json!({}));
    let servers = get_or_create_object(&mut cfg, "mcpServers");
    let cmd_args: Vec<serde_json::Value> = args.iter().map(|a| serde_json::json!(a)).collect();
    servers[name] = serde_json::json!({
        "command": toksave_abs(),
        "args": cmd_args
    });
    write_json_file(&p.global_json, &cfg)
}

fn remove_mcp(name: &str) -> Result<()> {
    let p = claude_paths();
    if let Some(mut cfg) = read_json_file(&p.global_json)? {
        if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
            mcp.remove(name);
        }
        write_json_file(&p.global_json, &cfg)?;
    }
    Ok(())
}

fn has_mcp(name: &str) -> bool {
    let p = claude_paths();
    let cfg = read_json_file(&p.global_json).ok().flatten();
    cfg.as_ref()
        .and_then(|c| c.get("mcpServers"))
        .and_then(|m| m.get(name))
        .is_some()
}
