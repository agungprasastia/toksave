pub mod rtk;

use crate::registry::{RunOpts, ToolId};
pub use crate::tools::rtk::{installed_version as rtk_installed_version, RtkTool};
use crate::util::errors::Result;
use crate::util::health::HealthStatus;

#[allow(async_fn_in_trait)]
pub trait Tool {
    async fn install(&self, opts: &RunOpts) -> Result<bool>;
    fn installed_version(&self) -> Option<String>;
    async fn latest_version(&self) -> Result<Option<String>>;
    fn health_check(&self) -> HealthStatus;
}

pub fn tool_installed_version(tool: ToolId) -> Option<String> {
    match tool {
        ToolId::Rtk => rtk_installed_version(),
        _ => None, // other tools ported in later phases
    }
}

pub async fn install_tool(tool: ToolId, opts: &RunOpts) -> Result<bool> {
    match tool {
        ToolId::Rtk => RtkTool.install(opts).await,
        _ => Ok(false), // not yet implemented
    }
}

pub fn tool_health_check(tool: ToolId) -> HealthStatus {
    match tool {
        ToolId::Rtk => RtkTool.health_check(),
        _ => HealthStatus {
            healthy: false,
            version: None,
            issues: vec![],
        },
    }
}
