use std::process::Command;

#[derive(Debug, Clone)]
pub struct RunResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn cmd_std(cmd: &str, args: &[&str]) -> RunResult {
    let mut c = Command::new(cmd);
    c.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());
    match c.output() {
        Ok(out) => RunResult {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        },
        Err(e) => RunResult {
            code: -1,
            stdout: String::new(),
            stderr: format!("Failed to execute {cmd}: {e}"),
        },
    }
}

pub fn run(cmd: &str, args: &[&str]) -> RunResult {
    cmd_std(cmd, args)
}

pub fn run_ok(cmd: &str, args: &[&str]) -> bool {
    run(cmd, args).code == 0
}

pub fn run_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let r = run(cmd, args);
    if r.code == 0 {
        Some(r.stdout)
    } else {
        None
    }
}

pub fn npm_cmd() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

pub fn npx_cmd() -> &'static str {
    if cfg!(windows) {
        "npx.cmd"
    } else {
        "npx"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_ok_true_for_echo() {
        let r = if cfg!(windows) {
            run_ok("cmd", &["/c", "echo", "hi"])
        } else {
            run_ok("echo", &["hi"])
        };
        assert!(r);
    }

    #[test]
    fn run_stdout_echo() {
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
}
