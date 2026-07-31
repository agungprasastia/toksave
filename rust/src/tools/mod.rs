pub mod caveman;
pub mod codegraph;
pub mod context_mode;
pub mod ponytail;
pub mod principles;
pub mod rtk;

use crate::registry::{RunOpts, ToolId};
pub use crate::tools::caveman::CavemanTool;
pub use crate::tools::codegraph::CodegraphTool;
pub use crate::tools::context_mode::ContextModeTool;
pub use crate::tools::ponytail::PonytailTool;
pub use crate::tools::principles::PrinciplesTool;
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
        ToolId::Caveman => CavemanTool.installed_version(),
        ToolId::Codegraph => CodegraphTool.installed_version(),
        ToolId::ContextMode => ContextModeTool.installed_version(),
        ToolId::Ponytail => PonytailTool.installed_version(),
        ToolId::Principles => PrinciplesTool.installed_version(),
    }
}

pub async fn install_tool(tool: ToolId, opts: &RunOpts) -> Result<bool> {
    if std::env::var("TOKSAVE_TEST_RTK_INSTALL").is_ok() && tool == ToolId::Rtk && !opts.dry_run {
        return Ok(true);
    }
    match tool {
        ToolId::Rtk => RtkTool.install(opts).await,
        ToolId::Caveman => CavemanTool.install(opts).await,
        ToolId::Codegraph => CodegraphTool.install(opts).await,
        ToolId::ContextMode => ContextModeTool.install(opts).await,
        ToolId::Ponytail => PonytailTool.install(opts).await,
        ToolId::Principles => PrinciplesTool.install(opts).await,
    }
}

pub fn tool_health_check(tool: ToolId) -> HealthStatus {
    match tool {
        ToolId::Rtk => RtkTool.health_check(),
        ToolId::Caveman => CavemanTool.health_check(),
        ToolId::Codegraph => CodegraphTool.health_check(),
        ToolId::ContextMode => ContextModeTool.health_check(),
        ToolId::Ponytail => PonytailTool.health_check(),
        ToolId::Principles => PrinciplesTool.health_check(),
    }
}
