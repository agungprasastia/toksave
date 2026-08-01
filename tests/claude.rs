mod common;

use common::setup;
use std::fs;
use toksave::agents::claude::ClaudeAgent;
use toksave::agents::Agent;
use toksave::registry::{RunOpts, ToolId};
use toksave::util::json::read_json_file;
use toksave::util::paths::claude_paths;

const OPTS: RunOpts = RunOpts {
    dry_run: false,
    upgrade: false,
    verbose: false,
    yes: true,
};

#[test]
fn claude_wires_rtk_through_pretooluse_hook() {
    let _env = setup();
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
}

#[test]
fn claude_rtk_unwire_removes_hook() {
    let _env = setup();
    let agent = ClaudeAgent;
    agent.wire(ToolId::Rtk, &OPTS).unwrap();
    agent.unwire(ToolId::Rtk, &OPTS).unwrap();
    assert_eq!(agent.verify(ToolId::Rtk), Some(false));
}

#[test]
fn claude_detect_uses_config_dir_in_test_mode() {
    let _env = setup();
    fs::create_dir_all(claude_paths().dir).unwrap();
    let d = ClaudeAgent.detect();
    assert!(d.installed);
    assert_eq!(d.source, "config");
}

#[test]
fn claude_unparseable_settings_is_error_not_fallback() {
    // Trust boundary: wire must FAIL (error), not silently create {} and clobber.
    let _env = setup();
    fs::create_dir_all(claude_paths().dir).unwrap();
    fs::write(claude_paths().settings, "{ not json").unwrap();
    let agent = ClaudeAgent;
    let before = fs::read_to_string(claude_paths().settings).unwrap();
    assert!(agent.wire(ToolId::Rtk, &OPTS).is_err());
    assert_eq!(fs::read_to_string(claude_paths().settings).unwrap(), before);
}
