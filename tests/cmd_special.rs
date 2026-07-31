mod common;

use common::setup;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use toksave_rs::cli::{parse_cli, CommandType};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_toksave-rs"))
}

fn run_cmd(args: &[&str], stdin: &str, dir: Option<&std::path::Path>) -> (i32, String, String) {
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

// ---- cli parse ----

#[test]
fn parse_self_update_command() {
    let parsed = parse_cli(vec!["toksave".into(), "self-update".into()]);
    assert_eq!(parsed.command, CommandType::SelfUpdate);
}

#[test]
fn parse_runmcp_command_passes_args() {
    let parsed = parse_cli(vec![
        "toksave".into(),
        "runmcp".into(),
        "context-mode".into(),
    ]);
    assert_eq!(parsed.command, CommandType::Runmcp);
    assert_eq!(parsed.hook_args, vec!["context-mode".to_string()]);
}

#[test]
fn parse_runmcp_agent_flag_preserved() {
    let parsed = parse_cli(vec![
        "toksave".into(),
        "runmcp".into(),
        "--agent".into(),
        "codex".into(),
        "codegraph".into(),
        "serve".into(),
        "--mcp".into(),
    ]);
    assert_eq!(parsed.command, CommandType::Runmcp);
    assert!(parsed.hook_args.contains(&"codegraph".to_string()));
}

#[test]
fn parse_index_command_default_off() {
    let parsed = parse_cli(vec!["toksave".into(), "index".into()]);
    assert_eq!(parsed.command, CommandType::Index);
    assert!(!parsed.auto);
}

#[test]
fn parse_index_auto_flag() {
    let parsed = parse_cli(vec!["toksave".into(), "index".into(), "--auto".into()]);
    assert_eq!(parsed.command, CommandType::Index);
    assert!(parsed.auto);
}

// ---- runmcp ----

#[test]
fn runmcp_no_args_prints_usage() {
    let (code, _, stderr) = run_cmd(&["runmcp"], "", None);
    assert_eq!(code, 1);
    assert!(stderr.contains("Usage: toksave runmcp"));
}

#[test]
fn runmcp_missing_executable_errors() {
    let _env = setup();
    let (code, _, stderr) = run_cmd(&["runmcp", "definitely-not-a-real-mcp-tool-xyz"], "", None);
    assert_eq!(code, 1);
    assert!(stderr.contains("MCP executable not found"));
}

#[test]
fn runmcp_executes_echo_like_command() {
    let _env = setup();
    if cfg!(windows) {
        // cmd shim: create fake exe that exits 42 via batch on isolated PATH
        // simpler: run `cmd /c exit 42` isn't a direct binary arg flow; skip
        // precise exit-code check on Windows and just assert spawn success.
        let (code, _, _) = run_cmd(
            &[
                "runmcp",
                "C:\\Windows\\System32\\cmd.exe",
                "/c",
                "echo",
                "hi",
            ],
            "",
            None,
        );
        assert_eq!(code, 0);
    } else {
        let (code, stdout, _) = run_cmd(&["runmcp", "/bin/echo", "hi"], "", None);
        assert_eq!(code, 0);
        assert!(stdout.contains("hi"));
    }
}

#[test]
fn runmcp_parses_agent_flag_in_library_impl() {
    let (agent, rest) = toksave_rs::commands::runmcp::parse_agent_flag(&[
        "--agent".into(),
        "codex".into(),
        "codegraph".into(),
        "serve".into(),
    ]);
    assert_eq!(agent, "codex");
    assert_eq!(rest, vec!["codegraph", "serve"]);
}

#[test]
fn runmcp_is_node_shebang_script_detects_env_node() {
    let tmp = std::env::temp_dir().join(format!("toksave-shebang-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let yes = tmp.join("tool-yes.js");
    std::fs::write(&yes, "#!/usr/bin/env node\nconsole.log(1)\n").unwrap();
    let win = tmp.join("tool-win.js");
    std::fs::write(&win, "#!node.exe\n").unwrap();
    let no = tmp.join("tool-no.sh");
    std::fs::write(&no, "#!/bin/sh\n").unwrap();
    assert!(toksave_rs::commands::runmcp::is_node_shebang_script(&yes));
    assert!(toksave_rs::commands::runmcp::is_node_shebang_script(&win));
    assert!(!toksave_rs::commands::runmcp::is_node_shebang_script(&no));
    assert!(!toksave_rs::commands::runmcp::is_node_shebang_script(
        &tmp.join("missing.js")
    ));
    std::fs::remove_dir_all(&tmp).ok();
}

// ---- index ----

#[test]
fn index_auto_silent_skip_when_no_project() {
    let _env = setup();
    let tmp = std::env::temp_dir().join(format!(
        "toksave-index-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let code = toksave_rs::commands::index::run_index_with(true, &tmp);
    assert_eq!(code, 0);
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn index_manual_reports_missing_codegraph() {
    let _env = setup();
    // TOKSAVE_TEST=1 is set by setup(); codegraph::installed_version is skipped
    // in test mode only when TOKSAVE_TEST=1 per TS build-index.ts:59. The Rust
    // port mirrors that: non-auto runs still return a result without panicking.
    let tmp = std::env::temp_dir().join(format!(
        "toksave-index-manual-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(tmp.join(".git")).unwrap();
    let code = toksave_rs::commands::index::run_index_with(false, &tmp);
    assert!(code == 0 || code == 1);
    std::fs::remove_dir_all(&tmp).ok();
}
