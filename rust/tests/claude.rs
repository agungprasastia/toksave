use std::fs;
use toksave_rs::agents::claude::ClaudeAgent;
use toksave_rs::agents::Agent;
use toksave_rs::registry::{RunOpts, ToolId};
use toksave_rs::util::json::read_json_file;
use toksave_rs::util::paths::claude_paths;

fn test_env() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("toksave-claude-test-{}", std::process::id()))
}

struct SavedEnv {
    home: Option<std::ffi::OsString>,
    path: Option<std::ffi::OsString>,
    localappdata: Option<std::ffi::OsString>,
    appdata: Option<std::ffi::OsString>,
}

fn set_home() -> SavedEnv {
    let saved = SavedEnv {
        home: std::env::var_os("HOME"),
        path: std::env::var_os("PATH"),
        localappdata: std::env::var_os("LOCALAPPDATA"),
        appdata: std::env::var_os("APPDATA"),
    };
    let te = test_env();
    let empty_bin = te.join("empty-bin");
    fs::create_dir_all(&empty_bin).ok();
    std::env::set_var("HOME", te.join("home"));
    std::env::set_var("USERPROFILE", te.join("home"));
    std::env::set_var("TOKSAVE_TEST", "1");
    // Isolate PATH so real claude/desktop binaries aren't found
    std::env::set_var("PATH", &empty_bin);
    std::env::set_var("LOCALAPPDATA", te.join("localappdata"));
    std::env::set_var("APPDATA", te.join("appdata"));
    saved
}

fn restore_home(saved: SavedEnv) {
    fn restore(key: &str, val: Option<std::ffi::OsString>) {
        match val {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
    restore("HOME", saved.home);
    restore("PATH", saved.path);
    restore("LOCALAPPDATA", saved.localappdata);
    restore("APPDATA", saved.appdata);
    std::env::remove_var("USERPROFILE");
    std::env::remove_var("TOKSAVE_TEST");
    fs::remove_dir_all(test_env()).ok();
}

const OPTS: RunOpts = RunOpts {
    dry_run: false,
    upgrade: false,
    verbose: false,
    yes: true,
};

#[test]
fn claude_wires_rtk_through_pretooluse_hook() {
    let old = set_home();
    let agent = ClaudeAgent;
    agent.wire(ToolId::Rtk, &OPTS).unwrap();
    let settings = read_json_file(&claude_paths().settings).unwrap().unwrap();
    let pre = settings["hooks"]["PreToolUse"].as_array().unwrap();
    assert!(pre.iter().any(|g| {
        g["hooks"][0]["command"]
            .as_str()
            .map(|c| c.contains("rtk-hook claude"))
            .unwrap_or(false)
    }));
    assert_eq!(agent.verify(ToolId::Rtk), Some(true));
    restore_home(old);
}

#[test]
fn claude_rtk_unwire_removes_hook() {
    let old = set_home();
    let agent = ClaudeAgent;
    agent.wire(ToolId::Rtk, &OPTS).unwrap();
    agent.unwire(ToolId::Rtk, &OPTS).unwrap();
    assert_eq!(agent.verify(ToolId::Rtk), Some(false));
    restore_home(old);
}

#[test]
fn claude_detect_uses_config_dir_in_test_mode() {
    let old = set_home();
    fs::create_dir_all(claude_paths().dir).unwrap();
    let d = ClaudeAgent.detect();
    assert!(d.installed);
    assert_eq!(d.source, "config");
    restore_home(old);
}

#[test]
fn claude_unparseable_settings_is_error_not_fallback() {
    // Trust boundary: wire must FAIL (error), not silently create {} and clobber.
    let old = set_home();
    fs::create_dir_all(claude_paths().dir).unwrap();
    fs::write(claude_paths().settings, "{ not json").unwrap();
    let agent = ClaudeAgent;
    let before = fs::read_to_string(claude_paths().settings).unwrap();
    assert!(agent.wire(ToolId::Rtk, &OPTS).is_err());
    assert_eq!(fs::read_to_string(claude_paths().settings).unwrap(), before);
    restore_home(old);
}
