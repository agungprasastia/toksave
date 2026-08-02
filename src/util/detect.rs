use std::env;
use std::path::{Path, PathBuf};

fn path_var() -> Vec<PathBuf> {
    env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect()
}

fn exe_candidates(name: &str) -> Vec<String> {
    if cfg!(windows) {
        let pathext = env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
        let mut v = vec![name.to_string()];
        for ext in pathext.split(';') {
            if !ext.is_empty() {
                v.push(format!("{name}{}", ext.to_lowercase()));
            }
        }
        v
    } else {
        vec![name.to_string()]
    }
}

fn exists_executable(dir: &Path, name: &str) -> Option<String> {
    for cand in exe_candidates(name) {
        let p = dir.join(&cand);
        if p.is_file() && is_executable(&p) {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_p: &Path) -> bool {
    true
}

pub fn is_on_path(name: &str) -> bool {
    find_binary(name).is_some()
}

pub fn find_binary(name: &str) -> Option<String> {
    for dir in path_var() {
        if let Some(found) = exists_executable(&dir, name) {
            return Some(found);
        }
    }
    None
}

pub fn find_binary_in(name: &str, extra_dirs: &[PathBuf]) -> Option<String> {
    if let Some(found) = find_binary(name) {
        return Some(found);
    }
    for dir in extra_dirs {
        if let Some(found) = exists_executable(dir, name) {
            return Some(found);
        }
    }
    None
}

/// Directories where tools land that may not be on a stale PATH (port of
/// tokless ExpectedBinDirs, minus the tokless-specific dir).
pub fn expected_bin_dirs() -> Vec<PathBuf> {
    let h = crate::util::paths::home();
    let mut v = vec![h.join(".local").join("bin"), h.join(".bun").join("bin")];
    if cfg!(windows)
        && let Some(local) = env::var_os("LOCALAPPDATA")
    {
        v.push(PathBuf::from(local).join("Programs").join("toksave"));
    }
    v
}

/// Prepend existing expected bin dirs to this process's PATH so freshly
/// installed tools are found without a shell restart.
pub fn ensure_process_path() {
    let Some(path) = env::var_os("PATH") else {
        return;
    };
    let mut parts = env::split_paths(&path).collect::<Vec<_>>();
    let mut changed = false;
    for dir in expected_bin_dirs() {
        if dir.is_dir() && !parts.contains(&dir) {
            parts.insert(0, dir);
            changed = true;
        }
    }
    if changed && let Ok(p) = env::join_paths(&parts) {
        // Safe: called once at CLI startup (main thread, no other threads yet).
        unsafe { env::set_var("PATH", p) };
    }
}

pub fn resolve_node() -> Option<String> {
    if let Some(found) = find_binary("node") {
        return Some(found);
    }
    let extra: Vec<PathBuf> = if cfg!(windows) {
        [
            "C:\\Program Files\\nodejs",
            "C:\\Program Files (x86)\\nodejs",
        ]
        .iter()
        .map(PathBuf::from)
        .collect()
    } else {
        ["/usr/local/bin", "/usr/bin", "/opt/homebrew/bin"]
            .iter()
            .map(PathBuf::from)
            .collect()
    };
    find_binary_in("node", &extra)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_binary_in_checks_extra_dirs() {
        let tmp = env::temp_dir().join("toksave-detect-test");
        fs::create_dir_all(&tmp).unwrap();
        let exe = if cfg!(windows) {
            "fakebin.exe"
        } else {
            "fakebin"
        };
        fs::write(tmp.join(exe), "x").unwrap();
        let found = find_binary_in("fakebin", std::slice::from_ref(&tmp));
        assert!(found.is_some());
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_node_returns_path() {
        // node may not exist on all dev machines; when it does, path must be non-empty
        if let Some(p) = resolve_node() {
            assert!(!p.is_empty());
        }
    }

    #[test]
    fn ensure_process_path_prepends_existing_dirs() {
        let _g = crate::util::env_test_lock();
        let tmp = env::temp_dir().join("toksave-path-test");
        let local_bin = tmp.join(".local").join("bin");
        fs::create_dir_all(&local_bin).unwrap();
        let old_home = env::var_os("HOME");
        let old_path = env::var_os("PATH");
        // Safe: serialized by env_test_lock against other env-mutating tests.
        unsafe {
            env::set_var("HOME", &tmp);
            env::set_var("PATH", tmp.join("other"));
        }
        ensure_process_path();
        let parts: Vec<_> = env::split_paths(&env::var_os("PATH").unwrap()).collect();
        assert!(parts.contains(&local_bin));
        let i_lb = parts.iter().position(|p| p == &local_bin).unwrap();
        let i_other = parts.iter().position(|p| p.ends_with("other")).unwrap();
        assert!(i_lb < i_other);
        unsafe {
            if let Some(o) = old_home {
                env::set_var("HOME", o);
            } else {
                env::remove_var("HOME");
            }
            if let Some(o) = old_path {
                env::set_var("PATH", o);
            } else {
                env::remove_var("PATH");
            }
        }
        fs::remove_dir_all(&tmp).ok();
    }
}
