pub mod antigravity;
pub mod claude;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod devin;
pub mod droid;
pub mod opencode;
pub mod warp;

pub use antigravity::AntigravityAgent;
pub use claude::ClaudeAgent;
pub use codex::CodexAgent;
pub use copilot::CopilotAgent;
pub use cursor::CursorAgent;
pub use devin::DevinAgent;
pub use droid::DroidAgent;
pub use opencode::OpencodeAgent;
pub use warp::WarpAgent;

use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::errors::Result;

pub trait Agent {
    fn detect(&self) -> Detection;
    fn wire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool>;
    fn unwire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool>;
    fn verify(&self, tool: ToolId) -> Option<bool>;
}
