use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "go.mod",
    "Cargo.toml",
    "pyproject.toml",
    "pom.xml",
    "build.gradle",
    "tsconfig.json",
    "requirements.txt",
];

pub fn looks_like_project(dir: &Path) -> bool {
    PROJECT_MARKERS.iter().any(|m| dir.join(m).exists())
}

pub fn find_project_dir(start: &Path) -> PathBuf {
    let mut cur = start.to_path_buf();
    for _ in 0..20 {
        if looks_like_project(&cur) {
            return cur;
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => break,
        }
    }
    start.to_path_buf()
}

fn resolve_project_dir(input: &str) -> Option<PathBuf> {
    if !input.is_empty()
        && let Ok(req) = serde_json::from_str::<serde_json::Value>(input)
    {
        if let Some(p) = req
            .get("workspacePaths")
            .and_then(|w| w.as_array())
            .and_then(|a| a.first())
            .and_then(|p| p.as_str())
        {
            return Some(find_project_dir(Path::new(p)));
        }
        if let Some(cwd) = req.get("cwd").and_then(|c| c.as_str()) {
            return Some(find_project_dir(Path::new(cwd)));
        }
    }
    std::env::current_dir().ok().map(|d| find_project_dir(&d))
}

fn resolve_codegraph_bin() -> Option<PathBuf> {
    let exe = format!("codegraph{}", std::env::consts::EXE_SUFFIX);
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|d| d.join(&exe))
        .find(|p| p.is_file())
}

/// Antigravity/Copilot codegraph index hook: silently `codegraph init|sync`
/// in the detected project dir. Never fails the agent invocation.
pub fn run_codegraph_index_hook(input: &str) -> i32 {
    let dir = match resolve_project_dir(input) {
        Some(d) => d,
        None => return 0,
    };

    let indexed = dir.join(".codegraph").exists();
    if !looks_like_project(&dir) && !indexed {
        return 0;
    }

    let bin = match resolve_codegraph_bin() {
        Some(b) => b,
        None => return 0,
    };

    let _ = Command::new(bin)
        .arg(if indexed { "sync" } else { "init" })
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    0
}
