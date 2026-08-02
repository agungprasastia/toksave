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
    pub plugins_dir: PathBuf,
}

pub fn opencode_paths() -> OpencodePaths {
    let h = home();
    let dir = h.join(".config").join("opencode");
    OpencodePaths {
        config: dir.join("config.json"),
        agents_md: dir.join("AGENTS.md"),
        plugins_dir: dir.join("plugins"),
        dir,
    }
}

pub fn opencode_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub fn opencode_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            return vec![
                PathBuf::from(local)
                    .join("Programs")
                    .join("OpenCode")
                    .join("OpenCode.exe"),
            ];
        }
        vec![]
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/OpenCode.app")]
    } else {
        vec![PathBuf::from("/usr/bin/ai.opencode.desktop")]
    }
}

pub struct CodexPaths {
    pub dir: PathBuf,
    pub config: PathBuf,
    pub hooks: PathBuf,
    pub instructions: PathBuf,
}

pub fn codex_paths() -> CodexPaths {
    let h = home();
    let dir = h.join(".codex");
    CodexPaths {
        config: dir.join("config.toml"),
        hooks: dir.join("hooks.json"),
        instructions: dir.join("instructions.md"),
        dir,
    }
}

pub fn codex_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub struct AntigravityPaths {
    pub dir: PathBuf,
    pub hooks: PathBuf,
}

pub fn antigravity_paths() -> AntigravityPaths {
    let h = home();
    let gemini = h.join(".gemini");
    let dir = if gemini.exists() {
        gemini
    } else {
        h.join(".antigravity")
    };
    AntigravityPaths {
        hooks: dir.join("config").join("hooks.json"),
        dir,
    }
}

pub fn antigravity_known_bin_dirs() -> Vec<PathBuf> {
    let mut v = vec![home().join(".local").join("bin")];
    if cfg!(windows)
        && let Some(local) = env::var_os("LOCALAPPDATA")
    {
        v.push(PathBuf::from(local).join("agy").join("bin"));
    }
    v
}

pub fn antigravity_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        let mut v = vec![];
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            let p = PathBuf::from(local);
            v.push(
                p.join("Programs")
                    .join("Antigravity")
                    .join("Antigravity.exe"),
            );
            v.push(
                p.join("Programs")
                    .join("Antigravity IDE")
                    .join("Antigravity IDE.exe"),
            );
        }
        v
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/Applications/Antigravity.app"),
            PathBuf::from("/Applications/Antigravity IDE.app"),
        ]
    } else {
        vec![
            PathBuf::from("/opt/antigravity"),
            PathBuf::from("/opt/antigravity-ide"),
        ]
    }
}

pub fn antigravity_mcp_files() -> Vec<PathBuf> {
    let p = antigravity_paths();
    vec![
        p.dir.join("mcp.json"),
        p.dir.join("config").join("mcp.json"),
    ]
}

pub fn antigravity_settings_files() -> Vec<PathBuf> {
    let p = antigravity_paths();
    vec![
        p.dir.join("settings.json"),
        p.dir.join("config").join("settings.json"),
    ]
}

pub struct CopilotPaths {
    pub dir: PathBuf,
    pub hooks_dir: PathBuf,
    pub mcp_config: PathBuf,
    pub instructions: PathBuf,
    pub skills_dir: PathBuf,
}

pub fn copilot_paths() -> CopilotPaths {
    let h = home();
    let dir = h.join(".copilot");
    CopilotPaths {
        hooks_dir: dir.join("hooks"),
        mcp_config: dir.join("mcp.json"),
        instructions: dir.join("instructions.md"),
        skills_dir: dir.join("skills"),
        dir,
    }
}

pub fn copilot_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub struct DroidPaths {
    pub dir: PathBuf,
    pub hooks_file: PathBuf,
    pub mcp_config: PathBuf,
}

pub fn droid_paths() -> DroidPaths {
    let h = home();
    let dir = h.join(".factory-droid");
    DroidPaths {
        hooks_file: dir.join("hooks.json"),
        mcp_config: dir.join("mcp.json"),
        dir,
    }
}

pub fn droid_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub fn droid_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            return vec![
                PathBuf::from(local)
                    .join("Programs")
                    .join("Factory Droid")
                    .join("Factory Droid.exe"),
            ];
        }
        vec![]
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/Factory Droid.app")]
    } else {
        vec![]
    }
}

pub struct DevinPaths {
    pub dir: PathBuf,
    pub hooks_file: PathBuf,
    pub mcp_config: PathBuf,
}

pub fn devin_paths() -> DevinPaths {
    let h = home();
    let dir = h.join(".devin");
    DevinPaths {
        hooks_file: dir.join("hooks.json"),
        mcp_config: dir.join("mcp.json"),
        dir,
    }
}

pub fn devin_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub fn devin_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            return vec![
                PathBuf::from(local)
                    .join("Programs")
                    .join("Devin")
                    .join("Devin.exe"),
            ];
        }
        vec![]
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/Devin.app")]
    } else {
        vec![]
    }
}

pub struct WarpPaths {
    pub dir: PathBuf,
    pub hooks_file: PathBuf,
    pub mcp_config: PathBuf,
}

pub fn warp_paths() -> WarpPaths {
    let h = home();
    let dir = h.join(".warp");
    WarpPaths {
        hooks_file: dir.join("hooks.json"),
        mcp_config: dir.join("mcp.json"),
        dir,
    }
}

pub fn warp_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub fn warp_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            return vec![
                PathBuf::from(local)
                    .join("Programs")
                    .join("Warp")
                    .join("Warp.exe"),
            ];
        }
        vec![]
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/Warp.app")]
    } else {
        vec![PathBuf::from("/usr/bin/warp-terminal")]
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
        let _g = crate::util::env_test_lock();
        let tmp = env::temp_dir().join("toksave-paths-test-home");
        fs::create_dir_all(&tmp).unwrap();
        let old = env::var_os("HOME");
        // Safe: serialized by env_test_lock against other env-mutating tests.
        unsafe {
            env::set_var("HOME", &tmp);
        }
        assert_eq!(home(), tmp);
        unsafe {
            if let Some(o) = old {
                env::set_var("HOME", o);
            } else {
                env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn cache_dir_uses_env_override() {
        let _g = crate::util::env_test_lock();
        let tmp = env::temp_dir().join("toksave-cache-test");
        let old = env::var_os("TOKSAVE_CACHE_DIR");
        // Safe: serialized by env_test_lock against other env-mutating tests.
        unsafe {
            env::set_var("TOKSAVE_CACHE_DIR", &tmp);
        }
        assert_eq!(cache_dir(), tmp);
        unsafe {
            if let Some(o) = old {
                env::set_var("TOKSAVE_CACHE_DIR", o);
            } else {
                env::remove_var("TOKSAVE_CACHE_DIR");
            }
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
        let _g = crate::util::env_test_lock();
        let tmp = env::temp_dir().join("toksave-claude-test");
        let old = env::var_os("HOME");
        // Safe: serialized by env_test_lock against other env-mutating tests.
        unsafe {
            env::set_var("HOME", &tmp);
        }
        let cp = claude_paths();
        assert_eq!(cp.dir, tmp.join(".claude"));
        assert_eq!(cp.global_json, tmp.join(".claude.json"));
        unsafe {
            if let Some(o) = old {
                env::set_var("HOME", o);
            } else {
                env::remove_var("HOME");
            }
        }
    }
}
