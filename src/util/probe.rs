//! Runtime probe: verify wired hook/MCP commands actually resolve and run.
//! Port of tokless internal/commands/doctor_probe.go, scoped to toksave configs.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::registry::AgentId;
use crate::util::detect::find_binary;
use crate::util::paths::{
    antigravity_mcp_files, antigravity_paths, claude_paths, codex_paths, copilot_paths,
    devin_paths, droid_paths, opencode_paths, warp_paths,
};
use crate::util::toml::read_toml_file;

const MANAGED_HOOKS: &[&str] = &[
    "rtk-hook",
    "codex-perm-hook",
    "codex-perm",
    "context-mode-hook",
    "agy-hook",
    "copilot-hook",
    "index",
];
const LIVE_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub struct RuntimeIssue {
    pub kind: &'static str,
    pub detail: String,
}

/// True when the exe basename is `toksave` or `toksave.exe`.
fn is_toksave_exe(exe: &str) -> bool {
    let base = exe.replace('\\', "/");
    let base = base.rsplit('/').next().unwrap_or(&base).to_lowercase();
    base == "toksave" || base == "toksave.exe"
}

/// A command wired into an agent config is a toksave-managed hook when the
/// exe is `toksave` and the subcommand is one we ship.
pub fn is_managed_hook(fields: &[&str]) -> bool {
    if fields.len() < 2 {
        return false;
    }
    if !is_toksave_exe(fields[0]) {
        return false;
    }
    MANAGED_HOOKS.contains(&fields[1])
}

/// Unquoted Windows paths with backslashes break Git Bash hooks.
pub fn windows_bash_hostile_path(exe: &str) -> bool {
    if !cfg!(windows) {
        return false;
    }
    if exe.contains('/') {
        return false;
    }
    let b = exe.as_bytes();
    if b.len() >= 3 && b[1] == b':' && b[2] == b'\\' {
        return true;
    }
    exe.starts_with("\\\\")
}

/// Walk a parsed config, collecting every `command` entry (string or argv
/// array) with its `args`, mirroring tokless managedHookCommands extraction.
fn collect_commands(value: &Value, out: &mut Vec<(String, Vec<String>)>) {
    match value {
        Value::Object(map) => {
            if let Some(cmd) = map.get("command") {
                match cmd {
                    Value::String(s) if !s.is_empty() => {
                        let args = map
                            .get("args")
                            .and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        out.push((s.clone(), args));
                    }
                    Value::Array(arr) => {
                        let parts: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if let Some(first) = parts.first() {
                            out.push((first.clone(), parts[1..].to_vec()));
                        }
                    }
                    _ => {}
                }
            }
            for v in map.values() {
                collect_commands(v, out);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_commands(v, out);
            }
        }
        _ => {}
    }
}

/// Walk a codex config.toml, collecting every `[mcp_servers.*]` command.
fn collect_toml_commands(doc: &toml_edit::DocumentMut, out: &mut Vec<(String, Vec<String>)>) {
    let Some(servers) = doc.get("mcp_servers") else {
        return;
    };
    let Some(tbl) = servers.as_table_like() else {
        return;
    };
    for (_, item) in tbl.iter() {
        let Some(server) = item.as_table_like() else {
            continue;
        };
        let Some(cmd) = server.get("command").and_then(|c| c.as_str()) else {
            continue;
        };
        let args = server
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        out.push((cmd.to_string(), args));
    }
}

/// A command resolves when it is an existing path or findable on PATH.
fn resolve_runnable(exe: &str) -> bool {
    if exe.is_empty() {
        return false;
    }
    let has_sep = exe.contains('/')
        || exe.contains('\\')
        || (cfg!(windows) && exe.len() >= 2 && exe.as_bytes().get(1) == Some(&b':'));
    if has_sep {
        return Path::new(exe).exists() || find_binary(exe).is_some();
    }
    find_binary(exe).is_some()
}

fn short_path(p: &str) -> String {
    if p.len() <= 64 {
        return p.to_string();
    }
    format!("{}…{}", &p[..28], &p[p.len() - 28..])
}

