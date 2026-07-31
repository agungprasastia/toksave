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
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
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
}
