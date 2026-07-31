use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::errors::Result;
use crate::util::json::{get_or_create_object, read_json_file, write_json_file};
use crate::util::paths::{droid_known_bin_dirs, droid_paths, toksave_abs};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

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
        let has_cli = droid_known_bin_dirs()
            .iter()
            .any(|d| d.join("droid").exists());
        let has_config = p.dir.exists();
        if has_cli {
            Detection {
                installed: true,
                source: "cli".to_string(),
            }
        } else if has_config {
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
                cfg["PreToolUse"] = serde_json::json!([hook_entry]);
                write_json_file(&p.hooks_file, &cfg)?;
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
                    write_json_file(&p.mcp_config, &cfg)?;
                }
                remove_owner("droid", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("context-mode");
                    }
                    write_json_file(&p.mcp_config, &cfg)?;
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
                    if let Some(obj) = cfg.as_object_mut() {
                        obj.remove("PreToolUse");
                    }
                    write_json_file(&p.hooks_file, &cfg)?;
                }
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
                Some(hcfg.as_ref().and_then(|c| c.get("PreToolUse")).is_some())
            }
            ToolId::Ponytail => Some(has_owner("droid", "ponytail")),
            ToolId::Principles => Some(has_owner("droid", "principles")),
        }
    }
}
