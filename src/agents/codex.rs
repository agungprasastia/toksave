use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{
    get_or_create_object, merge_hook_group, read_json_file, remove_hook_group, write_json_file,
    write_json_pruned,
};
use crate::util::paths::{codex_known_bin_dirs, codex_paths, toksave_abs, toksave_hook_command};
use crate::util::toml::{
    prune_empty_tables, read_toml_file, remove_table, set_table_array, upsert_table,
    write_toml_file, write_toml_pruned,
};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};

pub struct CodexAgent;

impl CodexAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodexAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for CodexAgent {
    fn detect(&self) -> Detection {
        let p = codex_paths();
        let has_cli = find_binary_in("codex", &codex_known_bin_dirs()).is_some();
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
        let p = codex_paths();

        match tool {
            ToolId::Codegraph => {
                let mut doc = read_toml_file(&p.config)?;
                upsert_table(&mut doc, "mcp_servers.codegraph", &toksave_abs());
                if let Some(args) = crate::util::mcp::expected_runmcp_args(
                    crate::registry::AgentId::Codex,
                    ToolId::Codegraph,
                ) {
                    set_table_array(
                        &mut doc,
                        "mcp_servers.codegraph",
                        "args",
                        &args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>(),
                    );
                }
                write_toml_file(&p.config, &doc)?;
                write_owner("codex", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let mut doc = read_toml_file(&p.config)?;
                upsert_table(&mut doc, "mcp_servers.context-mode", &toksave_abs());
                if let Some(args) = crate::util::mcp::expected_runmcp_args(
                    crate::registry::AgentId::Codex,
                    ToolId::ContextMode,
                ) {
                    set_table_array(
                        &mut doc,
                        "mcp_servers.context-mode",
                        "args",
                        &args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>(),
                    );
                }
                write_toml_file(&p.config, &doc)?;
                write_owner("codex", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                write_owner("codex", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                let mut cfg = read_json_file(&p.hooks)?.unwrap_or_else(|| serde_json::json!({}));
                let hook_entry = serde_json::json!({
                    "matcher": "Bash",
                    "hooks": [{ "type": "command", "command": toksave_hook_command("rtk-hook codex"), "timeout": 10 }]
                });
                let hooks = get_or_create_object(&mut cfg, "hooks");
                merge_hook_group(hooks, "PreToolUse", hook_entry, "rtk-hook codex");
                write_json_file(&p.hooks, &cfg)?;
                Ok(true)
            }
            ToolId::Ponytail => {
                write_owner("codex", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                write_owner("codex", "principles")?;
                let mut cfg = read_json_file(&p.hooks)?.unwrap_or_else(|| serde_json::json!({}));
                let perm_entry = serde_json::json!({
                    "matcher": "",
                    "hooks": [{ "type": "command", "command": toksave_hook_command("codex-perm-hook"), "timeout": 5 }]
                });
                let hooks = get_or_create_object(&mut cfg, "hooks");
                merge_hook_group(hooks, "PermissionRequest", perm_entry, "codex-perm-hook");
                write_json_file(&p.hooks, &cfg)?;
                Ok(true)
            }
        }
    }

    fn unwire(&self, tool: ToolId, _opts: &RunOpts) -> Result<bool> {
        let p = codex_paths();
        match tool {
            ToolId::Codegraph => {
                let mut doc = read_toml_file(&p.config)?;
                remove_table(&mut doc, "mcp_servers.codegraph");
                prune_empty_tables(&mut doc);
                write_toml_pruned(&p.config, &doc)?;
                remove_owner("codex", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                let mut doc = read_toml_file(&p.config)?;
                remove_table(&mut doc, "mcp_servers.context-mode");
                prune_empty_tables(&mut doc);
                write_toml_pruned(&p.config, &doc)?;
                remove_owner("codex", "context-mode")?;
                Ok(true)
            }
            ToolId::Caveman => {
                remove_owner("codex", "caveman")?;
                Ok(true)
            }
            ToolId::Rtk => {
                if let Some(mut cfg) = read_json_file(&p.hooks)? {
                    if let Some(hooks) = cfg.get_mut("hooks") {
                        remove_hook_group(hooks, "PreToolUse", "rtk-hook codex");
                        if hooks.as_object().is_some_and(|o| o.is_empty()) {
                            cfg.as_object_mut().expect("object").remove("hooks");
                        }
                    }
                    write_json_pruned(&p.hooks, &cfg)?;
                }
                Ok(true)
            }
            ToolId::Ponytail => {
                remove_owner("codex", "ponytail")?;
                Ok(true)
            }
            ToolId::Principles => {
                if let Some(mut cfg) = read_json_file(&p.hooks)? {
                    if let Some(hooks) = cfg.get_mut("hooks") {
                        if !hooks.is_object() {
                            return Err(crate::util::errors::ToksaveError::config(
                                &p.hooks.to_string_lossy(),
                                "Expected hooks to be an object",
                            ));
                        }
                        remove_hook_group(hooks, "PermissionRequest", "codex-perm-hook");
                        if hooks.as_object().is_some_and(|o| o.is_empty()) {
                            cfg.as_object_mut().expect("object").remove("hooks");
                        }
                    }
                    write_json_pruned(&p.hooks, &cfg)?;
                }
                remove_owner("codex", "principles")?;
                Ok(true)
            }
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        let p = codex_paths();
        match tool {
            ToolId::Codegraph => {
                let doc = read_toml_file(&p.config).ok()?;
                Some(crate::util::mcp::toml_tool_healthy(
                    &doc,
                    crate::registry::AgentId::Codex,
                    ToolId::Codegraph,
                ))
            }
            ToolId::ContextMode => {
                let doc = read_toml_file(&p.config).ok()?;
                Some(crate::util::mcp::toml_tool_healthy(
                    &doc,
                    crate::registry::AgentId::Codex,
                    ToolId::ContextMode,
                ))
            }
            ToolId::Caveman => Some(has_owner("codex", "caveman")),
            ToolId::Rtk => {
                let cfg = read_json_file(&p.hooks).ok().flatten();
                Some(
                    cfg.as_ref()
                        .and_then(|c| c.get("hooks"))
                        .and_then(|h| h.get("PreToolUse"))
                        .is_some(),
                )
            }
            ToolId::Ponytail => Some(has_owner("codex", "ponytail")),
            ToolId::Principles => Some(has_owner("codex", "principles")),
        }
    }
}
