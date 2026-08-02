use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::{Result, ToksaveError};
use crate::util::json::{get_or_create_object, read_json_file, write_json_file};
use crate::util::paths::{toksave_abs, warp_desktop_paths, warp_known_bin_dirs, warp_paths};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

pub struct WarpAgent;

impl WarpAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WarpAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for WarpAgent {
    fn detect(&self) -> Detection {
        let p = warp_paths();
        let has_cli = find_binary_in("warp", &warp_known_bin_dirs()).is_some();
        let has_desktop = warp_desktop_paths().iter().any(|p| p.exists());
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
        let p = warp_paths();

        match tool {
            ToolId::Codegraph => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["codegraph"] = serde_json::json!({
                    "command": toksave_abs(),
                    "args": ["runmcp", "--agent", "warp", "codegraph", "serve", "--mcp"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("warp", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["context-mode"] = serde_json::json!({
                    "command": toksave_abs(),
                    "args": ["runmcp", "--agent", "warp", "context-mode"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("warp", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("warp", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                // Trust boundary: if hooks_file exists but is corrupted, read_json_file will return ToksaveError::Config
                if p.hooks_file.exists() {
                    let raw = std::fs::read_to_string(&p.hooks_file).unwrap_or_default();
                    if !raw.trim().is_empty()
                        && serde_json::from_str::<serde_json::Value>(&raw).is_err()
                    {
                        return Err(ToksaveError::config(
                            &p.hooks_file.to_string_lossy(),
                            "Corrupted JSON in Warp hooks file",
                        ));
                    }
                }
                let mut cfg =
                    read_json_file(&p.hooks_file)?.unwrap_or_else(|| serde_json::json!({}));
                let hook_entry = serde_json::json!({
                    "matcher": "Execute",
                    "hooks": [{ "type": "command", "command": format!("{} rtk-hook warp", toksave_abs()), "timeout": 10 }]
                });
                crate::util::json::merge_pretool_use(&mut cfg, hook_entry, "rtk-hook warp");
                write_json_file(&p.hooks_file, &cfg)?;
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("warp", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("warp", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = warp_paths();
        match tool {
            ToolId::Codegraph => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("codegraph");
                    }
                    write_json_file(&p.mcp_config, &cfg)?;
                }
                remove_owner("warp", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("context-mode");
                    }
                    write_json_file(&p.mcp_config, &cfg)?;
                }
                remove_owner("warp", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("warp", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if let Some(mut cfg) = read_json_file(&p.hooks_file)? {
                    crate::util::json::remove_pretool_use(&mut cfg, "rtk-hook warp");
                    write_json_file(&p.hooks_file, &cfg)?;
                }
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("warp", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("warp", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = warp_paths();
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
            ToolId::Caveman => Some(has_owner("warp", "caveman")),
            ToolId::Rtk => {
                let hcfg = read_json_file(&p.hooks_file).ok().flatten();
                Some(hcfg.as_ref().is_some_and(|c| {
                    crate::util::json::has_pretool_with_command_marker(c, "rtk-hook warp")
                }))
            }
            ToolId::Ponytail => Some(has_owner("warp", "ponytail")),
            ToolId::Principles => Some(has_owner("warp", "principles")),
        }
    }
}
