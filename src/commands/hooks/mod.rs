use std::io::Read;

pub mod agy;
pub mod codex_perm;
pub mod context_mode;
pub mod copilot;
pub mod rtk;

/// Read all of stdin. Returns empty string when stdin is a TTY or unreadable.
pub fn read_stdin() -> String {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return String::new();
    }
    buf
}
