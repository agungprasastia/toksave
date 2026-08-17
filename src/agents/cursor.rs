use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::{Result, ToksaveError};
use crate::util::json::{
    add_to_array_if_missing, get_or_create_object, read_json_file, remove_from_array,
    write_json_file, write_json_pruned,
};
use crate::util::paths::{cursor_desktop_paths, cursor_known_bin_dirs, cursor_paths, toksave_abs};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};
use serde_json::{Value, json};

const RTK_MARKER: &str = "rtk-hook cursor";
const RTK_ALLOW: &str = "Shell(rtk *)";

pub struct CursorAgent;

impl CursorAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CursorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for CursorAgent {
    fn detect(&self) -> Detection {
        let p = cursor_paths();
        let bins = cursor_known_bin_dirs();
        let has_cli =
            find_binary_in("agent", &bins).is_some() || find_binary_in("cursor", &bins).is_some();
        let has_desktop = std::env::var("TOKSAVE_TEST").is_err()
            && cursor_desktop_paths().iter().any(|p| p.exists());
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
        let has_config = std::env::var("TOKSAVE_TEST").is_ok() && p.dir.exists();
        let has_mcp = p.mcp_config.exists();
        if has_config || has_mcp {
            Detection {
                installed: true,
                source: "config".to_string(),
            }
        } else {
            Detection {
                installed: false,
                source: String::new(),
            }
        }
    }

