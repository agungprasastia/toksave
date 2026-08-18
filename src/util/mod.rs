pub mod version;

/// Serialize tests that mutate process-global env vars (HOME, PATH, ...).
/// Cargo runs unit tests multi-threaded in one process; writers must not
/// overlap, or other tests read contaminated values.
pub fn env_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub mod colors;
pub mod detect;
pub mod download;
pub mod errors;
pub mod exec;
pub mod health;
pub mod json;
pub mod manifest;
pub mod mcp;
pub mod paths;
pub mod probe;
pub mod toml;
pub mod ui;
pub mod unified_block;
pub mod winsh;
