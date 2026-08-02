use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::detect::is_on_path;
use crate::util::errors::{Result, ToksaveError};
use crate::util::exec::{run, run_stdout};
use crate::util::health::{HealthIssue, HealthStatus, RepairResult};
use crate::util::json::{read_json_file, write_json_file};
use crate::util::paths::opencode_paths;
use std::env;

pub const PONYTAIL_PKG: &str = "@dietrichgebert/ponytail";

pub struct PonytailTool;

impl Tool for PonytailTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        if opts.dry_run || env::var("TOKSAVE_TEST").is_ok() {
            return Ok(true);
        }

        if !opts.upgrade
            && let Some(_v) = installed_version()
        {
            return Ok(true);
        }

        let npm = crate::util::exec::npm_cmd();
        if !is_on_path(npm) {
            return Ok(false);
        }

        opts.reportf("npm install -g", 0.4);
        let res = run(npm, &["install", "-g", &format!("{PONYTAIL_PKG}@latest")]);
        if res.code != 0 {
            return Err(ToksaveError::install(
                "ponytail",
                &format!(
                    "npm install failed\n{}",
                    crate::util::exec::last_nonempty_lines(
                        &format!("{}\n{}", res.stderr, res.stdout),
                        4,
                    )
                ),
                Some("Check your npm registry or network, then run: toksave install ponytail"),
            ));
        }
        opts.reportf("ready", 1.0);
        Ok(true)
    }

    fn installed_version(&self) -> Option<String> {
        installed_version()
    }

    async fn latest_version(&self) -> Result<Option<String>> {
        latest_version().await
    }

    fn health_check(&self) -> HealthStatus {
        health_check()
    }
}

pub fn installed_version() -> Option<String> {
    let npm = crate::util::exec::npm_cmd();
    if is_on_path(npm)
        && let Some(stdout) = run_stdout(npm, &["list", "-g", PONYTAIL_PKG, "--depth=0", "--json"])
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout)
        && let Some(ver) = json
            .get("dependencies")
            .and_then(|d| d.get(PONYTAIL_PKG))
            .and_then(|entry| entry.get("version"))
            .and_then(|v| v.as_str())
    {
        return Some(ver.to_string());
    }

    if ponytail_plugin_installed() {
        return Some("0.0.0".to_string());
    }

    None
}

pub async fn latest_version() -> Result<Option<String>> {
    Ok(crate::util::download::latest_npm_version(PONYTAIL_PKG).await)
}

pub fn health_check() -> HealthStatus {
    let v = installed_version();
    let Some(version) = v else {
        return HealthStatus {
            healthy: false,
            version: None,
            issues: vec![HealthIssue::error(
                "Ponytail not installed",
                "Run: toksave --tools ponytail",
            )],
        };
    };
    HealthStatus {
        healthy: true,
        version: Some(version),
        issues: vec![],
    }
}

pub async fn repair(opts: &RunOpts) -> RepairResult {
    let before = health_check();
    if before.healthy {
        return RepairResult {
            success: true,
            message: "Ponytail already healthy".to_string(),
            health_after_repair: Some(before),
        };
    }

    let upgrade_opts = RunOpts {
        upgrade: true,
        ..opts.clone()
    };
    let ok = PonytailTool.install(&upgrade_opts).await.unwrap_or(false);
    let after = health_check();
    RepairResult {
        success: ok && after.healthy,
        message: if ok {
            "Ponytail repaired".to_string()
        } else {
            "Ponytail repair failed".to_string()
        },
        health_after_repair: Some(after),
    }
}

pub fn register_opencode_plugin() {
    let p = opencode_paths();
    let mut cfg = read_json_file(&p.config).ok().flatten().unwrap_or_else(|| {
        serde_json::json!({
            "$schema": "https://opencode.ai/config.json"
        })
    });

    let plugins = cfg
        .get("plugin")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default();

    let already_present = plugins.iter().any(|pl| {
        pl.as_str()
            .map(|s| s.eq_ignore_ascii_case(PONYTAIL_PKG))
            .unwrap_or(false)
    });

    if already_present {
        return;
    }

    let mut inserted = false;
    let mut out = Vec::new();
    for pl in plugins {
        if !inserted
            && let Some(s) = pl.as_str()
            && (s == "context-mode" || s.starts_with("context-mode@"))
        {
            out.push(serde_json::json!(PONYTAIL_PKG));
            inserted = true;
        }
        out.push(pl);
    }
    if !inserted {
        out.push(serde_json::json!(PONYTAIL_PKG));
    }

    cfg["plugin"] = serde_json::Value::Array(out);
    let _ = write_json_file(&p.config, &cfg);
}

pub fn unregister_opencode_plugin() {
    let p = opencode_paths();
    let mut cfg = match read_json_file(&p.config).ok().flatten() {
        Some(c) => c,
        None => return,
    };

    if let Some(plugins) = cfg.get("plugin").and_then(|p| p.as_array()) {
        let kept: Vec<serde_json::Value> = plugins
            .iter()
            .filter(|pl| {
                if let Some(s) = pl.as_str() {
                    !s.eq_ignore_ascii_case(PONYTAIL_PKG)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        cfg["plugin"] = serde_json::Value::Array(kept);
        let _ = write_json_file(&p.config, &cfg);
    }
}

pub fn ponytail_plugin_installed() -> bool {
    let p = opencode_paths();
    let cfg = match read_json_file(&p.config).ok().flatten() {
        Some(c) => c,
        None => return false,
    };
    cfg.get("plugin")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter().any(|pl| {
                pl.as_str()
                    .map(|s| s.eq_ignore_ascii_case(PONYTAIL_PKG))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}
