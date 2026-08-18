//! Windows shell command tokens that work in cmd.exe, Windows PowerShell 5.1,
//! and PowerShell 7 (`pwsh`) — Git Bash is not required.
//!
//! MCP `command` fields are CreateProcess argv (see `toksave_abs()`). This
//! module is for *shell strings*: hook `command` values and RTK rewrites that
//! the agent will run in the user's shell.

use std::path::Path;

/// Format `path` as the first token of a Windows shell command.
///
/// Rules (cmd.exe + powershell.exe 5.1 + pwsh 7):
/// - Use backslashes. PowerShell treats `C:/...` as the `C:` drive alias plus a
///   `/...` switch, so forward-slash absolute paths do not run.
/// - Do not prefix `&` (cmd's command separator) or use single quotes (cmd
///   treats them as literals).
/// - Unquoted when there is no whitespace (the only form all three shells run
///   as a program). Paths with spaces try the 8.3 short name; if that is
///   unavailable they are double-quoted (cmd-safe; PowerShell still needs `&`
///   for quoted programs — short names avoid that).
pub fn format_windows_shell_exe(path: &str) -> String {
    let mut s = path.replace('/', "\\");
    if s.chars().any(char::is_whitespace)
        && let Some(short) = native_short_path(&s)
        && !short.chars().any(char::is_whitespace)
    {
        s = short;
    }
    if s.chars().any(char::is_whitespace) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

/// First token for a hook/RTK shell string on this OS.
pub fn shell_exe_token(path: &Path) -> String {
    let s = path.to_string_lossy();
    if cfg!(windows) {
        format_windows_shell_exe(&s)
    } else if s.chars().any(char::is_whitespace) {
        format!("\"{s}\"")
    } else {
        s.into_owned()
    }
}

/// True when `command` already starts with this executable token (either slash
/// style, quoted or not, plus a following space or end of string).
pub fn command_starts_with_exe(command: &str, exe: &str) -> bool {
    let command = command.trim();
    let slash_alt = if exe.contains('\\') {
        exe.replace('\\', "/")
    } else {
        exe.replace('/', "\\")
    };
    for raw in [exe, slash_alt.as_str()] {
        if command == raw || command.starts_with(&format!("{raw} ")) {
            return true;
        }
        let quoted = format!("\"{raw}\"");
        if command == quoted || command.starts_with(&format!("{quoted} ")) {
            return true;
        }
    }
    false
}

#[cfg(windows)]
fn native_short_path(path: &str) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut buf = vec![0u16; 32768];
    let len =
        unsafe { kernel32::GetShortPathNameW(wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32) };
    if len == 0 || (len as usize) >= buf.len() {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}

#[cfg(windows)]
mod kernel32 {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetShortPathNameW(
            lpsz_long_path: *const u16,
            lpsz_short_path: *mut u16,
            cch_buffer: u32,
        ) -> u32;
    }
}

#[cfg(not(windows))]
fn native_short_path(_path: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_token_uses_backslashes_not_forward_slashes() {
        assert_eq!(
            format_windows_shell_exe(r"C:/Users/me/AppData/Local/Programs/toksave/rtk.exe"),
            r"C:\Users\me\AppData\Local\Programs\toksave\rtk.exe"
        );
    }

    #[test]
    fn windows_token_without_spaces_is_unquoted() {
        let t = format_windows_shell_exe(r"C:\Users\me\rtk.exe");
        assert!(!t.contains('"'), "{t}");
        assert!(!t.starts_with('&'), "cmd treats & as a command separator");
        assert!(!t.contains('\''), "cmd treats single quotes as literals");
    }

    #[test]
    fn windows_token_with_spaces_is_double_quoted_when_no_short_name() {
        let t = format_windows_shell_exe(r"C:\Users\Jon Marien\rtk.exe");
        assert_eq!(t, r#""C:\Users\Jon Marien\rtk.exe""#);
        assert!(!t.starts_with('&'));
    }

    #[test]
    fn prefix_match_accepts_either_slash_style() {
        let exe = r"C:\Users\me\rtk.exe";
        assert!(command_starts_with_exe(
            r"C:\Users\me\rtk.exe git status",
            exe
        ));
        assert!(command_starts_with_exe(
            r"C:/Users/me/rtk.exe git status",
            exe
        ));
        assert!(command_starts_with_exe(r"rtk git status", "rtk"));
        assert!(command_starts_with_exe(r"rtk.exe git status", "rtk.exe"));
        assert!(!command_starts_with_exe("git status", exe));
    }
}
