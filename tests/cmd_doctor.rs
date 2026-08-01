mod common;

use common::setup;

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
