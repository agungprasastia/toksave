use toksave::registry::{
    AgentId, RunOpts, ToolId, detect_agent, parse_agent_id, parse_tool_id, unwire_tool,
    verify_tool, wire_tool,
};
use toksave::util::errors::ToksaveErrorKind;
use toksave::util::json::read_json_file;
use toksave::util::paths::{
    antigravity_paths, claude_paths, copilot_paths, cursor_paths, devin_paths, droid_paths,
    warp_cli_paths, warp_mcp_files, warp_paths, write_file,
};

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

    toksave::registry::unwire_tool(AgentId::Cursor, ToolId::Rtk, &opts)
        .await
        .unwrap();
    assert!(
        !p.cli_config.exists(),
        "cli-config.json should be pruned after unwire"
    );
    assert!(
        !p.hooks_file.exists(),
        "hooks.json should be pruned after unwire"
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
fn test_cursor_paths_ignore_xdg_and_use_dot_cursor() {
    // Cursor's documented user hooks file is ~/.cursor/hooks.json, not
    // $XDG_CONFIG_HOME/cursor/hooks.json. Official RTK writes the former;
    // wiring the XDG path leaves two competing files and the CLI never sees ours.
    let _env = common::setup();
    let xdg = _env.home().join(".config");
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", &xdg);
    }
    let p = cursor_paths();
    assert_eq!(p.dir, _env.home().join(".cursor"));
    assert_eq!(p.hooks_file, _env.home().join(".cursor").join("hooks.json"));
    assert_ne!(p.dir, xdg.join("cursor"));
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
async fn test_antigravity_codegraph_wires_mcp_config_json_under_gemini() {
    let _env = common::setup();
    let p = antigravity_paths();
    assert!(
        p.dir.ends_with(".gemini"),
        "Antigravity CLI keeps the ~/.gemini namespace even with Gemini CLI discontinued, got {}",
        p.dir.display()
    );

    wire_tool(AgentId::Antigravity, ToolId::Codegraph, &RunOpts::default())
        .await
        .unwrap();

    let mcp_file = p.dir.join("config").join("mcp_config.json");
    assert!(
        mcp_file.exists(),
        "expected Antigravity's real MCP file at {}",
        mcp_file.display()
    );
    let cfg = read_json_file(&mcp_file).unwrap().unwrap();
    assert!(cfg["mcpServers"]["codegraph"].is_object());
    assert_eq!(
        verify_tool(AgentId::Antigravity, ToolId::Codegraph),
        Some(true)
    );

    unwire_tool(AgentId::Antigravity, ToolId::Codegraph, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(
        verify_tool(AgentId::Antigravity, ToolId::Codegraph),
        Some(false)
    );
}

#[tokio::test]
async fn test_droid_rtk_wires_under_dot_factory_dir() {
    let _env = common::setup();
    let p = droid_paths();
    assert!(
        p.dir.ends_with(".factory"),
        "Droid's real config dir is ~/.factory, not ~/.factory-droid, got {}",
        p.dir.display()
    );

    wire_tool(AgentId::Droid, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    let cfg = read_json_file(&p.hooks_file).unwrap().unwrap();
    let arr = cfg["PreToolUse"].as_array().expect("PreToolUse");
    assert!(arr.iter().any(|g| {
        g.get("matcher").and_then(|m| m.as_str()) == Some("Execute")
            && g.get("hooks")
                .and_then(|h| h.as_array())
                .is_some_and(|hooks| {
                    hooks.iter().any(|h| {
                        h.get("command")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains("rtk-hook droid"))
                    })
                })
    }));

    unwire_tool(AgentId::Droid, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Droid, ToolId::Rtk), Some(false));
}

#[tokio::test]
async fn test_devin_rtk_wires_under_config_devin_with_exec_matcher() {
    let _env = common::setup();
    let p = devin_paths();

    wire_tool(AgentId::Devin, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    assert!(
        !p.hooks_file.exists(),
        "Devin CLI has no standalone hooks.json -- hooks live in config.json"
    );
    let cfg = read_json_file(&p.config).unwrap().unwrap();
    let arr = cfg["hooks"]["PreToolUse"].as_array().expect("PreToolUse");
    assert!(arr.iter().any(|g| {
        g.get("matcher").and_then(|m| m.as_str()) == Some("exec")
            && g.get("hooks")
                .and_then(|h| h.as_array())
                .is_some_and(|hooks| {
                    hooks.iter().any(|h| {
                        h.get("command")
                            .and_then(|c| c.as_str())
                            .is_some_and(|c| c.contains("rtk-hook devin"))
                    })
                })
    }));
    assert_eq!(verify_tool(AgentId::Devin, ToolId::Rtk), Some(true));

    unwire_tool(AgentId::Devin, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Devin, ToolId::Rtk), Some(false));
}

#[tokio::test]
async fn test_warp_rtk_wire_writes_no_hook_file() {
    let _env = common::setup();
    let p = warp_paths();

    wire_tool(AgentId::Warp, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    assert!(
        !p.hooks_file.exists(),
        "Warp has no confirmed hook engine to wire RTK against"
    );
    assert_eq!(verify_tool(AgentId::Warp, ToolId::Rtk), Some(true));

    unwire_tool(AgentId::Warp, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Warp, ToolId::Rtk), Some(true));
}

#[tokio::test]
async fn test_warp_rtk_wire_scrubs_legacy_bunfs_entries() {
    let _env = common::setup();
    let legacy_dir = _env.home().join(".config").join("warp");
    std::fs::create_dir_all(&legacy_dir).unwrap();
    write_file(
        &legacy_dir.join("hooks.json"),
        r#"{
            "PreToolUse": [
                { "matcher": "Bash", "hooks": [{ "type": "command", "command": "echo user-owned" }] },
                { "matcher": "Execute", "hooks": [{ "type": "command", "command": "/$bunfs/root/toksave rtk-hook warp" }] }
            ]
        }"#,
    )
    .unwrap();
    write_file(
        &legacy_dir.join("mcp.json"),
        r#"{
            "mcpServers": {
                "codegraph": { "command": "/$bunfs/root/toksave", "args": ["runmcp", "codegraph", "serve", "--mcp"] },
                "other": { "command": "/usr/bin/other-mcp" }
            }
        }"#,
    )
    .unwrap();

    wire_tool(AgentId::Warp, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();

    let hooks_cfg = read_json_file(&legacy_dir.join("hooks.json"))
        .unwrap()
        .unwrap();
    let arr = hooks_cfg["PreToolUse"].as_array().expect("PreToolUse");
    assert_eq!(arr.len(), 1, "only the dead bunfs entry should be scrubbed");
    assert_eq!(
        arr[0]["hooks"][0]["command"].as_str().unwrap(),
        "echo user-owned"
    );

    let mcp_cfg = read_json_file(&legacy_dir.join("mcp.json"))
        .unwrap()
        .unwrap();
    let servers = mcp_cfg["mcpServers"].as_object().expect("mcpServers");
    assert!(
        !servers.contains_key("codegraph"),
        "dead bunfs mcp entry should be scrubbed"
    );
    assert!(
        servers.contains_key("other"),
        "user-owned mcp entry should survive"
    );
}

