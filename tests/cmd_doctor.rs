mod common;

use common::setup;
use toksave::registry::{AgentId, ToolId, verify_tool};

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
