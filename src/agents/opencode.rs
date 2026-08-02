use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{
    get_or_create_object, has_key, read_json_file, write_json_file, write_json_pruned,
};
use crate::util::paths::{
    opencode_desktop_paths, opencode_known_bin_dirs, opencode_paths, write_file,
};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};
use std::fs;

pub struct OpencodeAgent;

impl OpencodeAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpencodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for OpencodeAgent {
    fn detect(&self) -> Detection {
        let p = opencode_paths();
        let has_cli = find_binary_in("opencode", &opencode_known_bin_dirs()).is_some();
        let has_desktop = opencode_desktop_paths().iter().any(|p| p.exists());
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
        let p = opencode_paths();
        let mut cfg = read_json_file(&p.config)?.unwrap_or_else(|| serde_json::json!({}));
        if !has_key(&cfg, "$schema") {
            cfg["$schema"] = serde_json::json!("https://opencode.ai/config.json");
        }

        match tool {
            ToolId::Codegraph => {
                let mcp = get_or_create_object(&mut cfg, "mcp");
                mcp["codegraph"] = serde_json::json!({
                    "type": "local",
                    "command": [crate::util::paths::toksave_abs(), "runmcp", "codegraph", "serve", "--mcp"],
                    "enabled": true
                });
                write_json_file(&p.config, &cfg)?;
                write_owner("opencode", "codegraph")?;
                let plugin_file = p.plugins_dir.join("toksave-autoindex.js");
                let plugin_code = r#"let indexed = false;
export const Plugin = async () => ({
  "tool.execute.before": async () => {
    if (indexed) return;
    indexed = true;
    const { execSync } = require("node:child_process");
    try { execSync("toksave index --auto", { timeout: 120000 }); } catch {}
  },
});
"#;
                write_file(&plugin_file, plugin_code)?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let plugins = cfg.get_mut("plugin").and_then(|v| v.as_array_mut());
                let mut plugin_arr = match plugins {
                    Some(arr) => arr.clone(),
                    None => vec![],
                };
                if !plugin_arr.contains(&serde_json::json!("context-mode")) {
                    plugin_arr.push(serde_json::json!("context-mode"));
                }
                cfg["plugin"] = serde_json::Value::Array(plugin_arr);
                write_json_file(&p.config, &cfg)?;
                write_owner("opencode", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("opencode", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let plugin_file = p.plugins_dir.join("toksave-rtk.js");
                let rtk_plugin = r#"export const Plugin = async () => ({
  "tool.execute.before": async (input, output) => {
    if (input.tool !== "bash") return;
    const command = String(output.args.command ?? "").trim();
    if (!command || command === "rtk" || command.startsWith("rtk ")) return;
    output.args.command = `rtk ${command}`;
  },
});
"#;
                write_file(&plugin_file, rtk_plugin)?;
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("opencode", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("opencode", "principles")?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = opencode_paths();
        match tool {
            ToolId::Codegraph => {
                if let Some(mut cfg) = read_json_file(&p.config)? {
                    if let Some(mcp) = cfg.get_mut("mcp").and_then(|v| v.as_object_mut()) {
                        mcp.remove("codegraph");
                    }
                    write_json_pruned(&p.config, &cfg)?;
                }
                let _ = fs::remove_file(p.plugins_dir.join("toksave-autoindex.js"));
                remove_owner("opencode", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.config)? {
                    if let Some(plugins) = cfg.get_mut("plugin").and_then(|v| v.as_array_mut()) {
                        plugins.retain(|v| v != "context-mode");
                    }
                    write_json_pruned(&p.config, &cfg)?;
                }
                remove_owner("opencode", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("opencode", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let _ = fs::remove_file(p.plugins_dir.join("toksave-rtk.js"));
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("opencode", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                remove_owner("opencode", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = opencode_paths();
        let cfg = read_json_file(&p.config).ok().flatten();
        match tool {
            ToolId::Codegraph => Some(
                cfg.as_ref()
                    .and_then(|c| c.get("mcp"))
                    .and_then(|m| m.get("codegraph"))
                    .is_some(),
            ),
            ToolId::ContextMode => Some(
                cfg.as_ref()
                    .and_then(|c| c.get("plugin"))
                    .and_then(|p| p.as_array())
                    .map(|arr| arr.contains(&serde_json::json!("context-mode")))
                    .unwrap_or(false),
            ),
            ToolId::Caveman => Some(has_owner("opencode", "caveman")),
            ToolId::Rtk => Some(p.plugins_dir.join("toksave-rtk.js").exists()),
            ToolId::Ponytail => Some(has_owner("opencode", "ponytail")),
            ToolId::Principles => Some(has_owner("opencode", "principles")),
        }
    }
}
