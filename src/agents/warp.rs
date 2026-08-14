use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::{Result, ToksaveError};
use crate::util::json::{get_or_create_object, read_json_file, write_json_file, write_json_pruned};
use crate::util::paths::{
    toksave_abs, warp_cli_paths, warp_desktop_paths, warp_known_bin_dirs, warp_mcp_files,
    warp_paths,
};
use crate::util::unified_block::{has_owner, remove_owner, write_owner};
use serde_json::Value;
use std::path::{Path, PathBuf};

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

struct FileBackup {
    path: PathBuf,
    previous: Option<Vec<u8>>,
    existed: bool,
}

fn restore_backups(backups: &[FileBackup]) {
    for b in backups.iter().rev() {
        if b.existed {
            if let Some(bytes) = &b.previous {
                let _ = std::fs::write(&b.path, bytes);
            }
        } else {
            let _ = std::fs::remove_file(&b.path);
        }
    }
}

fn backup_file(path: &Path) -> FileBackup {
    let existed = path.exists();
    FileBackup {
        path: path.to_path_buf(),
        previous: if existed {
            std::fs::read(path).ok()
        } else {
            None
        },
        existed,
    }
}

fn mcp_file_has(path: &Path, tool: &str) -> bool {
    read_json_file(path)
        .ok()
        .flatten()
        .and_then(|c| c.get("mcpServers").cloned())
        .and_then(|m| m.get(tool).cloned())
        .is_some()
}

fn mcp_has_all(tool: &str) -> bool {
    let files = warp_mcp_files();
    !files.is_empty() && files.iter().all(|f| mcp_file_has(f, tool))
}

fn upsert_mcp_all(tool: &str, entry: Value) -> Result<()> {
    let files = warp_mcp_files();
    let mut backups = Vec::new();
    for path in files {
        backups.push(backup_file(&path));
        let result = (|| {
            let mut cfg = read_json_file(&path)?.unwrap_or_else(|| serde_json::json!({}));
            let servers = get_or_create_object(&mut cfg, "mcpServers");
            servers[tool] = entry.clone();
            write_json_file(&path, &cfg)
        })();
        if let Err(e) = result {
            restore_backups(&backups);
            return Err(e);
        }
    }
    Ok(())
}

fn remove_mcp_all(tool: &str) -> Result<()> {
    let files = warp_mcp_files();
    let mut backups = Vec::new();
    for path in files {
        if !path.exists() {
            continue;
        }
        backups.push(backup_file(&path));
        let result = (|| {
            if let Some(mut cfg) = read_json_file(&path)? {
                if let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                    mcp.remove(tool);
                }
                write_json_pruned(&path, &cfg)?;
            }
            Ok::<(), ToksaveError>(())
        })();
        if let Err(e) = result {
            restore_backups(&backups);
            return Err(e);
        }
    }
    Ok(())
}

impl Agent for WarpAgent {
    fn detect(&self) -> Detection {
        let p = warp_paths();
        let cli = warp_cli_paths();
        let has_warp_bin = find_binary_in("warp", &warp_known_bin_dirs()).is_some();
        let has_oz_bin = find_binary_in("oz", &warp_known_bin_dirs()).is_some();
        let has_cli = has_warp_bin || has_oz_bin;
        let has_desktop = std::env::var("TOKSAVE_TEST").is_err()
            && warp_desktop_paths().iter().any(|p| p.exists());
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
        let has_config =
            std::env::var("TOKSAVE_TEST").is_ok() && (p.dir.exists() || cli.dir.exists());
        let has_mcp = warp_mcp_files().iter().any(|f| f.exists());
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
                upsert_mcp_all(
                    "codegraph",
                    serde_json::json!({
                        "command": toksave_abs(),
                        "args": ["runmcp", "--agent", "warp", "codegraph", "serve", "--mcp"]
                    }),
                )?;
                write_owner("warp", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                upsert_mcp_all(
                    "context-mode",
                    serde_json::json!({
                        "command": toksave_abs(),
                        "args": ["runmcp", "--agent", "warp", "context-mode"]
                    }),
                )?;
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
                remove_mcp_all("codegraph")?;
                remove_owner("warp", "codegraph")?;
                Ok(true)
            }
            ToolId::ContextMode => {
                remove_mcp_all("context-mode")?;
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
                    write_json_pruned(&p.hooks_file, &cfg)?;
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
        match tool {
            ToolId::Codegraph => Some(mcp_has_all("codegraph")),
            ToolId::ContextMode => Some(mcp_has_all("context-mode")),
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
