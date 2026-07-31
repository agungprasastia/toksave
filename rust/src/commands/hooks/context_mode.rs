use std::io::Write;
use std::process::{Command, Stdio};

use super::read_stdin;

/// Context-Mode PreInvocation hook: proxies stdin to `context-mode hook ...`
/// and forwards its stdout. Never blocks the invocation on error.
pub fn run(args: &[String]) -> i32 {
    let input = read_stdin();
    if input.is_empty() {
        return 0;
    }

    let child = Command::new("context-mode")
        .arg("hook")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return 0,
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    match child.wait_with_output() {
        Ok(out) => {
            if !out.stdout.is_empty() {
                let _ = std::io::stdout().write_all(&out.stdout);
            }
            out.status.code().unwrap_or(0)
        }
        Err(_) => 0,
    }
}