/// Probe one wired command. Returns a problem detail, or None when fine.
fn probe_command(exe: &str, args: &[String], managed: bool) -> Option<String> {
    if exe.is_empty() {
        return Some("empty command".to_string());
    }
    if managed && windows_bash_hostile_path(exe) {
        return Some(format!(
            "backslash path breaks Git Bash hooks: {}",
            short_path(exe)
        ));
    }
    if !resolve_runnable(exe) {
        return Some(format!("binary not found: {}", short_path(exe)));
    }
    if managed {
        return live_hook(exe, args);
    }
    if args.first().map(String::as_str) == Some("runmcp")
        && let Some(inner) = args.get(1)
        && !resolve_runnable(inner)
        && (inner.contains('/') || inner.contains('\\'))
    {
        return Some(format!("mcp target not found: {}", short_path(inner)));
    }
    None
}

/// Run a managed hook with empty stdin; must exit 0 within 3s. Timeout counts
/// as fine (hook was slow, not broken) — mirror tokless liveHook.
fn live_hook(exe: &str, args: &[String]) -> Option<String> {
    let mut child = std::process::Command::new(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) => {
                if st.success() {
                    return None;
                }
                return Some(format!(
                    "hook not runnable: {} (exit {:?})",
                    short_path(exe),
                    st.code()
                ));
            }
            Ok(None) => {
                if start.elapsed() > LIVE_TIMEOUT {
                    let _ = child.kill();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return Some(format!("hook not runnable: {}", short_path(exe))),
        }
    }
}

/// Config files that may hold toksave-wired commands for an agent.
fn probe_files(agent: AgentId) -> Vec<PathBuf> {
    let mut v = Vec::new();
    match agent {
        AgentId::Claude => {
            let p = claude_paths();
            v.push(p.settings);
            v.push(p.global_json);
        }
        AgentId::Opencode => v.push(opencode_paths().config),
        AgentId::Codex => {
            let p = codex_paths();
            v.push(p.hooks);
            v.push(p.config);
        }
        AgentId::Antigravity => {
            let p = antigravity_paths();
            v.push(p.hooks);
            v.extend(antigravity_mcp_files());
        }
        AgentId::Copilot => {
            let p = copilot_paths();
            v.push(p.mcp_config);
            if let Ok(rd) = std::fs::read_dir(&p.hooks_dir) {
                v.extend(rd.flatten().map(|e| e.path()));
            }
        }
        AgentId::Droid => {
            let p = droid_paths();
            v.push(p.hooks_file);
            v.push(p.mcp_config);
        }
        AgentId::Devin => {
            let p = devin_paths();
            v.push(p.hooks_file);
            v.push(p.mcp_config);
        }
        AgentId::Warp => {
            let p = warp_paths();
            v.push(p.hooks_file);
            v.push(p.mcp_config);
        }
    }
    v.retain(|p| p.exists());
    v
}

/// Probe toksave-wired commands in the given config files.
fn probe_files_of(files: &[PathBuf]) -> Vec<RuntimeIssue> {
    let mut issues = Vec::new();
    for file in files {
        let mut spawns = Vec::new();
        if file
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
        {
            let Ok(doc) = read_toml_file(file) else {
                continue;
            };
            collect_toml_commands(&doc, &mut spawns);
        } else {
            let Ok(raw) = std::fs::read_to_string(file) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<Value>(&raw) else {
                continue;
            };
            collect_commands(&value, &mut spawns);
        }
        for (cmd, args) in spawns {
            let fields: Vec<&str> = cmd.split_whitespace().collect();
            let Some(exe) = fields.first().copied() else {
                continue;
            };
            // Only probe toksave-managed commands; skip user's own MCP entries.
            if !is_toksave_exe(exe) {
                continue;
            }
            let managed = is_managed_hook(&fields);
            let cmd_args: Vec<String> = if managed {
                fields[1..].iter().map(|s| s.to_string()).collect()
            } else {
                args
            };
            if let Some(detail) = probe_command(exe, &cmd_args, managed) {
                issues.push(RuntimeIssue {
                    kind: if managed { "hook" } else { "mcp" },
                    detail,
                });
            }
        }
    }
    issues
}

