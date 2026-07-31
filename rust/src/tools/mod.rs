use crate::registry::RunOpts;
use crate::util::errors::Result;
use crate::util::health::HealthStatus;

#[allow(async_fn_in_trait)]
pub trait Tool {
    async fn install(&self, opts: &RunOpts) -> Result<bool>;
    fn installed_version(&self) -> Option<String>;
    async fn latest_version(&self) -> Result<Option<String>>;
    fn health_check(&self) -> HealthStatus;
}
