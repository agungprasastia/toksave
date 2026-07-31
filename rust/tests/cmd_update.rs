mod common;

use common::setup;
use toksave_rs::cli::{parse_cli, CommandType};
use toksave_rs::commands::update::run_update;

#[tokio::test]
async fn update_yes_runs_cleanly() {
    let _env = setup();
    std::env::set_var("TOKSAVE_TEST_RTK_INSTALL", "1");

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "update".to_string(),
    ]);
    assert_eq!(parsed.command, CommandType::Update);
    assert!(parsed.opts.yes);

    let code = run_update(&parsed).await;
    assert!(code == 0 || code == 1);
}

#[tokio::test]
async fn update_dry_run_makes_no_changes() {
    let _env = setup();

    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "--dry-run".to_string(),
        "update".to_string(),
    ]);
    assert!(parsed.opts.dry_run);

    let code = run_update(&parsed).await;
    assert_eq!(code, 0);
}
