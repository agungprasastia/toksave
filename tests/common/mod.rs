#[allow(unused_imports)]
pub use toksave::util::paths as ts_paths;

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

pub struct TestEnvGuard {
    pub root: PathBuf,
    pub old: Vec<(String, Option<OsString>)>,
    _guard: MutexGuard<'static, ()>,
}

#[allow(dead_code)]
impl TestEnvGuard {
    pub fn home(&self) -> PathBuf {
        self.root.join("home")
    }
    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        // Safe: serialized by ENV_LOCK against setup().
        unsafe {
            for (k, v) in &self.old {
                match v {
                    Some(v) => std::env::set_var(k, v),
                    None => std::env::remove_var(k),
                }
            }
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Set HOME/USERPROFILE/APPDATA/LOCALAPPDATA/TOKSAVE_CACHE_DIR to a fresh temp dir.
/// Also isolates PATH to an empty dir so host binaries don't interfere with detection tests.
/// Thread-safe via static mutex. On drop, restores previous values and removes temp dir.
pub fn setup() -> TestEnvGuard {
    let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join(format!(
        "toksave-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let cache = root.join("cache");
    let empty_bin = root.join("empty-bin");
    std::fs::create_dir_all(&empty_bin).unwrap();

    let mut old = vec![];
    for k in [
        "HOME",
        "USERPROFILE",
        "APPDATA",
        "LOCALAPPDATA",
        "XDG_CONFIG_HOME",
        "TOKSAVE_CACHE_DIR",
        "PATH",
        "TOKSAVE_TEST",
        "TOKSAVE_TEST_RTK_INSTALL",
        "CURSOR_CONFIG_DIR",
    ] {
        old.push((k.to_string(), std::env::var_os(k)));
    }
    // Safe: serialized by ENV_LOCK; restored in Drop under the same lock.
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::set_var("USERPROFILE", &home);
        std::env::set_var("APPDATA", root.join("AppData").join("Roaming"));
        std::env::set_var("LOCALAPPDATA", root.join("AppData").join("Local"));
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
        std::env::set_var("TOKSAVE_CACHE_DIR", &cache);
        std::env::set_var("PATH", &empty_bin);
        std::env::set_var("TOKSAVE_TEST", "1");
        std::env::remove_var("TOKSAVE_TEST_RTK_INSTALL");
        std::env::remove_var("CURSOR_CONFIG_DIR");
    }

    TestEnvGuard {
        root,
        old,
        _guard: guard,
    }
}
