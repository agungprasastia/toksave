use serde_json::{Value, json};

use super::read_stdin;

const ALLOWLIST: &[&str] = &[
    "rtk",
    "toksave",
    "tokless",
    "git",
    "cd",
    "ls",
    "node",
    "npm",
    "npx",
    "context-mode",
    "codegraph",
    "cat",
    "head",
    "tail",
    "grep",
    "find",
    "pwd",
    "which",
    "echo",
    "true",
    "false",
    "bash",
];

/// Codex PermissionRequest hook: auto-allows known-safe commands.
/// Prints an allow decision JSON when matched; otherwise stays silent.
pub fn run() -> i32 {
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

    if !is_allowed(tool_name, command) {
        return 0; // fallback to manual prompt
    }

    let out = json!({
        "hookSpecificOutput": {
            "hookEventName": "PermissionRequest",
            "decision": { "behavior": "allow" },
        }
    });
    println!("{out}");
    0
}

fn first_token(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

fn strip_path(tok: &str) -> &str {
    let last_sep = tok.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let base = &tok[last_sep..];
    if cfg!(windows) {
        base.strip_suffix(".exe")
            .or_else(|| base.strip_suffix(".cmd"))
            .or_else(|| base.strip_suffix(".bat"))
            .unwrap_or(base)
    } else {
        base
    }
}

/// Port of TS: matches `bash -c "inner"` and checks the inner first token.
fn bash_inner_script_allowed(cmd: &str) -> bool {
    let rest = match cmd.split_once("-c") {
        Some((_, r)) => r.trim_start(),
        None => return false,
    };
    let quote = match rest.chars().next() {
        Some(q @ ('"' | '\'')) => q,
        _ => return false,
    };
    // Greedy like TS regex /["'](.*)["']/: inner spans first quote to last matching quote.
    let body = &rest[quote.len_utf8()..];
    let end = body.rfind(quote).unwrap_or(0);
    let tok = strip_path(first_token(body[..end].trim()));
    ALLOWLIST.contains(&tok)
}

fn is_allowed(tool_name: &str, command: &str) -> bool {
    if tool_name == "apply_patch" {
        return true;
    }
    if tool_name.starts_with("ctx_") || tool_name.starts_with("codegraph_") {
        return true;
    }
    if tool_name != "Bash" {
        return false;
    }

    let cmd = command.trim();
    if cmd.is_empty() {
        return false;
    }

    let tok = strip_path(first_token(cmd));
    if tok == "bash" || tok == "sh" {
        return bash_inner_script_allowed(cmd);
    }

    ALLOWLIST.contains(&tok)
}
