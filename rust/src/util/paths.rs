use crate::util::errors::Result;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    if let Some(h) = env::var_os("HOME") {
        return PathBuf::from(h);
    }
    if let Some(h) = env::var_os("USERPROFILE") {
        return PathBuf::from(h);
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub struct ClaudePaths {
    pub dir: PathBuf,
    pub global_json: PathBuf,
    pub settings: PathBuf,
    pub skills_dir: PathBuf,
    pub agents_md: PathBuf,
}

pub fn claude_paths() -> ClaudePaths {
    let h = home();
    let dir = h.join(".claude");
    ClaudePaths {
        global_json: h.join(".claude.json"),
        settings: dir.join("settings.json"),
        skills_dir: dir.join("skills"),
        agents_md: dir.join("AGENTS.md"),
        dir,
    }
}

pub fn claude_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub fn claude_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        let mut v = vec![];
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            v.push(
                PathBuf::from(local)
                    .join("AnthropicClaude")
                    .join("claude.exe"),
            );
        }
        if let Some(roam) = env::var_os("APPDATA") {
            v.push(PathBuf::from(roam).join("Claude").join("claude.exe"));
        }
        v
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/Claude.app")]
    } else {
        vec![]
    }
}

pub struct OpencodePaths {
    pub dir: PathBuf,
    pub config: PathBuf,
    pub agents_md: PathBuf,
}

pub fn opencode_paths() -> OpencodePaths {
    let h = home();
    let dir = h.join(".config").join("opencode");
    OpencodePaths {
        config: dir.join("config.json"),
        agents_md: dir.join("AGENTS.md"),
        dir,
    }
}

pub struct CodexPaths {
    pub dir: PathBuf,
    pub instructions: PathBuf,
}

pub fn codex_paths() -> CodexPaths {
    let h = home();
    let dir = h.join(".codex");
    CodexPaths {
        instructions: dir.join("instructions.md"),
        dir,
    }
}

pub struct AntigravityPaths {
    pub dir: PathBuf,
}

pub fn antigravity_paths() -> AntigravityPaths {
    let h = home();
    AntigravityPaths {
        dir: h.join(".antigravity"),
    }
}

pub struct CopilotPaths {
    pub dir: PathBuf,
    pub skills_dir: PathBuf,
}

pub fn copilot_paths() -> CopilotPaths {
    let h = home();
    let dir = h.join(".copilot");
    CopilotPaths {
        skills_dir: dir.join("skills"),
        dir,
    }
}

pub fn local_bin() -> PathBuf {
    if cfg!(windows) {
        if let Some(la) = env::var_os("LOCALAPPDATA") {
            return PathBuf::from(la).join("Programs").join("toksave");
        }
        home()
            .join("AppData")
            .join("Local")
            .join("Programs")
            .join("toksave")
    } else {
        home().join(".local").join("bin")
    }
}

pub fn cache_dir() -> PathBuf {
    if let Some(c) = env::var_os("TOKSAVE_CACHE_DIR") {
        return PathBuf::from(c);
    }
    home().join(".cache").join("toksave")
}

pub fn ensure_dir(p: &Path) -> Result<()> {
    if !p.exists() {
        fs::create_dir_all(p)?;
    }
    Ok(())
}

pub fn read_file(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

pub fn write_file(p: &Path, content: &str) -> Result<()> {
    if let Some(parent) = p.parent() {
        ensure_dir(parent)?;
    }
    let pid = std::process::id();
    let tmp = p.with_extension(format!(
        "{}.{}.tmp",
        p.extension().unwrap_or_default().to_string_lossy(),
        pid
    ));
    let mut f = fs::File::create(&tmp)?;
    f.write_all(content.as_bytes())?;
    f.flush()?;
    fs::rename(&tmp, p)?;
    Ok(())
}

pub fn toksave_abs() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "toksave".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn home_uses_env() {
        let tmp = env::temp_dir().join("toksave-paths-test-home");
        fs::create_dir_all(&tmp).unwrap();
        let old = env::var_os("HOME");
        env::set_var("HOME", &tmp);
        assert_eq!(home(), tmp);
        if let Some(o) = old {
            env::set_var("HOME", o);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn cache_dir_uses_env_override() {
        let tmp = env::temp_dir().join("toksave-cache-test");
        let old = env::var_os("TOKSAVE_CACHE_DIR");
        env::set_var("TOKSAVE_CACHE_DIR", &tmp);
        assert_eq!(cache_dir(), tmp);
        if let Some(o) = old {
            env::set_var("TOKSAVE_CACHE_DIR", o);
        } else {
            env::remove_var("TOKSAVE_CACHE_DIR");
        }
    }

    #[test]
    fn write_file_is_atomic_and_readable() {
        let p = env::temp_dir().join("toksave-write-test.txt");
        write_file(&p, "hello\n").unwrap();
        assert_eq!(read_file(&p).as_deref(), Some("hello\n"));
        fs::remove_file(&p).ok();
    }

    #[test]
    fn claude_paths_under_home() {
        let tmp = env::temp_dir().join("toksave-claude-test");
        let old = env::var_os("HOME");
        env::set_var("HOME", &tmp);
        let cp = claude_paths();
        assert_eq!(cp.dir, tmp.join(".claude"));
        assert_eq!(cp.global_json, tmp.join(".claude.json"));
        if let Some(o) = old {
            env::set_var("HOME", o);
        } else {
            env::remove_var("HOME");
        }
    }
}