#[tokio::test]
async fn test_copilot_rtk_writes_native_hooks_file() {
    let _env = common::setup();
    let p = copilot_paths();
    // Simulate the pre-fix typo'd file to confirm it gets cleaned up on wire.
    std::fs::create_dir_all(&p.hooks_dir).unwrap();
    write_file(&p.hooks_dir.join("tokless-rtk.json"), "{}").unwrap();

    wire_tool(AgentId::Copilot, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();

    assert!(!p.hooks_dir.join("tokless-rtk.json").exists());
    let cfg = read_json_file(&p.hooks_dir.join("toksave-rtk.json"))
        .unwrap()
        .unwrap();
    let arr = cfg["hooks"]["preToolUse"].as_array().expect("preToolUse");
    assert!(arr.iter().any(|h| {
        h.get("matcher").and_then(|m| m.as_str()) == Some("bash")
            && h.get("command")
                .and_then(|c| c.as_str())
                .is_some_and(|c| c.contains("rtk-hook copilot"))
    }));

    unwire_tool(AgentId::Copilot, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();
    assert_eq!(verify_tool(AgentId::Copilot, ToolId::Rtk), Some(false));
}

#[test]
fn copilot_empty_rtk_hook_file_is_not_wired() {
    let _env = common::setup();
    let p = copilot_paths();
    std::fs::create_dir_all(&p.hooks_dir).unwrap();
    write_file(&p.hooks_dir.join("toksave-rtk.json"), "{}").unwrap();
    assert_eq!(
        verify_tool(AgentId::Copilot, ToolId::Rtk),
        Some(false),
        "an empty toksave-rtk.json must not count as wired"
    );
}

#[tokio::test]
async fn test_claude_rtk_strips_dangling_rtk_ref_from_claude_md() {
    let _env = common::setup();
    let p = claude_paths();
    std::fs::create_dir_all(&p.dir).unwrap();
    write_file(&p.claude_md, "# Notes\n\n@RTK.md\n").unwrap();

    wire_tool(AgentId::Claude, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();

    let contents = std::fs::read_to_string(&p.claude_md).unwrap();
    assert!(
        !contents.contains("@RTK.md"),
        "dangling @RTK.md ref should be stripped from CLAUDE.md, got: {contents}"
    );
    assert!(contents.contains("# Notes"));
}

#[tokio::test]
async fn test_droid_rtk_wire_cleans_up_legacy_factory_droid_dir() {
    let _env = common::setup();
    let legacy = toksave::util::paths::droid_legacy_hooks_file();
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    write_file(
        &legacy,
        r#"{
            "PreToolUse": [
                { "matcher": "Bash", "hooks": [{ "type": "command", "command": "echo user-owned" }] },
                { "matcher": "Execute", "hooks": [{ "type": "command", "command": "/old/toksave rtk-hook droid" }] }
            ]
        }"#,
    )
    .unwrap();

    wire_tool(AgentId::Droid, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();

    let cfg = read_json_file(&legacy).unwrap().unwrap();
    let arr = cfg["PreToolUse"].as_array().expect("PreToolUse");
    assert_eq!(
        arr.len(),
        1,
        "only the stale toksave entry should be scrubbed"
    );
    assert_eq!(
        arr[0]["hooks"][0]["command"].as_str().unwrap(),
        "echo user-owned"
    );
}

#[tokio::test]
async fn test_devin_rtk_wire_cleans_up_legacy_devin_hooks_file() {
    let _env = common::setup();
    let legacy = devin_paths().hooks_file;
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    write_file(
        &legacy,
        r#"{
            "PreToolUse": [
                { "matcher": "X", "hooks": [{ "type": "command", "command": "echo user-owned" }] },
                { "matcher": "Execute", "hooks": [{ "type": "command", "command": "/old/toksave rtk-hook devin" }] }
            ]
        }"#,
    )
    .unwrap();

    wire_tool(AgentId::Devin, ToolId::Rtk, &RunOpts::default())
        .await
        .unwrap();

    let cfg = read_json_file(&legacy).unwrap().unwrap();
    let arr = cfg["PreToolUse"].as_array().expect("PreToolUse");
    assert_eq!(
        arr.len(),
        1,
        "only the stale toksave entry should be scrubbed"
    );
    assert_eq!(
        arr[0]["hooks"][0]["command"].as_str().unwrap(),
        "echo user-owned"
    );
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
