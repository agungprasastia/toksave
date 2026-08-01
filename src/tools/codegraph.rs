use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::detect::is_on_path;
use crate::util::errors::{Result, ToksaveError};
use crate::util::exec::{run, run_stdout};
use crate::util::health::{HealthIssue, HealthStatus, RepairResult};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const PACKAGE: &str = "@colbymchenry/codegraph";

pub struct CodegraphTool;

impl Tool for CodegraphTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        if is_on_path("codegraph") && !opts.upgrade {
            return Ok(true);
        }
        if env::var("TOKSAVE_TEST").is_ok() || opts.dry_run {
            return Ok(true);
        }

        let npm = crate::util::exec::npm_cmd();
        let res = run(npm, &["install", "-g", PACKAGE]);
        Ok(res.code == 0 || is_on_path("codegraph") || resolve_codegraph_bin().is_some())
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

pub fn resolve_codegraph_bin() -> Option<String> {
    if is_on_path("codegraph") {
        Some("codegraph".to_string())
    } else {
        None
    }
}

pub fn codegraph_real_install(opts: &RunOpts, agent: &str) -> bool {
    if opts.dry_run || env::var("TOKSAVE_TEST").is_ok() {
        return true;
    }
    let bin = match resolve_codegraph_bin() {
        Some(b) => b,
        None => return false,
    };

    let help_out = run_stdout(&bin, &["install", "--help"]).unwrap_or_default();
    let has_yes = help_out.contains("--yes");
    let has_target = help_out.contains("--target");

    let mut args = vec!["install"];
    let target_str;
    if has_yes {
        args.push("--yes");
    }
    if has_target {
        let mut target = agent;
        if target == "antigravity" {
            target = "gemini";
        }
        if target.is_empty() {
            target = "all";
        }
        target_str = target;
        args.push("--target");
        args.push(target_str);
    }

    let res = run(&bin, &args);
    res.code == 0
}

pub fn installed_version() -> Option<String> {
    // npm global list requires npm to be resolvable; on Windows "npm" bare name
    // may not spawn (no .exe) — use npm_cmd() which resolves "npm.cmd".
    let npm = crate::util::exec::npm_cmd();
    if is_on_path(npm) {
        if let Some(stdout) = run_stdout(npm, &["list", "-g", PACKAGE, "--depth=0", "--json"]) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(ver) = json
                    .get("dependencies")
                    .and_then(|d| d.get(PACKAGE))
                    .and_then(|entry| entry.get("version"))
                    .and_then(|v| v.as_str())
                {
                    return Some(ver.to_string());
                }
            }
        }
    }
    // Fallback: if the binary itself is on PATH, read its --version directly.
    if let Some(out) = run_stdout("codegraph", &["--version"]) {
        let v = out.trim().trim_start_matches('v');
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

pub async fn latest_version() -> Result<Option<String>> {
    let url = format!(
        "https://registry.npmjs.org/{}",
        urlencoding::encode(PACKAGE)
    );
    match crate::util::download::fetch_json(&url).await {
        Ok(json) => Ok(json
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())),
        Err(_) => Ok(None),
    }
}

pub struct IndexResult {
    pub success: bool,
    pub index_path: PathBuf,
}

pub struct IndexOptions<'a> {
    pub verbose: bool,
    pub skip_patterns: Vec<String>,
    pub on_progress: Option<&'a dyn Fn(&str)>,
}

pub fn index_project(dir: &Path, opts: Option<&IndexOptions>) -> Result<IndexResult> {
    let index_path = dir.join(".codegraph");
    let has_index = index_path.exists();

    if env::var("TOKSAVE_TEST").is_ok() {
        let _ = fs::create_dir_all(&index_path);
        return Ok(IndexResult {
            success: true,
            index_path,
        });
    }

    if let Some(opts) = opts {
        if let Some(on_progress) = opts.on_progress {
            if has_index {
                on_progress("Syncing existing index...");
                let res = run("codegraph", &["sync"]);
                if res.code != 0 {
                    return Err(ToksaveError::tool(
                        "codegraph",
                        "Failed to sync CodeGraph index",
                    ));
                }
            } else {
                on_progress("Creating new index...");
                let init_i = run("codegraph", &["init", "-i"]);
                if init_i.code != 0 {
                    let init_res = run("codegraph", &["init"]);
                    if init_res.code != 0 {
                        return Err(ToksaveError::tool(
                            "codegraph",
                            "Failed to initialize CodeGraph index",
                        ));
                    }
                }
            }
        }
    }

    Ok(IndexResult {
        success: true,
        index_path,
    })
}

pub fn codegraph_index_ready() -> bool {
    if env::var("TOKSAVE_TEST").is_ok() {
        return true;
    }
    is_on_path("codegraph")
}

pub fn health_check() -> HealthStatus {
    let mut issues = vec![];
    let version = installed_version();

    let Some(version) = version else {
        return HealthStatus {
            healthy: false,
            version: None,
            issues: vec![HealthIssue::error(
                "CodeGraph is not installed",
                "Run: toksave install codegraph",
            )],
        };
    };

    if !is_on_path("codegraph") {
        issues.push(HealthIssue::error(
            "CodeGraph is installed but not in PATH",
            "Add npm global bin directory to PATH",
        ));
    }

    let healthy = issues.is_empty();
    HealthStatus {
        healthy,
        version: Some(version),
        issues,
    }
}

pub async fn repair(opts: &RunOpts) -> RepairResult {
    let before_health = health_check();

    if before_health.healthy {
        return RepairResult {
            success: true,
            message: "CodeGraph is already healthy, no repair needed".to_string(),
            health_after_repair: Some(before_health),
        };
    }

    let upgrade_opts = RunOpts {
        upgrade: true,
        ..*opts
    };
    let _ = CodegraphTool.install(&upgrade_opts).await;

    let after_health = health_check();

    if after_health.healthy {
        RepairResult {
            success: true,
            message: "CodeGraph successfully repaired".to_string(),
            health_after_repair: Some(after_health),
        }
    } else {
        RepairResult {
            success: false,
            message: "Repair attempted but health check still failing".to_string(),
            health_after_repair: Some(after_health),
        }
    }
}
