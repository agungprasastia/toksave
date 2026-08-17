use toksave::registry::{
    AgentId, RunOpts, ToolId, detect_agent, parse_agent_id, parse_tool_id, unwire_tool,
    verify_tool, wire_tool,
};
use toksave::util::errors::ToksaveErrorKind;
use toksave::util::json::read_json_file;
use toksave::util::paths::{warp_cli_paths, warp_mcp_files, warp_paths, write_file};

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
    assert_eq!(parse_agent_id("oz"), Some(AgentId::Warp));
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

fn mcp_has(path: &std::path::Path, tool: &str) -> bool {
    read_json_file(path)
        .ok()
        .flatten()
        .and_then(|c| c.get("mcpServers").cloned())
        .and_then(|m| m.get(tool).cloned())
        .is_some()
}

#[tokio::test]
async fn test_warp_wire_codegraph_writes_all_mcp_files() {
    let _env = common::setup();
    let files = warp_mcp_files();
    assert!(
        files.len() >= 3,
        "expected legacy desktop, official desktop, and CLI MCP files, got {files:?}"
    );
    assert!(
        files
            .iter()
            .any(|f| f.file_name().is_some_and(|n| n == "mcp.json"))
    );
    assert!(
        files
            .iter()
            .any(|f| f.ends_with(".warp/.mcp.json")
                || f.file_name().is_some_and(|n| n == ".mcp.json"))
    );

    wire_tool(AgentId::Warp, ToolId::Codegraph, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Warp, ToolId::Codegraph), Some(true));
    for file in &files {
        assert!(file.exists(), "missing MCP file {}", file.display());
        assert!(
            mcp_has(file, "codegraph"),
            "codegraph missing from {}",
            file.display()
        );
    }

    unwire_tool(AgentId::Warp, ToolId::Codegraph, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Warp, ToolId::Codegraph), Some(false));
    for file in &files {
        assert!(
            !mcp_has(file, "codegraph"),
            "codegraph still in {}",
            file.display()
        );
    }
}

#[tokio::test]
async fn test_warp_wire_context_mode_writes_all_mcp_files() {
    let _env = common::setup();
    wire_tool(AgentId::Warp, ToolId::ContextMode, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Warp, ToolId::ContextMode), Some(true));
    for file in warp_mcp_files() {
        assert!(mcp_has(&file, "context-mode"), "{}", file.display());
    }
    unwire_tool(AgentId::Warp, ToolId::ContextMode, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Warp, ToolId::ContextMode), Some(false));
}

#[test]
fn test_warp_detect_cli_config_dir() {
    let _env = common::setup();
    let cli = warp_cli_paths();
    std::fs::create_dir_all(&cli.dir).unwrap();
    let d = detect_agent(AgentId::Warp);
    assert!(d.installed);
    assert_eq!(d.source, "config");
}

#[test]
fn test_warp_detect_oz_binary() {
    let _env = common::setup();
    let bin = _env.home().join(".local").join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let oz = bin.join("oz");
    std::fs::write(&oz, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&oz, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let d = detect_agent(AgentId::Warp);
    assert!(d.installed);
    assert_eq!(d.source, "cli");
}

#[tokio::test]
async fn test_warp_mcp_write_rollback() {
    let _env = common::setup();
    let files = warp_mcp_files();
    let last = files.last().expect("cli mcp path");
    if let Some(parent) = last.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::create_dir_all(last).unwrap();

    let res = wire_tool(AgentId::Warp, ToolId::Codegraph, &RunOpts::default()).await;
    assert!(res.is_err());
    for file in files.iter().filter(|f| *f != last) {
        assert!(
            !mcp_has(file, "codegraph"),
            "rollback left codegraph in {}",
            file.display()
        );
    }
}
