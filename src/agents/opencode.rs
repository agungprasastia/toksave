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

/// Agent implementation managing OpenCode integration (v1 and v2).
pub struct OpencodeAgent;

impl OpencodeAgent {
    /// Creates a new `OpencodeAgent` instance.
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
                let server_entry = serde_json::json!({
                    "type": "local",
                    "command": [crate::util::paths::toksave_abs(), "runmcp", "codegraph", "serve", "--mcp"],
                    "enabled": true
                });
                mcp["codegraph"] = server_entry.clone();
                let is_v2 = p.config.file_name().and_then(|f| f.to_str()) == Some("opencode.json")
                    || mcp.get("servers").is_some();
                if is_v2 {
                    let servers = get_or_create_object(mcp, "servers");
                    let mut v2_server = server_entry;
                    v2_server["disabled"] = serde_json::json!(false);
                    servers["codegraph"] = v2_server;
                }
                write_json_file(&p.config, &cfg)?;
                write_owner("opencode", "codegraph")?;
                let plugin_file = p.plugins_dir.join("toksave-autoindex.js");
                let plugin_code = r#"import { execSync } from "node:child_process";
import { Plugin } from "@opencode/plugin";

let indexed = false;
export default Plugin.define({
  id: "toksave.autoindex",
  async setup(ctx) {
    await ctx.tool.hook("execute.before", () => {
      if (indexed) return;
      indexed = true;
      try { execSync("toksave index --auto", { timeout: 120000 }); } catch {}
    });
  },
});
"#;
                write_file(&plugin_file, plugin_code)?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let is_v2 = cfg.get("plugins").is_some()
                    || p.config.file_name().and_then(|f| f.to_str()) == Some("opencode.json");
                let key = if is_v2 { "plugins" } else { "plugin" };
                let plugins = cfg.get_mut(key).and_then(|v| v.as_array_mut());
                let mut plugin_arr = match plugins {
                    Some(arr) => arr.clone(),
                    None => vec![],
                };
                if !plugin_arr.contains(&serde_json::json!("context-mode")) {
                    plugin_arr.push(serde_json::json!("context-mode"));
                }
                cfg[key] = serde_json::Value::Array(plugin_arr);
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
                // Resolve rtk's absolute path the same way toksave's own hook does: a bare
                // `rtk` prefix fails when OpenCode was launched before rtk's install dir was
                // added to PATH (fresh install, GUI-launched session, stale shell PATH cache).
                let rtk_plugin = r#"import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { Plugin } from "@opencode/plugin";

function resolveRtkBin() {
  let rtkBin = "rtk";
  const localPath = process.platform === "win32"
    ? path.join(process.env.LOCALAPPDATA || path.join(os.homedir(), "AppData", "Local"), "Programs", "toksave", "rtk.exe")
    : path.join(os.homedir(), ".local", "bin", "rtk");
  if (fs.existsSync(localPath)) {
    rtkBin = process.platform === "win32" ? localPath.replace(/\//g, "\\") : localPath;
    if (/\s/.test(rtkBin)) rtkBin = `"${rtkBin}"`;
  }
  return rtkBin;
}

export default Plugin.define({
  id: "toksave.rtk",
  async setup(ctx) {
    const rtkBin = resolveRtkBin();
    const alts = [rtkBin, rtkBin.replace(/\\/g, "/"), rtkBin.replace(/\//g, "\\")];
    await ctx.tool.hook("execute.before", (event) => {
      if (event.tool !== "shell" && event.tool !== "bash") return;
      const input = event.input;
      if (!input || typeof input !== "object") return;
      const command = String(input.command ?? "").trim();
      if (!command || /^(rtk|rtk\\.exe)(\\s|$)/.test(command) || alts.some((p) => command === p || command.startsWith(p + " "))) return;
      input.command = `${rtkBin} ${command}`;
    });
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
                        if let Some(servers) =
                            mcp.get_mut("servers").and_then(|s| s.as_object_mut())
                        {
                            servers.remove("codegraph");
                        }
                    }
                    write_json_pruned(&p.config, &cfg)?;
                }
                let _ = fs::remove_file(p.plugins_dir.join("toksave-autoindex.js"));
                let _ = fs::remove_dir_all(p.plugins_dir.join("toksave-autoindex"));
                remove_owner("opencode", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                if let Some(mut cfg) = read_json_file(&p.config)? {
                    for key in ["plugin", "plugins"] {
                        if let Some(plugins) = cfg.get_mut(key).and_then(|v| v.as_array_mut()) {
                            plugins.retain(|v| {
                                v.as_str().map(|s| s != "context-mode").unwrap_or(true)
                            });
                        }
                    }
                    if let Some(servers) = cfg
                        .get_mut("mcp")
                        .and_then(|m| m.get_mut("servers"))
                        .and_then(|s| s.as_object_mut())
                    {
                        servers.remove("context-mode");
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
                let _ = fs::remove_dir_all(p.plugins_dir.join("toksave-rtk"));
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
            ToolId::Codegraph => Some(cfg.as_ref().is_some_and(|c| {
                crate::util::mcp::json_tool_healthy(
                    c,
                    "mcp",
                    crate::registry::AgentId::Opencode,
                    ToolId::Codegraph,
                ) || c
                    .get("mcp")
                    .and_then(|m| m.get("servers"))
                    .is_some_and(|s| {
                        let mut wrapper = serde_json::json!({});
                        wrapper["servers"] = s.clone();
                        crate::util::mcp::json_tool_healthy(
                            &wrapper,
                            "servers",
                            crate::registry::AgentId::Opencode,
                            ToolId::Codegraph,
                        )
                    })
            })),
            ToolId::ContextMode => Some(
                cfg.as_ref()
                    .map(|c| {
                        let in_v1 = c
                            .get("plugin")
                            .and_then(|p| p.as_array())
                            .map(|arr| arr.contains(&serde_json::json!("context-mode")))
                            .unwrap_or(false);
                        let in_v2 = c
                            .get("plugins")
                            .and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter().any(|pl| {
                                    pl.as_str()
                                        .map(|s| s.contains("context-mode"))
                                        .unwrap_or(false)
                                })
                            })
                            .unwrap_or(false);
                        let in_mcp = c
                            .get("mcp")
                            .and_then(|m| m.get("servers"))
                            .and_then(|s| s.get("context-mode"))
                            .is_some();
                        in_v1 || in_v2 || in_mcp
                    })
                    .unwrap_or(false),
            ),
            ToolId::Caveman => Some(has_owner("opencode", "caveman")),
            ToolId::Rtk => Some(
                p.plugins_dir.join("toksave-rtk.js").exists()
                    || p.plugins_dir.join("toksave-rtk").exists(),
            ),
            ToolId::Ponytail => Some(has_owner("opencode", "ponytail")),
            ToolId::Principles => Some(has_owner("opencode", "principles")),
        }
    }
}
