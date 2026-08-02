use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::detect::is_on_path;
use crate::util::download::{DownloadOptions, download_tar_gz, download_zip, make_executable};
use crate::util::errors::{Result, ToksaveError};
use crate::util::exec::{run, run_stdout};
use crate::util::health::{HealthIssue, HealthStatus};
use crate::util::paths::{ensure_dir, local_bin};
use std::path::PathBuf;

#[allow(unreachable_code)]
pub fn asset_name() -> Option<&'static str> {
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return Some("rtk-x86_64-apple-darwin.tar.gz");
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return Some("rtk-aarch64-apple-darwin.tar.gz");
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Some("rtk-x86_64-unknown-linux-musl.tar.gz");
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return Some("rtk-aarch64-unknown-linux-musl.tar.gz");
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return Some("rtk-x86_64-pc-windows-msvc.zip");
    }
    None
}

pub fn rtk_bin_name() -> &'static str {
    if cfg!(windows) { "rtk.exe" } else { "rtk" }
}

pub fn local_rtk_path() -> PathBuf {
    local_bin().join(rtk_bin_name())
}

pub fn is_installed_but_unreachable() -> bool {
    if is_on_path("rtk") {
        return false;
    }
    local_rtk_path().exists()
}

pub struct RtkTool;

impl Tool for RtkTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        opts.reportf("checking", 0.1);
        if installed_version().is_some() && !opts.upgrade {
            return Ok(true);
        }
        if opts.dry_run {
            return Ok(true);
        }

        let dest = local_bin();
        ensure_dir(&dest)?;

        if let Some(asset) = asset_name() {
            let url = format!("https://github.com/rtk-ai/rtk/releases/latest/download/{asset}");
            let result = async {
                opts.reportf("downloading", 0.4);
                if asset.ends_with(".tar.gz") {
                    download_tar_gz(&url, &dest, &DownloadOptions::default()).await?;
                    make_executable(&dest.join("rtk"))?;
                } else if asset.ends_with(".zip") {
                    download_zip(&url, &dest, &DownloadOptions::default()).await?;
                }
                let rtk_path = local_rtk_path();
                let init = run(&rtk_path.to_string_lossy(), &["init", "-g"]);
                if init.code != 0 {
                    let _ = std::fs::remove_file(&rtk_path);
                    return Err(ToksaveError::install(
                        "rtk",
                        "Failed to initialize RTK shell integration",
                        Some("Try running 'rtk init -g' manually after installation completes"),
                    ));
                }
                opts.reportf("ready", 1.0);
                Ok(true)
            }
            .await;
            match result {
                Ok(ok) => return Ok(ok),
                Err(e) => {
                    // fall through to fallback methods (mirror TS: try next method)
                    let _ = e;
                }
            }
        }

        // Fallback: official install script (Unix only)
        #[cfg(not(windows))]
        {
            if is_on_path("curl") && is_on_path("sh") {
                let r = run(
                    "sh",
                    &[
                        "-c",
                        "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh",
                    ],
                );
                if r.code == 0 {
                    let init = run("rtk", &["init", "-g"]);
                    if init.code == 0 {
                        return Ok(true);
                    }
                }
            }
        }

        // Fallback: cargo install
        if is_on_path("cargo") {
            let r = run(
                "cargo",
                &["install", "--git", "https://github.com/rtk-ai/rtk"],
            );
            if r.code == 0 {
                let init = run("rtk", &["init", "-g"]);
                if init.code == 0 {
                    return Ok(true);
                }
            }
        }

        Err(ToksaveError::platform(
            &format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            "No installation method available",
            Some("Visit https://github.com/rtk-ai/rtk for manual installation instructions"),
        ))
    }

    fn installed_version(&self) -> Option<String> {
        installed_version()
    }

    async fn latest_version(&self) -> Result<Option<String>> {
        match crate::util::download::fetch_json(
            "https://api.github.com/repos/rtk-ai/rtk/releases/latest",
        )
        .await
        {
            Ok(v) => Ok(v
                .get("tag_name")
                .and_then(|t| t.as_str())
                .map(|s| s.trim_start_matches('v').to_string())),
            Err(_) => Ok(None),
        }
    }

    fn health_check(&self) -> HealthStatus {
        let mut issues = vec![];
        let version = installed_version();

        let Some(version) = version else {
            return HealthStatus {
                healthy: false,
                version: None,
                issues: vec![HealthIssue::error(
                    "RTK is not installed",
                    "Run: toksave init -t rtk",
                )],
            };
        };

        if is_installed_but_unreachable() {
            let bin = local_bin();
            let instruct = if cfg!(windows) {
                format!(
                    "Add {} to your system PATH via System Properties or setx PATH \"%PATH%;{}\"",
                    bin.display(),
                    bin.display()
                )
            } else {
                format!(
                    "Add 'export PATH=\"$PATH:{}\"' to your shell rc file (e.g. ~/.bashrc or ~/.zshrc) and restart your terminal",
                    bin.display()
                )
            };
            issues.push(HealthIssue {
                severity: crate::util::health::Severity::Error,
                message: format!(
                    "RTK is installed in {} but is not on your PATH",
                    bin.display()
                ),
                remediation: Some(format!(
                    "Enforcement hooks will fail with 'command not found'. {instruct}"
                )),
            });
        }

        let healthy = issues.is_empty()
            || issues
                .iter()
                .all(|i| i.severity == crate::util::health::Severity::Warning);
        HealthStatus {
            healthy,
            version: Some(version),
            issues,
        }
    }
}

pub async fn repair(opts: &RunOpts) -> crate::util::health::RepairResult {
    let before = RtkTool.health_check();
    if before.healthy {
        return crate::util::health::RepairResult {
            success: true,
            message: "RTK is already healthy, no repair needed".to_string(),
            health_after_repair: Some(before),
        };
    }
    let upgrade_opts = RunOpts {
        upgrade: true,
        ..opts.clone()
    };
    let _ = RtkTool.install(&upgrade_opts).await;
    let after = RtkTool.health_check();
    if after.healthy {
        crate::util::health::RepairResult {
            success: true,
            message: "RTK successfully repaired".to_string(),
            health_after_repair: Some(after),
        }
    } else {
        crate::util::health::RepairResult {
            success: false,
            message: "Repair attempted but health check still failing".to_string(),
            health_after_repair: Some(after),
        }
    }
}

pub fn installed_version() -> Option<String> {
    let path_version = run_stdout("rtk", &["--version"])?;
    let pv = path_version.trim().trim_start_matches("rtk ").trim();
    if !pv.is_empty() {
        return Some(pv.to_string());
    }
    let local = local_rtk_path();
    if !local.exists() {
        return None;
    }
    run_stdout(&local.to_string_lossy(), &["--version"])
}
