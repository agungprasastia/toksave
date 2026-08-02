use std::io::Read;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RunResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn cmd_std(cmd: &str, args: &[&str]) -> RunResult {
    run_with_timeout(&resolve_windows_cmd(cmd), args, Duration::from_secs(120))
}

/// On Windows, spawn the PATHEXT-qualified twin (`.cmd`/`.exe`) rather than a
/// bare name: spawning a bare npm shim (a POSIX shell script) fails with
/// os error 193. Non-Windows and already-qualified names pass through.
fn resolve_windows_cmd(cmd: &str) -> String {
    if !cfg!(windows) || cmd.contains('/') || cmd.contains('\\') || cmd.contains('.') {
        return cmd.to_string();
    }
    crate::util::detect::find_binary(cmd).unwrap_or_else(|| cmd.to_string())
}

fn run_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> RunResult {
    let mut child = match Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            return RunResult {
                code: -1,
                stdout: String::new(),
                stderr: format!("Failed to execute {cmd}: {e}"),
            };
        }
    };

    let stdout = match child.stdout.take() {
        Some(pipe) => std::thread::spawn(move || drain(pipe)),
        None => return fail_no_pipe(cmd),
    };
    let stderr = match child.stderr.take() {
        Some(pipe) => std::thread::spawn(move || drain(pipe)),
        None => return fail_no_pipe(cmd),
    };

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return RunResult {
                        code: -1,
                        stdout: String::new(),
                        stderr: format!("Command timed out after 120s: {cmd}"),
                    };
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return RunResult {
                    code: -1,
                    stdout: String::new(),
                    stderr: format!("Failed to execute {cmd}: {e}"),
                };
            }
        }
    };

    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    RunResult {
        code: status.code().unwrap_or(-1),
        stdout: stdout.trim().to_string(),
        stderr: stderr.trim().to_string(),
    }
}

fn fail_no_pipe(cmd: &str) -> RunResult {
    RunResult {
        code: -1,
        stdout: String::new(),
        stderr: format!("Failed to execute {cmd}: stdout/stderr pipe unavailable"),
    }
}

fn drain(mut pipe: impl Read) -> String {
    let mut buf = String::new();
    let _ = pipe.read_to_string(&mut buf);
    buf
}

pub fn run(cmd: &str, args: &[&str]) -> RunResult {
    cmd_std(cmd, args)
}

pub fn run_ok(cmd: &str, args: &[&str]) -> bool {
    run(cmd, args).code == 0
}

pub fn run_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let r = run(cmd, args);
    if r.code == 0 { Some(r.stdout) } else { None }
}

pub fn npm_cmd() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

pub fn npx_cmd() -> &'static str {
    if cfg!(windows) { "npx.cmd" } else { "npx" }
}

/// Up to `n` most recent non-empty lines of a command's output, joined with
/// newlines — the failure detail window shown when an install fails.
pub fn last_nonempty_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_ok_true_for_echo() {
        let _g = crate::util::env_test_lock();
        let r = if cfg!(windows) {
            run_ok("cmd", &["/c", "echo", "hi"])
        } else {
            run_ok("echo", &["hi"])
        };
        assert!(r);
    }

    #[test]
    fn run_stdout_echo() {
        let _g = crate::util::env_test_lock();
        if cfg!(windows) {
            assert!(run_stdout("cmd", &["/c", "echo", "hi"]).is_some());
        } else {
            assert_eq!(run_stdout("echo", &["hi"]).as_deref(), Some("hi"));
        }
    }

    #[test]
    fn run_failure_returns_nonzero() {
        let r = run("definitely-not-a-real-binary-xyz", &[]);
        assert_ne!(r.code, 0);
    }

    #[test]
    fn run_times_out() {
        let _g = crate::util::env_test_lock();
        let r = if cfg!(windows) {
            run_with_timeout("ping", &["-n", "30", "127.0.0.1"], Duration::from_secs(2))
        } else {
            run_with_timeout("sleep", &["30"], Duration::from_secs(2))
        };
        assert_eq!(r.code, -1);
        assert!(r.stderr.contains("timed out"));
    }

    #[test]
    fn last_nonempty_lines_takes_tail() {
        assert_eq!(last_nonempty_lines("a\n\nb\nc\nd\ne", 3), "c\nd\ne");
        assert_eq!(last_nonempty_lines("only", 4), "only");
        assert_eq!(last_nonempty_lines("", 4), "");
    }
}
