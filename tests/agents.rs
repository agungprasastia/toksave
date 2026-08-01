use toksave::registry::{parse_agent_id, parse_tool_id, AgentId, RunOpts, ToolId};
use toksave::util::errors::ToksaveErrorKind;
use toksave::util::paths::{warp_paths, write_file};

mod common;

#[tokio::test]
async fn test_warp_corrupted_config_fails() {
    let _env = common::setup();
    let p = warp_paths();
    write_file(&p.hooks_file, "{ invalid json ").unwrap();

    let res = toksave::registry::wire_tool(AgentId::Warp, ToolId::Rtk, &RunOpts::default()).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(matches!(err.kind, ToksaveErrorKind::Config));
}

#[test]
fn test_agent_parsing() {
    assert_eq!(parse_agent_id("claude"), Some(AgentId::Claude));
    assert_eq!(parse_agent_id("opencode"), Some(AgentId::Opencode));
    assert_eq!(parse_agent_id("codex"), Some(AgentId::Codex));
    assert_eq!(parse_agent_id("antigravity"), Some(AgentId::Antigravity));
    assert_eq!(parse_agent_id("copilot"), Some(AgentId::Copilot));
    assert_eq!(parse_agent_id("droid"), Some(AgentId::Droid));
    assert_eq!(parse_agent_id("devin"), Some(AgentId::Devin));
    assert_eq!(parse_agent_id("warp"), Some(AgentId::Warp));
}

#[test]
fn test_tool_parsing() {
    assert_eq!(parse_tool_id("rtk"), Some(ToolId::Rtk));
    assert_eq!(parse_tool_id("caveman"), Some(ToolId::Caveman));
    assert_eq!(parse_tool_id("codegraph"), Some(ToolId::Codegraph));
    assert_eq!(parse_tool_id("context-mode"), Some(ToolId::ContextMode));
    assert_eq!(parse_tool_id("ponytail"), Some(ToolId::Ponytail));
    assert_eq!(parse_tool_id("principles"), Some(ToolId::Principles));
}
