use toksave::registry::{AgentId, RunOpts, ToolId, detect_agent, parse_agent_id, parse_tool_id};
use toksave::util::errors::ToksaveErrorKind;
use toksave::util::json::read_json_file;
use toksave::util::paths::{cursor_paths, warp_paths, write_file};

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
    assert_eq!(parse_agent_id("cursor"), Some(AgentId::Cursor));
    assert_eq!(parse_agent_id("cursor-cli"), Some(AgentId::Cursor));
}

#[tokio::test]
async fn test_cursor_corrupted_hooks_fails() {
    let _env = common::setup();
    let p = cursor_paths();
    write_file(&p.hooks_file, "{ invalid json ").unwrap();

    let res = toksave::registry::wire_tool(AgentId::Cursor, ToolId::Rtk, &RunOpts::default()).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(matches!(err.kind, ToksaveErrorKind::Config));
}

#[tokio::test]
async fn test_cursor_rtk_writes_native_pretooluse() {
    let _env = common::setup();
    let opts = RunOpts::default();
    toksave::registry::wire_tool(AgentId::Cursor, ToolId::Rtk, &opts)
        .await
        .unwrap();

    let p = cursor_paths();
    let cfg = read_json_file(&p.hooks_file).unwrap().unwrap();
    let hooks = cfg["hooks"]["preToolUse"].as_array().expect("preToolUse");
    assert!(
        hooks.iter().any(|h| {
            h.get("command")
                .and_then(|c| c.as_str())
                .is_some_and(|c| c.contains("rtk-hook cursor"))
                && h.get("matcher").and_then(|m| m.as_str()) == Some("Shell")
        }),
        "expected native Cursor preToolUse hook, got {cfg}"
    );

    let cli = read_json_file(&p.cli_config).unwrap().unwrap();
    let allow = cli["permissions"]["allow"].as_array().expect("allow");
    assert!(
        allow.iter().any(|v| v.as_str() == Some("Shell(rtk *)")),
        "expected Shell(rtk *) allow, got {cli}"
    );
}

#[test]
fn test_cursor_detect_uses_config_dir_in_test_mode() {
    let _env = common::setup();
    let p = cursor_paths();
    std::fs::create_dir_all(&p.dir).unwrap();
    let det = detect_agent(AgentId::Cursor);
    assert!(det.installed);
    assert_eq!(det.source, "config");
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
