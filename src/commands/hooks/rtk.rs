use serde_json::{Value, json};

use super::read_stdin;

/// RTK PreToolUse hook: prefixes Bash-like tool commands with `rtk`.
/// `agent` selects the payload key (`claude` uses `updatedInput`, others `modifiedToolInput`).
pub fn run(agent: Option<&str>) -> i32 {
    let input = read_stdin();
    if input.is_empty() {
        return 0;
    }

    let req: Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    let tool_name = req.get("tool_name").and_then(Value::as_str).unwrap_or("");
    let command = req
        .get("tool_input")
        .and_then(|t| t.get("command"))
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_bash_tool(tool_name) || command.trim().is_empty() {
        return 0;
    }

    let trimmed = command.trim();
    if trimmed.starts_with("rtk ") || trimmed == "rtk" {
        return 0;
    }

    let patch = json!({ "command": format!("rtk {trimmed}") });
    let key = if agent.unwrap_or("").eq_ignore_ascii_case("claude") {
        "updatedInput"
    } else {
        "modifiedToolInput"
    };
    let out = json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            key: patch,
        }
    });
    println!("{out}");
    0
}

fn is_bash_tool(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "bash" | "run_command" | "execute_command" | "cmd" | "sh" | "pwsh"
    )
}
