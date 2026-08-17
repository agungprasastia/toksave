use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{get_or_create_object, read_json_file, write_json_file, write_json_pruned};
use crate::util::paths::{
    droid_desktop_paths, droid_known_bin_dirs, droid_legacy_hooks_file, droid_paths, toksave_abs,
};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

/// Scrub a stale RTK hook entry from the pre-fix `~/.factory-droid/hooks.json` location.
/// Best-effort: a corrupted legacy file is left alone rather than failing the whole call.
fn cleanup_legacy_droid_hooks() {
    let path = droid_legacy_hooks_file();
    if !path.exists() {
        return;
    }
    let Ok(Some(mut cfg)) = read_json_file(&path) else {
        return;
    };
    let before = cfg.clone();
    crate::util::json::remove_pretool_use(&mut cfg, "rtk-hook droid");
    if cfg != before {
        let _ = write_json_pruned(&path, &cfg);
    }
}

pub struct DroidAgent;

impl DroidAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DroidAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for DroidAgent {
    fn detect(&self) -> Detection {
        let p = droid_paths();
        let has_cli = find_binary_in("droid", &droid_known_bin_dirs()).is_some();
        let has_desktop = droid_desktop_paths().iter().any(|p| p.exists());
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
        if std::env::var("TOKSAVE_TEST").is_ok() && p.dir.exists() {
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
        if opts.dry_run {
            return Ok(true);
        }
        let p = droid_paths();

        match tool {
            ToolId::Codegraph => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["codegraph"] = serde_json::json!({
                    "command": toksave_abs(),
                    "args": ["runmcp", "--agent", "droid", "codegraph", "serve", "--mcp"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("droid", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["context-mode"] = serde_json::json!({
                    "command": toksave_abs(),
                    "args": ["runmcp", "--agent", "droid", "context-mode"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("droid", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("droid", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let mut cfg =
                    read_json_file(&p.hooks_file)?.unwrap_or_else(|| serde_json::json!({}));
                let hook_entry = serde_json::json!({
                    "matcher": "Execute",
                    "hooks": [{ "type": "command", "command": format!("{} rtk-hook droid", toksave_abs()), "timeout": 10 }]
                });
                crate::util::json::merge_pretool_use(&mut cfg, hook_entry, "rtk-hook droid");
                write_json_file(&p.hooks_file, &cfg)?;
                cleanup_legacy_droid_hooks();
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("droid", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("droid", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = droid_paths();
        match tool {
            ToolId::Codegraph => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("codegraph");
                    }
                    write_json_pruned(&p.mcp_config, &cfg)?;
                }
                remove_owner("droid", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("context-mode");
                    }
                    write_json_pruned(&p.mcp_config, &cfg)?;
                }
                remove_owner("droid", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("droid", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if let Some(mut cfg) = read_json_file(&p.hooks_file)? {
                    crate::util::json::remove_pretool_use(&mut cfg, "rtk-hook droid");
                    write_json_pruned(&p.hooks_file, &cfg)?;
                }
                cleanup_legacy_droid_hooks();
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("droid", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("droid", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = droid_paths();
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
            ToolId::Caveman => Some(has_owner("droid", "caveman")),
            ToolId::Rtk => {
                let hcfg = read_json_file(&p.hooks_file).ok().flatten();
                Some(hcfg.as_ref().is_some_and(|c| {
                    crate::util::json::has_pretool_with_command_marker(c, "rtk-hook droid")
                }))
            }
            ToolId::Ponytail => Some(has_owner("droid", "ponytail")),
            ToolId::Principles => Some(has_owner("droid", "principles")),
        }
    }
}
