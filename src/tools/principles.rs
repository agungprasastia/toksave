use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::errors::Result;
use crate::util::health::{HealthStatus, RepairResult};

pub struct PrinciplesTool;

impl Tool for PrinciplesTool {
    async fn install(&self, _opts: &RunOpts) -> Result<bool> {
        Ok(true)
    }

    fn installed_version(&self) -> Option<String> {
        installed_version()
    }

    async fn latest_version(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn health_check(&self) -> HealthStatus {
        health_check()
    }
}

pub fn installed_version() -> Option<String> {
    Some("instruction-only".to_string())
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
        message: "Principles is instruction-only".to_string(),
        health_after_repair: Some(h),
    }
}
