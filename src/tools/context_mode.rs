use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::detect::is_on_path;
use crate::util::errors::{Result, ToksaveError};
use crate::util::exec::{run, run_stdout};
use crate::util::health::{HealthIssue, HealthStatus, RepairResult};

const PACKAGE: &str = "context-mode";

pub struct ContextModeTool;

impl Tool for ContextModeTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        opts.reportf("checking", 0.1);
        if is_on_path("context-mode") && !opts.upgrade {
            return Ok(true);
        }
        if opts.dry_run {
            return Ok(true);
        }

        let npm = crate::util::exec::npm_cmd();
        opts.reportf("npm install -g", 0.4);
        let res = run(npm, &["install", "-g", PACKAGE]);
        if res.code != 0 && !is_on_path("context-mode") {
            return Err(ToksaveError::install(
                "context-mode",
                &format!(
                    "npm install failed\n{}",
                    crate::util::exec::last_nonempty_lines(
                        &format!("{}\n{}", res.stderr, res.stdout),
                        4,
                    )
                ),
                Some("Check your npm registry or network, then run: toksave install context-mode"),
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
        && let Some(stdout) = run_stdout(npm, &["list", "-g", PACKAGE, "--depth=0", "--json"])
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout)
        && let Some(ver) = json
            .get("dependencies")
            .and_then(|d| d.get(PACKAGE))
            .and_then(|entry| entry.get("version"))
            .and_then(|v| v.as_str())
    {
        return Some(ver.to_string());
    }
    if is_on_path("context-mode") {
        return Some("installed".to_string());
    }
    None
}

pub async fn latest_version() -> Result<Option<String>> {
    Ok(crate::util::download::latest_npm_version(PACKAGE).await)
}

pub fn health_check() -> HealthStatus {
    let mut issues = vec![];
    let version = installed_version();

    let Some(version) = version else {
        return HealthStatus {
            healthy: false,
            version: None,
            issues: vec![HealthIssue::error(
                "Context-Mode is not installed",
                "Run: toksave install context-mode",
            )],
        };
    };

    if !is_on_path("context-mode") {
        issues.push(HealthIssue::error(
            "Context-Mode is installed but not in PATH",
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
            message: "Context-Mode is already healthy, no repair needed".to_string(),
            health_after_repair: Some(before_health),
        };
    }

    let upgrade_opts = RunOpts {
        upgrade: true,
        ..opts.clone()
    };
    let _ = ContextModeTool.install(&upgrade_opts).await;

    let after_health = health_check();

    if after_health.healthy {
        RepairResult {
            success: true,
            message: "Context-Mode successfully repaired".to_string(),
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
