mod common;

use common::setup;
use toksave::cli::{CommandType, parse_cli};
use toksave::commands::disable::run_disable;
use toksave::util::manifest::{read_manifest, record_wire, was_wired_by_us};
use toksave::util::paths::local_bin;

#[tokio::test]
async fn disable_parses_command() {
    let _env = setup();
    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "disable".to_string(),
    ]);
    assert_eq!(parsed.command, CommandType::Disable);
}

#[tokio::test]
async fn disable_with_no_agents_returns_zero() {
    let _env = setup();
    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "disable".to_string(),
    ]);
    let code = run_disable(&parsed).await;
    assert_eq!(code, 0);
}

#[tokio::test]
async fn disable_marks_manifest_disabled_and_keeps_binaries() {
    let env = setup();

    // Fake detected agent.
    let claude_dir = env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    // Fake rtk binary in local bin — disable must NOT remove it.
    let bin = local_bin();
    std::fs::create_dir_all(&bin).unwrap();
    let rtk_path = bin.join(if cfg!(windows) { "rtk.exe" } else { "rtk" });
    std::fs::write(&rtk_path, b"fake").unwrap();

    record_wire("claude", "rtk", Some("1.0.0")).unwrap();
    assert!(was_wired_by_us("claude", "rtk"));

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "disable".to_string(),
    ]);
    let code = run_disable(&parsed).await;
    assert_eq!(code, 0);

    // Entry still present, but marked disabled.
    let m = read_manifest();
    let entry = m
        .entries
        .iter()
        .find(|e| e.agent == "claude" && e.tool == "rtk")
        .expect("entry retained on disable");
    assert_eq!(entry.state.as_deref(), Some("disabled"));

    // Binary untouched.
    assert!(rtk_path.exists());
}

#[tokio::test]
async fn disable_dry_run_makes_no_changes() {
    let env = setup();
    let claude_dir = env.home().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    record_wire("claude", "rtk", Some("1.0.0")).unwrap();

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "--dry-run".to_string(),
        "disable".to_string(),
    ]);
    let code = run_disable(&parsed).await;
    assert_eq!(code, 0);

    let m = read_manifest();
    let entry = m
        .entries
        .iter()
        .find(|e| e.agent == "claude" && e.tool == "rtk")
        .expect("entry retained under dry-run");
    assert_ne!(entry.state.as_deref(), Some("disabled"));
}
