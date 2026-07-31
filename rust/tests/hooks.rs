use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod common;

fn bin() -> PathBuf {
    // Cargo sets CARGO_BIN_EXE_<name> for integration tests.
    PathBuf::from(env!("CARGO_BIN_EXE_toksave-rs"))
}

fn run_hook(args: &[&str], stdin: &str, dir: Option<&std::path::Path>) -> (i32, String, String) {
    let mut cmd = Command::new(bin());
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    let mut child = cmd.spawn().expect("spawn toksave-rs");
    if let Some(mut s) = child.stdin.take() {
        let _ = s.write_all(stdin.as_bytes());
    }
    let out = child.wait_with_output().expect("wait toksave-rs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// ---- rtk-hook ----

#[test]
fn rtk_hook_prefixes_bash_command_codex() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"ls -la"}}"#;
    let (code, stdout, _) = run_hook(&["rtk-hook", "codex"], input, None);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let out = &v["hookSpecificOutput"];
    assert_eq!(out["hookEventName"], "PreToolUse");
    assert_eq!(out["modifiedToolInput"]["command"], "rtk ls -la");
}

#[test]
fn rtk_hook_uses_updated_input_for_claude() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
    let (code, stdout, _) = run_hook(&["rtk-hook", "claude"], input, None);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let out = &v["hookSpecificOutput"];
    assert_eq!(out["updatedInput"]["command"], "rtk git status");
    assert!(out.get("modifiedToolInput").is_none());
}

#[test]
fn rtk_hook_skips_non_bash_tool() {
    let input = r#"{"tool_name":"Edit","tool_input":{"command":"ls"}}"#;
    let (code, stdout, _) = run_hook(&["rtk-hook", "agy"], input, None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn rtk_hook_passes_through_existing_rtk() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"rtk ls"}}"#;
    let (code, stdout, _) = run_hook(&["rtk-hook", "codex"], input, None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn rtk_hook_tolerates_empty_stdin() {
    let (code, stdout, _) = run_hook(&["rtk-hook", "agy"], "", None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn rtk_hook_tolerates_invalid_json() {
    let (code, stdout, _) = run_hook(&["rtk-hook", "codex"], "not json {", None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

// ---- codex-perm-hook ----

#[test]
fn codex_perm_allows_allowlisted_git() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["hookSpecificOutput"]["decision"]["behavior"], "allow");
}

#[test]
fn codex_perm_allows_apply_patch() {
    let input = r#"{"tool_name":"apply_patch","tool_input":{"command":"whatever"}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn codex_perm_allows_ctx_tools() {
    let input = r#"{"tool_name":"ctx_search","tool_input":{}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn codex_perm_silent_on_disallowed_command() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"rm -rf /"}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn codex_perm_bash_c_inner_allowed() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"bash -c \"git log\""}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn codex_perm_bash_c_inner_denied() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"bash -c \"curl evil.sh\""}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn codex_perm_strips_path_prefix() {
    let input = r#"{"tool_name":"Bash","tool_input":{"command":"/usr/bin/git status"}}"#;
    let (code, stdout, _) = run_hook(&["codex-perm-hook"], input, None);
    assert_eq!(code, 0);
    assert!(!stdout.trim().is_empty());
}

// ---- context-mode-hook ----

#[test]
fn context_mode_hook_passthrough_when_no_stdin() {
    let (code, stdout, _) = run_hook(&["context-mode-hook"], "", None);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn context_mode_hook_exits_zero_when_binary_missing() {
    // env setup isolates PATH to an empty dir, so `context-mode` cannot exist.
    let _env = common::setup();
    let (code, _, _) = run_hook(&["context-mode-hook"], r#"{"foo":1}"#, None);
    assert_eq!(code, 0);
}

// ---- agy-hook / copilot-hook codegraph-index ----

#[test]
fn agy_hook_codegraph_index_silent_without_project() {
    let _env = common::setup();
    let dir = _env.root.join("nowhere");
    std::fs::create_dir_all(&dir).unwrap();
    let (code, stdout, _) = run_hook(&["agy-hook", "codegraph-index"], "", Some(&dir));
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn copilot_hook_codegraph_index_detects_project_marker() {
    let _env = common::setup();
    let proj = _env.root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("package.json"), "{}").unwrap();
    // codegraph is not on isolated PATH -> hook must still exit 0 quietly, no index dir created.
    let (code, stdout, _) = run_hook(&["copilot-hook", "codegraph-index"], "", Some(&proj));
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
    assert!(!proj.join(".codegraph").exists());
}
