use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{get_or_create_object, read_json_file, write_json_file, write_json_pruned};
use crate::util::paths::{copilot_known_bin_dirs, copilot_paths, toksave_abs};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

pub struct CopilotAgent;

impl CopilotAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CopilotAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for CopilotAgent {
    fn detect(&self) -> Detection {
        let p = copilot_paths();
        let has_cli = find_binary_in("copilot", &copilot_known_bin_dirs()).is_some();
        let has_config = std::env::var("TOKSAVE_TEST").is_ok() && p.dir.exists();
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
        let p = copilot_paths();

        match tool {
            ToolId::Codegraph => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["codegraph"] = serde_json::json!({
                    "type": "local",
                    "command": toksave_abs(),
                    "args": ["runmcp", "codegraph", "serve", "--mcp"],
                    "tools": ["*"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("copilot", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let mut cfg =
                    read_json_file(&p.mcp_config)?.unwrap_or_else(|| serde_json::json!({}));
                let servers = get_or_create_object(&mut cfg, "mcpServers");
                servers["context-mode"] = serde_json::json!({
                    "type": "local",
                    "command": toksave_abs(),
                    "args": ["runmcp", "context-mode"],
                    "tools": ["*"]
                });
                write_json_file(&p.mcp_config, &cfg)?;
                write_owner("copilot", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("copilot", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                // GitHub Copilot CLI's own hook schema (docs.github.com/en/copilot/reference/
                // hooks-reference): camelCase event key "preToolUse", each entry supports a
                // `matcher` regex tested against `toolName` ("bash" for the shell tool) and a
                // cross-platform `command` fallback field.
                let rtk_file = p.hooks_dir.join("toksave-rtk.json");
                let hook_content = serde_json::json!({
                    "version": 1,
                    "hooks": {
                        "preToolUse": [{
                            "type": "command",
                            "matcher": "bash",
                            "command": format!("{} rtk-hook copilot", toksave_abs()),
                            "timeoutSec": 10
                        }]
                    }
                });
                write_json_file(&rtk_file, &hook_content)?;
                let _ = std::fs::remove_file(p.hooks_dir.join("tokless-rtk.json"));
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("copilot", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("copilot", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = copilot_paths();
        match tool {
            ToolId::Codegraph => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("codegraph");
                    }
                    write_json_pruned(&p.mcp_config, &cfg)?;
                }
                remove_owner("copilot", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.mcp_config)? {
                    if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                        mcp.remove("context-mode");
                    }
                    write_json_pruned(&p.mcp_config, &cfg)?;
                }
                remove_owner("copilot", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("copilot", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let _ = std::fs::remove_file(p.hooks_dir.join("toksave-rtk.json"));
                let _ = std::fs::remove_file(p.hooks_dir.join("tokless-rtk.json"));
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("copilot", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("copilot", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = copilot_paths();
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
            ToolId::Caveman => Some(has_owner("copilot", "caveman")),
            ToolId::Rtk => Some(p.hooks_dir.join("toksave-rtk.json").exists()),
            ToolId::Ponytail => Some(has_owner("copilot", "ponytail")),
            ToolId::Principles => Some(has_owner("copilot", "principles")),
        }
    }
}
