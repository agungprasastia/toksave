//! Live spawn of a Windows shell token through cmd.exe, Windows PowerShell 5.1,
//! and PowerShell 7. Skipped on non-Windows; on Windows, pwsh is optional.

#[cfg(windows)]
mod windows_shells {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn writable_no_space_dir() -> Option<PathBuf> {
        for candidate in [r"C:\Windows\Temp", r"C:\Temp"] {
            let dir = PathBuf::from(candidate);
            let probe = dir.join("toksave-winsh-probe.tmp");
            if fs::write(&probe, b"ok").is_ok() {
                let _ = fs::remove_file(&probe);
                return Some(dir);
            }
        }
        None
    }

    fn run_line(program: &str, args: &[&str]) -> (bool, String, String) {
        let out = Command::new(program).args(args).output();
        match out {
            Ok(o) => (
                o.status.success(),
                String::from_utf8_lossy(&o.stdout).into_owned(),
                String::from_utf8_lossy(&o.stderr).into_owned(),
            ),
            Err(e) => (false, String::new(), e.to_string()),
        }
    }

    #[test]
    fn cmd_powershell5_and_pwsh_run_backslash_token() {
        let Some(dir) = writable_no_space_dir() else {
            eprintln!("skip: no writable directory without spaces");
            return;
        };
        let script = dir.join("toksave-winsh-echo.cmd");
        fs::write(&script, "@echo off\r\necho TOKSAVE_WINSH_OK %*\r\n").unwrap();
        let token = toksave::util::winsh::shell_exe_token(&script);
        assert!(
            !token.contains('/'),
            "PowerShell 5.1 cannot run a forward-slash first token: {token}"
        );
        assert!(
            !token.contains('"') && !token.starts_with('&'),
            "unquoted backslash form is the only token cmd + PS5 + pwsh all run: {token}"
        );

        let line = format!("{token} hello");

        let (ok, stdout, stderr) = run_line("cmd.exe", &["/C", &line]);
        assert!(ok, "cmd.exe failed: {stderr}");
        assert!(
            stdout.contains("TOKSAVE_WINSH_OK"),
            "cmd.exe stdout: {stdout}"
        );

        let (ok, stdout, stderr) = run_line(
            "powershell.exe",
            &["-NoProfile", "-NonInteractive", "-Command", &line],
        );
        assert!(ok, "powershell.exe (5.1) failed: {stderr} stdout={stdout}");
        assert!(
            stdout.contains("TOKSAVE_WINSH_OK"),
            "powershell.exe stdout: {stdout}"
        );

        let (ok, stdout, stderr) = run_line(
            "pwsh",
            &["-NoProfile", "-NonInteractive", "-Command", &line],
        );
        if stderr.contains("not recognized") || stderr.contains("cannot find") {
            eprintln!("skip: pwsh (PowerShell 7) is not installed");
        } else {
            assert!(ok, "pwsh failed: {stderr} stdout={stdout}");
            assert!(stdout.contains("TOKSAVE_WINSH_OK"), "pwsh stdout: {stdout}");
        }

        let _ = fs::remove_file(&script);
    }
}

#[cfg(not(windows))]
#[test]
fn windows_shell_token_formatter_is_cmd_powershell_safe() {
    // Formatter is OS-agnostic so Linux CI still locks the contract.
    let t = toksave::util::winsh::format_windows_shell_exe(
        r"C:/Users/me/AppData/Local/Programs/toksave/toksave.exe",
    );
    assert_eq!(t, r"C:\Users\me\AppData\Local\Programs\toksave\toksave.exe");
    assert!(!t.starts_with('&'));
    assert!(!t.contains('\''));
}
