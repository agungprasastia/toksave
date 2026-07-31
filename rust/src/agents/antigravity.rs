use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::errors::Result;
use crate::util::json::{get_or_create_object, read_json_file, write_json_file};
use crate::util::paths::{
    antigravity_known_bin_dirs, antigravity_mcp_files, antigravity_paths, toksave_abs,
};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

pub struct AntigravityAgent;

impl AntigravityAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AntigravityAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for AntigravityAgent {
    fn detect(&self) -> Detection {
        let p = antigravity_paths();
        let has_cli = antigravity_known_bin_dirs()
            .iter()
            .any(|d| d.join("agy").exists());
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
        let p = antigravity_paths();

        match tool {
            ToolId::Codegraph => {
                for mcp_file in antigravity_mcp_files() {
                    let mut cfg =
                        read_json_file(&mcp_file)?.unwrap_or_else(|| serde_json::json!({}));
                    let servers = get_or_create_object(&mut cfg, "mcpServers");
                    servers["codegraph"] = serde_json::json!({
                        "command": toksave_abs(),
                        "args": ["runmcp", "--agent", "antigravity", "codegraph", "serve", "--mcp"],
                        "trust": true
                    });
                    write_json_file(&mcp_file, &cfg)?;
                }
                write_owner("antigravity", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                for mcp_file in antigravity_mcp_files() {
                    let mut cfg =
                        read_json_file(&mcp_file)?.unwrap_or_else(|| serde_json::json!({}));
                    let servers = get_or_create_object(&mut cfg, "mcpServers");
                    servers["context-mode"] = serde_json::json!({
                        "command": toksave_abs(),
                        "args": ["runmcp", "--agent", "antigravity", "context-mode"],
                        "trust": true
                    });
                    write_json_file(&mcp_file, &cfg)?;
                }
                write_owner("antigravity", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("antigravity", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let mut cfg = read_json_file(&p.hooks)?.unwrap_or_else(|| serde_json::json!({}));
                cfg["rtk"] = serde_json::json!({
                    "PreToolUse": [{
                        "matcher": "^(Bash|run_command|execute_command|cmd|sh|pwsh)$",
                        "hooks": [{ "type": "command", "command": format!("{} rtk-hook agy", toksave_abs()), "timeout": 10 }]
                    }]
                });
                write_json_file(&p.hooks, &cfg)?;
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("antigravity", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("antigravity", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = antigravity_paths();
        match tool {
            ToolId::Codegraph => {
                for mcp_file in antigravity_mcp_files() {
                    if let Some(mut cfg) = read_json_file(&mcp_file)? {
                        if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut())
                        {
                            mcp.remove("codegraph");
                        }
                        write_json_file(&mcp_file, &cfg)?;
                    }
                }
                remove_owner("antigravity", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                for mcp_file in antigravity_mcp_files() {
                    if let Some(mut cfg) = read_json_file(&mcp_file)? {
                        if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut())
                        {
                            mcp.remove("context-mode");
                        }
                        write_json_file(&mcp_file, &cfg)?;
                    }
                }
                remove_owner("antigravity", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("antigravity", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if let Some(mut cfg) = read_json_file(&p.hooks)? {
                    if let Some(obj) = cfg.as_object_mut() {
                        obj.remove("rtk");
                    }
                    write_json_file(&p.hooks, &cfg)?;
                }
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("antigravity", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("antigravity", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = antigravity_paths();
        match tool {
            ToolId::Codegraph => {
                let file = antigravity_mcp_files().into_iter().next()?;
                let cfg = read_json_file(&file).ok().flatten();
                Some(
                    cfg.as_ref()
                        .and_then(|c| c.get("mcpServers"))
                        .and_then(|m| m.get("codegraph"))
                        .is_some(),
                )
            }
            ToolId::ContextMode => {
                let file = antigravity_mcp_files().into_iter().next()?;
                let cfg = read_json_file(&file).ok().flatten();
                Some(
                    cfg.as_ref()
                        .and_then(|c| c.get("mcpServers"))
                        .and_then(|m| m.get("context-mode"))
                        .is_some(),
                )
            }
            ToolId::Caveman => Some(has_owner("antigravity", "caveman")),
            ToolId::Rtk => {
                let cfg = read_json_file(&p.hooks).ok().flatten();
                Some(cfg.as_ref().and_then(|c| c.get("rtk")).is_some())
            }
            ToolId::Ponytail => Some(has_owner("antigravity", "ponytail")),
            ToolId::Principles => Some(has_owner("antigravity", "principles")),
        }
    }
}
