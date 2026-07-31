use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::errors::Result;

pub trait Agent {
    fn detect(&self) -> Detection;
    fn wire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool>;
    fn unwire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool>;
    fn verify(&self, tool: ToolId) -> Option<bool>;
}
