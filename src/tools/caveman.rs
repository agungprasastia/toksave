use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::errors::Result;
use crate::util::health::{HealthStatus, RepairResult};

pub struct CavemanTool;

impl Tool for CavemanTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        opts.reportf("instruction-only", 1.0);
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
    Some("instruction-only".to_string())
}

pub async fn latest_version() -> Result<Option<String>> {
    Ok(None)
}

pub fn health_check() -> HealthStatus {
    HealthStatus {
        healthy: true,
        version: None,
        issues: vec![],
    }
}

pub async fn repair(_opts: &RunOpts) -> RepairResult {
    let h = health_check();
    RepairResult {
        success: true,
        message: "Caveman is instruction-only".to_string(),
        health_after_repair: Some(h),
    }
}