    fn wire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool> {
        if opts.dry_run {
            return Ok(true);
        }
        let p = cursor_paths();

        match tool {
            ToolId::Codegraph => {
                configure_mcp(
                    "codegraph",
                    vec!["runmcp", "--agent", "cursor", "codegraph", "serve", "--mcp"],
                )?;
                write_owner("cursor", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                configure_mcp(
                    "context-mode",
                    vec!["runmcp", "--agent", "cursor", "context-mode"],
                )?;
                write_owner("cursor", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("cursor", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if p.hooks_file.exists() {
                    let raw = std::fs::read_to_string(&p.hooks_file).unwrap_or_default();
                    if !raw.trim().is_empty() && serde_json::from_str::<Value>(&raw).is_err() {
                        return Err(ToksaveError::config(
                            &p.hooks_file.to_string_lossy(),
                            "Corrupted JSON in Cursor hooks file",
                        ));
                    }
                }
                let mut cfg = read_json_file(&p.hooks_file)?.unwrap_or_else(|| json!({}));
                let hook_entry = json!({
                    "command": format!("{} {RTK_MARKER}", toksave_abs()),
                    "matcher": "Shell"
                });
                merge_cursor_pretool(&mut cfg, hook_entry, RTK_MARKER);
                write_json_file(&p.hooks_file, &cfg)?;
                allow_shell_rtk()?;
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("cursor", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("cursor", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = cursor_paths();
        match tool {
            ToolId::Codegraph => {
                remove_mcp("codegraph")?;
                remove_owner("cursor", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                remove_mcp("context-mode")?;
                remove_owner("cursor", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("cursor", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if let Some(mut cfg) = read_json_file(&p.hooks_file)? {
                    remove_cursor_pretool(&mut cfg, RTK_MARKER);
                    write_json_pruned(&p.hooks_file, &cfg)?;
                }
                remove_shell_rtk()?;
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("cursor", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("cursor", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = cursor_paths();
        let cfg = read_json_file(&p.mcp_config).ok().flatten();
        match tool {
            ToolId::Codegraph => Some(
                cfg.as_ref()
                    .and_then(|c| c.get("mcpServers"))
                    .and_then(|m| m.get("codegraph"))
                    .is_some(),
            ),
            ToolId::ContextMode => Some(
                cfg.as_ref()
                    .and_then(|c| c.get("mcpServers"))
                    .and_then(|m| m.get("context-mode"))
                    .is_some(),
            ),
            ToolId::Caveman => Some(has_owner("cursor", "caveman")),
            ToolId::Rtk => {
                let hcfg = read_json_file(&p.hooks_file).ok().flatten();
                Some(
                    hcfg.as_ref()
                        .is_some_and(|c| has_cursor_pretool(c, RTK_MARKER)),
                )
            }
            ToolId::Ponytail => Some(has_owner("cursor", "ponytail")),
            ToolId::Principles => Some(has_owner("cursor", "principles")),
        }
    }
}

fn configure_mcp(tool_id: &str, args: Vec<&str>) -> Result<()> {
    let p = cursor_paths();
    let mut cfg = read_json_file(&p.mcp_config)?.unwrap_or_else(|| json!({}));
    let servers = get_or_create_object(&mut cfg, "mcpServers");
    servers[tool_id] = json!({
        "command": toksave_abs(),
        "args": args
    });
    write_json_file(&p.mcp_config, &cfg)
}

fn remove_mcp(tool_id: &str) -> Result<()> {
    let p = cursor_paths();
    if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
        if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
            mcp.remove(tool_id);
        }
        write_json_pruned(&p.mcp_config, &cfg)?;
    }
    Ok(())
}

fn allow_shell_rtk() -> Result<()> {
    let p = cursor_paths();
    let mut cfg = read_json_file(&p.cli_config)?.unwrap_or_else(|| json!({}));
    let perms = get_or_create_object(&mut cfg, "permissions");
    let allow = perms
        .as_object_mut()
        .expect("permissions object")
        .entry("allow")
        .or_insert_with(|| json!([]));
    let allow = allow.as_array_mut().expect("allow array");
    add_to_array_if_missing(allow, json!(RTK_ALLOW));
    write_json_file(&p.cli_config, &cfg)
}

fn remove_shell_rtk() -> Result<()> {
    let p = cursor_paths();
    let Some(mut cfg) = read_json_file(&p.cli_config)? else {
        return Ok(());
    };
    if let Some(perms) = cfg.get_mut("permissions").and_then(|p| p.as_object_mut()) {
        if let Some(allow) = perms.get_mut("allow").and_then(|a| a.as_array_mut()) {
            remove_from_array(allow, &json!(RTK_ALLOW));
        }
        if perms
            .get("allow")
            .and_then(|a| a.as_array())
            .is_some_and(|a| a.is_empty())
        {
            perms.remove("allow");
        }
        if perms.is_empty() {
            cfg.as_object_mut().expect("object").remove("permissions");
        }
    }
    write_json_pruned(&p.cli_config, &cfg)
}

fn is_managed_cursor_hook(entry: &Value, marker: &str) -> bool {
    entry
        .get("command")
        .and_then(|c| c.as_str())
        .is_some_and(|c| c.contains(marker))
}

fn merge_cursor_pretool(cfg: &mut Value, entry: Value, marker: &str) {
    if !cfg.is_object() {
        *cfg = json!({});
    }
    if cfg.get("version").is_none() {
        cfg["version"] = json!(1);
    }
    let hooks = get_or_create_object(cfg, "hooks");
    let arr = hooks
        .as_object_mut()
        .expect("hooks object")
        .entry("preToolUse")
        .or_insert_with(|| json!([]));
    let Some(items) = arr.as_array_mut() else {
        return;
    };
    items.retain(|e| !is_managed_cursor_hook(e, marker));
    items.push(entry);
}

fn remove_cursor_pretool(cfg: &mut Value, marker: &str) {
    let Some(hooks) = cfg.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return;
    };
    let Some(arr) = hooks.get_mut("preToolUse").and_then(|v| v.as_array_mut()) else {
        return;
    };
    arr.retain(|e| !is_managed_cursor_hook(e, marker));
    if arr.is_empty() {
        hooks.remove("preToolUse");
    }
    if hooks.is_empty() {
        cfg.as_object_mut().expect("object").remove("hooks");
    }
    if cfg.get("hooks").is_none()
        && cfg.get("version") == Some(&json!(1))
        && cfg.as_object().is_some_and(|o| o.len() == 1)
    {
        cfg.as_object_mut().expect("object").remove("version");
    }
}

fn has_cursor_pretool(cfg: &Value, marker: &str) -> bool {
    cfg.get("hooks")
        .and_then(|h| h.get("preToolUse"))
        .and_then(|v| v.as_array())
        .is_some_and(|arr| arr.iter().any(|e| is_managed_cursor_hook(e, marker)))
}
