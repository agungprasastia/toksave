mod common;

use common::setup;
use std::fs;
use toksave::cli::{CommandType, parse_cli};
use toksave::commands::init::run_init;
use toksave::util::json::read_json_file;
use toksave::util::manifest::was_wired_by_us;
use toksave::util::paths::claude_paths;

#[tokio::test]
async fn init_wires_claude_rtk_and_records_manifest() {
    let _env = setup();
    // Safe: serialized by common::ENV_LOCK via setup().
    unsafe { std::env::set_var("TOKSAVE_TEST_RTK_INSTALL", "1") };

    // Simulate Claude installed via config dir (test mode detect)
    fs::create_dir_all(claude_paths().dir).unwrap();

    let args = vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "--agents".to_string(),
        "claude".to_string(),
        "--tools".to_string(),
        "rtk".to_string(),
    ];
    let parsed = parse_cli(args);
    assert_eq!(parsed.command, CommandType::Init);

    let code = run_init(&parsed).await;

    assert_eq!(code, 0);
    assert!(was_wired_by_us("claude", "rtk"));
    assert!(
        read_json_file(&claude_paths().settings).unwrap().unwrap()["hooks"]["PreToolUse"]
            .is_array()
    );
}

#[tokio::test]
async fn init_dry_run_does_not_write() {
    let _env = setup();
    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--dry-run".to_string(),
        "--yes".to_string(),
        "--agents".to_string(),
        "claude".to_string(),
        "--tools".to_string(),
        "rtk".to_string(),
    ]);
    let code = run_init(&parsed).await;
    assert!(code == 0 || code == 1);
    assert!(!claude_paths().settings.exists());
}
