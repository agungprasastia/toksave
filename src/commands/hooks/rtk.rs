use serde_json::{Value, json};

use super::read_stdin;

/// RTK PreToolUse hook: prefixes shell-tool commands with an `rtk` prefix so the agent
/// runs the token-saving wrapper instead of the raw command.
///
/// Each agent CLI has its own hook contract (confirmed against each agent's own docs and
/// RTK's own `hooks/README.md`, not guessed):
///   - claude / codex / droid / devin: `tool_name` is `Bash`/`Bash`/`Execute`/`exec`; response
///     is `hookSpecificOutput.updatedInput` with `permissionDecision: "allow"`.
///   - cursor: `tool_name` is `Shell`; response is a top-level `{"permission":"allow",
///     "updated_input":{...}}`. Cursor requires JSON on every path, so a no-rewrite response
///     is `{}`, not empty stdout.
///   - antigravity (Gemini CLI): `tool_name` is `run_shell_command`; response is
///     `{"decision":"allow","hookSpecificOutput":{"tool_input":{...}}}`.
///   - copilot: has its own dual input/output format entirely (see `run_copilot`).
pub fn run(agent: Option<&str>) -> i32 {
    let agent_lc = agent.unwrap_or("").to_ascii_lowercase();
    let input = read_stdin();

    if agent_lc == "copilot" {
        return run_copilot(&input);
    }

    if input.is_empty() {
        return no_rewrite(&agent_lc);
    }

    let req: Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return no_rewrite(&agent_lc),
    };

    let tool_name = req.get("tool_name").and_then(Value::as_str).unwrap_or("");
    if !is_bash_tool(&agent_lc, tool_name) {
        return no_rewrite(&agent_lc);
    }

    let command = req
        .get("tool_input")
        .and_then(|t| t.get("command"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let trimmed = command.trim();
    if trimmed.is_empty() || trimmed.starts_with("rtk ") || trimmed == "rtk" {
        return no_rewrite(&agent_lc);
    }

    let new_cmd = format!("{} {trimmed}", rtk_command_prefix());
    println!("{}", build_response(&agent_lc, &new_cmd));
    0
}

/// Absolute path to the locally-installed `rtk` binary when present (Windows-safe: forward
/// slashes, since backslashes break Git Bash hooks -- same reasoning as `toksave_abs()`).
/// Falls back to bare `rtk` when nothing is installed under toksave's managed bin dir, so a
/// system-wide install (Homebrew, cargo install, PATH-managed) still works.
fn rtk_command_prefix() -> String {
    let local = crate::tools::rtk::local_rtk_path();
    if local.exists() {
        let s = local.to_string_lossy().to_string();
        if cfg!(windows) {
            s.replace('\\', "/")
        } else {
            s
        }
    } else {
        "rtk".to_string()
    }
}

/// The exact `tool_name` each agent sends for a shell-command tool call. `None` falls back to
/// the broad matcher below for agents without a confirmed single tool name.
fn expected_tool_name(agent_lc: &str) -> Option<&'static str> {
    match agent_lc {
        "cursor" => Some("Shell"),
        "agy" => Some("run_shell_command"),
        "droid" => Some("Execute"),
        "devin" => Some("exec"),
        "claude" | "codex" => Some("Bash"),
        _ => None,
    }
}

fn is_bash_tool(agent_lc: &str, name: &str) -> bool {
    if let Some(expected) = expected_tool_name(agent_lc) {
        return name == expected;
    }
    matches!(
        name.to_ascii_lowercase().as_str(),
        "bash"
            | "shell"
            | "run_command"
            | "execute_command"
            | "cmd"
            | "sh"
            | "pwsh"
            | "run_shell_command"
            | "execute"
            | "exec"
    )
}

/// No rewrite applies on this path. Cursor requires JSON output on every path (an empty
/// stdout is not valid), so it gets an explicit `{}`; every other agent gets silent exit 0.
fn no_rewrite(agent_lc: &str) -> i32 {
    if agent_lc == "cursor" {
        println!("{{}}");
    }
    0
}

fn build_response(agent_lc: &str, new_cmd: &str) -> Value {
    match agent_lc {
        "cursor" => json!({
            "permission": "allow",
            "updated_input": { "command": new_cmd }
        }),
        "agy" => json!({
            "decision": "allow",
            "hookSpecificOutput": {
                "tool_input": { "command": new_cmd }
            }
        }),
        _ => json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "RTK auto-rewrite",
                "updatedInput": { "command": new_cmd }
            }
        }),
    }
}

/// GitHub Copilot CLI hook contract (docs.github.com/en/copilot/reference/hooks-reference):
/// input is camelCase `toolName`/`toolArgs`, where `toolArgs` is a JSON-encoded string, not a
/// nested object. Rewrite is a top-level `modifiedArgs` object alongside
/// `permissionDecision: "allow"` (not `hookSpecificOutput`/`updatedInput` -- Copilot CLI has its
/// own flat schema). The VS Code Copilot Chat extension reads a *different* config source
/// (Claude-format `settings.json`) but shares the same snake_case `tool_name`/`tool_input`
/// payload as Claude, so both are handled here for robustness.
fn run_copilot(input: &str) -> i32 {
    if input.is_empty() {
        return 0;
    }
    let req: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    if let Some(tool_name) = req.get("tool_name").and_then(Value::as_str) {
        if !tool_name.eq_ignore_ascii_case("bash") {
            return 0;
        }
        let command = req
            .get("tool_input")
            .and_then(|t| t.get("command"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some(new_cmd) = rewritten_command(command) else {
            return 0;
        };
        let out = json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "RTK auto-rewrite",
                "updatedInput": { "command": new_cmd }
            }
        });
        println!("{out}");
        return 0;
    }

    if let Some(tool_name) = req.get("toolName").and_then(Value::as_str) {
        if !tool_name.eq_ignore_ascii_case("bash") {
            return 0;
        }
        let args_str = req.get("toolArgs").and_then(Value::as_str).unwrap_or("");
        let args: Value = serde_json::from_str(args_str).unwrap_or_else(|_| json!({}));
        let command = args.get("command").and_then(Value::as_str).unwrap_or("");
        let Some(new_cmd) = rewritten_command(command) else {
            return 0;
        };
        let out = json!({
            "permissionDecision": "allow",
            "modifiedArgs": { "command": new_cmd }
        });
        println!("{out}");
        return 0;
    }

    0
}

fn rewritten_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() || trimmed.starts_with("rtk ") || trimmed == "rtk" {
        return None;
    }
    Some(format!("{} {trimmed}", rtk_command_prefix()))
}