/// Probe every toksave-wired command in an agent's configs.
pub fn probe_agent(agent: AgentId) -> Vec<RuntimeIssue> {
    probe_files_of(&probe_files(agent))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn is_managed_hook_matches_toksave_commands() {
        assert!(is_managed_hook(&["toksave", "rtk-hook", "agy"]));
        assert!(is_managed_hook(&[
            r"C:\x\toksave.exe",
            "copilot-hook",
            "claude"
        ]));
        assert!(is_managed_hook(&["/usr/local/bin/toksave", "index"]));
        assert!(!is_managed_hook(&["toksave", "runmcp", "codegraph"]));
        assert!(!is_managed_hook(&["other", "rtk-hook"]));
        assert!(!is_managed_hook(&["toksave"]));
    }

    #[test]
    fn windows_bash_hostile_path_detects_windows_paths() {
        if cfg!(windows) {
            assert!(windows_bash_hostile_path(r"C:\tools\toksave.exe"));
            assert!(windows_bash_hostile_path(r"\\server\share\toksave"));
            assert!(!windows_bash_hostile_path("/usr/bin/toksave"));
            assert!(!windows_bash_hostile_path("toksave"));
        } else {
            assert!(!windows_bash_hostile_path(r"C:\tools\toksave.exe"));
        }
    }

    #[test]
    fn collect_commands_extracts_hook_and_mcp() {
        let raw = r#"{
            "hooks": {"PreToolUse": [{"hooks": [{"command": "toksave rtk-hook agy"}]}]},
            "mcpServers": {"codegraph": {"command": "/bin/toksave", "args": ["runmcp", "codegraph", "serve", "--mcp"]}}
        }"#;
        let value: Value = serde_json::from_str(raw).unwrap();
        let mut out = Vec::new();
        collect_commands(&value, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0, "toksave rtk-hook agy");
        assert!(out[0].1.is_empty());
        assert_eq!(out[1].0, "/bin/toksave");
        assert_eq!(out[1].1, vec!["runmcp", "codegraph", "serve", "--mcp"]);
    }

    #[test]
    fn probe_files_of_reports_broken_hook_path() {
        let tmp = env::temp_dir().join("toksave-probe-test");
        fs::create_dir_all(&tmp).unwrap();
        let hooks = tmp.join("hooks.json");
        fs::write(
            &hooks,
            r#"{"PreToolUse":[{"hooks":[{"command":"C:\\no\\such\\toksave.exe rtk-hook warp","timeout":10}]}]}"#,
        )
        .unwrap();
        let issues = probe_files_of(std::slice::from_ref(&hooks));
        assert!(
            !issues.is_empty(),
            "expected issue for unresolvable hook, got none"
        );
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn probe_files_of_reports_broken_codex_toml_mcp() {
        let tmp = env::temp_dir().join("toksave-probe-toml-test");
        fs::create_dir_all(&tmp).unwrap();
        let cfg = tmp.join("config.toml");
        fs::write(
            &cfg,
            r#"[mcp_servers.codegraph]
command = "/no/such/bin/toksave"
args = ["runmcp", "codegraph", "serve", "--mcp"]
"#,
        )
        .unwrap();
        let issues = probe_files_of(std::slice::from_ref(&cfg));
        assert!(
            issues.iter().any(|i| i.detail.contains("binary not found")),
            "expected issue for missing codex MCP binary, got: {issues:?}"
        );
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn probe_files_of_skips_non_toksave_mcp_entries() {
        let tmp = env::temp_dir().join("toksave-probe-skip-test");
        fs::create_dir_all(&tmp).unwrap();
        let cfg = tmp.join("settings.json");
        // Simulates a Claude config with non-toksave MCP entries (node, shell builtins)
        fs::write(
            &cfg,
            r#"{
                "mcpServers": {
                    "user-tool": {"command": "node", "args": ["server.js"]},
                    "shell-thing": {"command": "if", "args": ["true"]},
                    "bracket": {"command": "[", "args": ["-f", "x"]}
                }
            }"#,
        )
        .unwrap();
        let issues = probe_files_of(std::slice::from_ref(&cfg));
        assert!(
            issues.is_empty(),
            "non-toksave MCP entries should not be probed, got: {issues:?}"
        );
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn probe_command_reports_missing_binary() {
        let exe = format!("{}-no-such-binary-xyz", env::temp_dir().display());
        let issue = probe_command(&exe, &[], false).expect("missing binary must be reported");
        assert!(issue.contains("binary not found"), "got: {issue}");
    }

    #[test]
    fn live_hook_fails_on_bad_exit() {
        let exe = std::env::current_exe().expect("test binary");
        // forward slashes so the Windows hostile-path check doesn't fire
        let exe = exe.to_string_lossy().replace('\\', "/");
        let issue = probe_command(exe.as_str(), &["--badflag".to_string()], true)
            .expect("bad exit must be reported");
        assert!(issue.contains("hook not runnable"), "got: {issue}");
    }
}
