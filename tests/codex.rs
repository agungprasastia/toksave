mod common;

use common::setup;
use std::fs;
use toksave::agents::Agent;
use toksave::agents::codex::CodexAgent;
use toksave::registry::{RunOpts, ToolId};
use toksave::util::json::{read_json_file, write_json_file};
use toksave::util::paths::codex_paths;

const OPTS: RunOpts = RunOpts {
    dry_run: false,
    upgrade: false,
    verbose: false,
    yes: true,
    report: None,
};

#[test]
fn codex_rtk_unwire_keeps_user_pretooluse_hook() {
    let _env = setup();
    let p = codex_paths();
    fs::create_dir_all(&p.dir).unwrap();
    write_json_file(
        &p.hooks,
        &serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{ "type": "command", "command": "my-own-linter" }]
                }]
            }
        }),
    )
    .unwrap();

    let agent = CodexAgent;
    agent.wire(ToolId::Rtk, &OPTS).unwrap();
    agent.unwire(ToolId::Rtk, &OPTS).unwrap();

    let cfg = read_json_file(&p.hooks).unwrap().unwrap();
    assert_eq!(
        cfg["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "my-own-linter"
    );
}

#[test]
fn codex_principles_unwire_rejects_non_object_hooks() {
    let _env = setup();
    let p = codex_paths();
    fs::create_dir_all(&p.dir).unwrap();
    write_json_file(&p.hooks, &serde_json::json!({ "hooks": [] })).unwrap();
    let before = fs::read_to_string(&p.hooks).unwrap();

    assert!(CodexAgent.unwire(ToolId::Principles, &OPTS).is_err());
    assert_eq!(fs::read_to_string(&p.hooks).unwrap(), before);
}
