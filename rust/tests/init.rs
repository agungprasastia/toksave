use std::fs;
use std::path::PathBuf;
use toksave_rs::cli::{parse_cli, CommandType};
use toksave_rs::commands::init::run_init;
use toksave_rs::util::json::read_json_file;
use toksave_rs::util::manifest::was_wired_by_us;
use toksave_rs::util::paths::claude_paths;

fn test_root() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-init-test-{}", std::process::id()))
}

fn set_env() -> Option<std::ffi::OsString> {
    let _old_home = std::env::var_os("HOME");
    std::env::set_var("HOME", test_root().join("home"));
    std::env::set_var("USERPROFILE", test_root().join("home"));
    std::env::set_var("TOKSAVE_CACHE_DIR", test_root().join("cache"));
    std::env::set_var("TOKSAVE_TEST", "1");
    std::env::var_os("PATH")
}

fn restore_env(old_home: Option<std::ffi::OsString>, old_path: Option<std::ffi::OsString>) {
    match old_home {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    std::env::remove_var("USERPROFILE");
    match old_path {
        Some(v) => std::env::set_var("PATH", v),
        None => std::env::remove_var("PATH"),
    }
    std::env::remove_var("TOKSAVE_CACHE_DIR");
    std::env::remove_var("TOKSAVE_TEST");
    std::env::remove_var("TOKSAVE_TEST_RTK_INSTALL");
    fs::remove_dir_all(test_root()).ok();
}

#[tokio::test]
async fn init_wires_claude_rtk_and_records_manifest() {
    let (old_home, old_path) = (set_env(), None::<std::ffi::OsString>);
    std::env::set_var("TOKSAVE_TEST_RTK_INSTALL", "1");

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

    restore_env(old_home, old_path);
}

#[tokio::test]
async fn init_dry_run_does_not_write() {
    let (old_home, old_path) = (set_env(), None::<std::ffi::OsString>);
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
    restore_env(old_home, old_path);
}
