use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{get_or_create_object, read_json_file, write_json_file};
use crate::util::paths::{devin_desktop_paths, devin_known_bin_dirs, devin_paths, toksave_abs};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

pub struct DevinAgent;

impl DevinAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DevinAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for DevinAgent {
    fn detect(&self) -> Detection {
        let p = devin_paths();
        let has_cli = find_binary_in("devin", &devin_known_bin_dirs()).is_some();
        let has_desktop = devin_desktop_paths().iter().any(|p| p.exists());
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
        let p = devin_paths();

        match tool {
            ToolId::Codegraph => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["codegraph"] = serde_json::json!({
                    "command": toksave_abs(),
                    "args": ["runmcp", "--agent", "devin", "codegraph", "serve", "--mcp"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("devin", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["context-mode"] = serde_json::json!({
                    "command": toksave_abs(),
                    "args": ["runmcp", "--agent", "devin", "context-mode"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("devin", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("devin", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let mut cfg =
                    read_json_file(&p.hooks_file)?.unwrap_or_else(|| serde_json::json!({}));
                let hook_entry = serde_json::json!({
                    "matcher": "Execute",
                    "hooks": [{ "type": "command", "command": format!("{} rtk-hook devin", toksave_abs()), "timeout": 10 }]
                });
                crate::util::json::merge_pretool_use(&mut cfg, hook_entry, "rtk-hook devin");
                write_json_file(&p.hooks_file, &cfg)?;
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("devin", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("devin", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = devin_paths();
        match tool {
            ToolId::Codegraph => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("codegraph");
                    }
                    write_json_file(&p.mcp_config, &cfg)?;
                }
                remove_owner("devin", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("context-mode");
                    }
                    write_json_file(&p.mcp_config, &cfg)?;
                }
                remove_owner("devin", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("devin", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if let Some(mut cfg) = read_json_file(&p.hooks_file)? {
                    crate::util::json::remove_pretool_use(&mut cfg, "rtk-hook devin");
                    write_json_file(&p.hooks_file, &cfg)?;
                }
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("devin", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("devin", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = devin_paths();
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
            ToolId::Caveman => Some(has_owner("devin", "caveman")),
            ToolId::Rtk => {
                let hcfg = read_json_file(&p.hooks_file).ok().flatten();
                Some(hcfg.as_ref().is_some_and(|c| {
                    crate::util::json::has_pretool_with_command_marker(c, "rtk-hook devin")
                }))
            }
            ToolId::Ponytail => Some(has_owner("devin", "ponytail")),
            ToolId::Principles => Some(has_owner("devin", "principles")),
        }
    }
}
