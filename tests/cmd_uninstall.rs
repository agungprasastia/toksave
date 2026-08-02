mod common;

use common::setup;
use toksave::cli::{CommandType, parse_cli};
use toksave::commands::uninstall::run_uninstall;
use toksave::util::manifest::{read_manifest, record_wire, was_wired_by_us};

#[tokio::test]
async fn uninstall_dry_run_makes_no_changes() {
    let _env = setup();

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "--dry-run".to_string(),
        "uninstall".to_string(),
    ]);
    assert_eq!(parsed.command, CommandType::Uninstall);
    assert!(parsed.opts.dry_run);

    let code = run_uninstall(&parsed).await;
    assert_eq!(code, 0);
}

#[tokio::test]
async fn uninstall_with_no_agents_detected_returns_zero() {
    let _env = setup();

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "uninstall".to_string(),
    ]);

    let code = run_uninstall(&parsed).await;
    assert_eq!(code, 0);
}

#[tokio::test]
async fn uninstall_removes_manifest_wires() {
    let env = setup();

    // Fake a detected agent: claude path presence triggers detection in test mode.
    let home = env.home();
    let claude_dir = home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    // Pre-wire an entry in the manifest to simulate a prior install.
    record_wire("claude", "rtk", Some("1.0.0")).unwrap();
    assert!(was_wired_by_us("claude", "rtk"));

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "uninstall".to_string(),
    ]);

    let code = run_uninstall(&parsed).await;
    assert_eq!(code, 0);
    assert!(!was_wired_by_us("claude", "rtk"));
    assert!(read_manifest().entries.is_empty());
}
