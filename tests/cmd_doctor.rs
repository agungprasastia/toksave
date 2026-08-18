mod common;

use common::setup;
use toksave::registry::{AgentId, ToolId, verify_tool};
use toksave::util::json::read_json_file;
use toksave::util::paths::{claude_paths, toksave_abs};

#[tokio::test]
async fn doctor_offline_runs_cleanly() {
    let _env = setup();

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
    ]);
    assert_eq!(parsed.command, toksave::cli::CommandType::Doctor);
    assert!(parsed.offline);

    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);
}

#[tokio::test]
async fn doctor_offline_with_fix_runs_cleanly() {
    let _env = setup();

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
        "--fix".to_string(),
    ]);
    assert!(parsed.fix);

    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);
}

#[tokio::test]
async fn doctor_fix_repairs_missing_wiring_not_just_tool_binaries() {
    let _env = setup();
    // Claude is "installed" (config dir present in test mode) but Principles was never
    // wired -- instruction_only, so it doesn't need a real binary to repair.
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    assert_eq!(
        verify_tool(AgentId::Claude, ToolId::Principles),
        Some(false)
    );

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
        "--fix".to_string(),
    ]);
    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);

    assert_eq!(
        verify_tool(AgentId::Claude, ToolId::Principles),
        Some(true),
        "doctor --fix should actually rewire missing tools, not just report them"
    );
}

#[tokio::test]
async fn doctor_without_fix_does_not_repair_missing_wiring() {
    let _env = setup();
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
    ]);
    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);

    assert_eq!(
        verify_tool(AgentId::Claude, ToolId::Principles),
        Some(false),
        "plain doctor (no --fix) must not mutate wiring"
    );
}

#[tokio::test]
async fn doctor_fix_does_not_wire_tool_whose_binary_was_never_installed() {
    // Codegraph is npm-channel; with the test's isolated (empty) PATH it's never
    // "installed", so repair must not silently wire an agent config that points at nothing.
    let _env = setup();
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
        "--fix".to_string(),
    ]);
    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);
    assert_eq!(
        verify_tool(AgentId::Claude, ToolId::Codegraph),
        Some(false),
        "repair must not wire a tool whose binary was never actually installed"
    );
}

#[tokio::test]
async fn doctor_fix_records_manifest_entry_on_repair() {
    let _env = setup();
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    assert!(!toksave::util::manifest::was_wired_by_us(
        "claude",
        "principles"
    ));

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
        "--fix".to_string(),
    ]);
    toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;

    assert!(toksave::util::manifest::was_wired_by_us(
        "claude",
        "principles"
    ));
}

fn plant_fake_codegraph(env: &common::TestEnvGuard) {
    let bin = env.root.join("empty-bin");
    std::fs::create_dir_all(&bin).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = bin.join("codegraph");
        std::fs::write(&path, "#!/bin/sh\necho 0.0.0\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    {
        std::fs::write(bin.join("codegraph.cmd"), "@echo 0.0.0\r\n").unwrap();
    }
}

#[test]
fn bunfs_mcp_entry_is_not_considered_wired() {
    let _env = setup();
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let global = claude_paths().global_json;
    std::fs::write(
        &global,
        r#"{
            "mcpServers": {
                "codegraph": {
                    "command": "/$bunfs/root/toksave",
                    "args": ["runmcp", "codegraph", "serve", "--mcp"]
                }
            }
        }"#,
    )
    .unwrap();

    assert_eq!(
        verify_tool(AgentId::Claude, ToolId::Codegraph),
        Some(false),
        "a leftover /$bunfs/ MCP command must not count as wired"
    );
}

#[tokio::test]
async fn doctor_fix_rewrites_bunfs_mcp_command_to_live_toksave() {
    let _env = setup();
    plant_fake_codegraph(&_env);
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let global = claude_paths().global_json;
    std::fs::write(
        &global,
        r#"{
            "mcpServers": {
                "codegraph": {
                    "command": "/$bunfs/root/toksave",
                    "args": ["runmcp", "codegraph", "serve", "--mcp"]
                }
            }
        }"#,
    )
    .unwrap();
    assert_eq!(verify_tool(AgentId::Claude, ToolId::Codegraph), Some(false));

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
        "--fix".to_string(),
    ]);
    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);
    assert_eq!(
        verify_tool(AgentId::Claude, ToolId::Codegraph),
        Some(true),
        "doctor --fix should rewrite bunfs MCP entries to the live toksave binary"
    );

    let cfg = read_json_file(&global).unwrap().unwrap();
    let cmd = cfg["mcpServers"]["codegraph"]["command"]
        .as_str()
        .unwrap_or("");
    assert_eq!(cmd, toksave_abs());
    assert!(!cmd.contains("/$bunfs/"));
}

#[tokio::test]
async fn doctor_fix_removes_stacked_native_claude_rtk_hook() {
    let _env = setup();
    let claude_dir = _env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_paths().settings,
        format!(
            r#"{{
            "hooks": {{
                "PreToolUse": [
                    {{
                        "matcher": "Bash",
                        "hooks": [{{ "type": "command", "command": "{} rtk-hook claude", "timeout": 10 }}]
                    }},
                    {{
                        "matcher": "Bash",
                        "hooks": [{{ "type": "command", "command": "rtk hook claude", "timeout": 10 }}]
                    }}
                ]
            }}
        }}"#,
            toksave_abs()
        ),
    )
    .unwrap();
    assert_eq!(verify_tool(AgentId::Claude, ToolId::Rtk), Some(false));

    let parsed = toksave::cli::parse_cli(vec![
        "toksave".to_string(),
        "doctor".to_string(),
        "--offline".to_string(),
        "--fix".to_string(),
    ]);
    let code = toksave::commands::doctor::run_doctor(&parsed, parsed.offline, parsed.fix).await;
    assert_eq!(code, 0);
    assert_eq!(verify_tool(AgentId::Claude, ToolId::Rtk), Some(true));

    let cfg = read_json_file(&claude_paths().settings).unwrap().unwrap();
    let pre = cfg["hooks"]["PreToolUse"].as_array().unwrap();
    let commands: Vec<&str> = pre
        .iter()
        .filter_map(|g| g["hooks"][0]["command"].as_str())
        .collect();
    assert!(
        commands.iter().any(|c| c.contains("rtk-hook claude")),
        "toksave wrapper must remain, got {commands:?}"
    );
    assert!(
        commands.iter().all(|c| !c.contains("rtk hook claude")),
        "native `rtk hook claude` must be removed, got {commands:?}"
    );
}
