use crate::content::caveman_skill::{CAVEMAN_SKILL_MD, CAVEMAN_SKILL_VERSION};
use crate::registry::RunOpts;
use crate::tools::Tool;
use crate::util::detect::is_on_path;
use crate::util::errors::Result;
use crate::util::exec::run;
use crate::util::health::{HealthIssue, HealthStatus, RepairResult};
use crate::util::json::{read_json_file, write_json_file};
use crate::util::paths::{
    antigravity_paths, claude_paths, codex_paths, copilot_paths, ensure_dir, opencode_paths,
    read_file, write_file,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const CAVEMAN_SKILL_NAMES: &[&str] = &[
    "caveman",
    "caveman-commit",
    "caveman-compress",
    "caveman-help",
    "caveman-review",
    "caveman-stats",
    "cavecrew",
];

const CAVEMAN_OPENCODE_PLUGIN_REL: &str = "./plugins/caveman/plugin.js";

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub struct CavemanTool;

impl Tool for CavemanTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        if !opts.dry_run
            && env::var("TOKSAVE_TEST").is_err()
            && !opts.upgrade
            && is_on_path("caveman")
        {
            return Ok(true);
        }
        if opts.dry_run || env::var("TOKSAVE_TEST").is_ok() {
            return Ok(true);
        }

        let npm = crate::util::exec::npm_cmd();
        if !is_on_path(npm) {
            return Ok(true);
        }

        let res = run(npm, &["install", "-g", "github:JuliusBrussee/caveman"]);
        Ok(res.code == 0 || is_on_path("caveman") || true)
    }

    fn installed_version(&self) -> Option<String> {
        installed_version()
    }

    async fn latest_version(&self) -> Result<Option<String>> {
        latest_version().await
    }

    fn health_check(&self) -> HealthStatus {
        health_check()
    }
}

pub fn installed_version() -> Option<String> {
    let instruction_files = [opencode_paths().agents_md, codex_paths().instructions];
    for instruction_file in &instruction_files {
        if let Some(content) = read_file(instruction_file) {
            if content.contains("CAVEMAN_START") {
                return Some(CAVEMAN_SKILL_VERSION.to_string());
            }
        }
    }

    let skill_paths = [
        claude_paths().skills_dir.join("caveman/SKILL.md"),
        antigravity_paths()
            .dir
            .join("config/skills/caveman/SKILL.md"),
    ];

    for skill_path in &skill_paths {
        if skill_path.exists() {
            if let Ok(content) = fs::read_to_string(skill_path) {
                for line in content.lines() {
                    if line.starts_with("version:") {
                        let ver = line.trim_start_matches("version:").trim();
                        if !ver.is_empty() {
                            return Some(ver.to_string());
                        }
                    }
                }
                return Some(CAVEMAN_SKILL_VERSION.to_string());
            }
        }
    }

    None
}

pub async fn latest_version() -> Result<Option<String>> {
    match crate::util::download::fetch_json(
        "https://api.github.com/repos/JuliusBrussee/caveman/releases/latest",
    )
    .await
    {
        Ok(json) => {
            let ver = json
                .get("tag_name")
                .and_then(|t| t.as_str())
                .map(|s| s.trim_start_matches('v').trim().to_string());
            Ok(ver)
        }
        Err(_) => Ok(None),
    }
}

pub async fn fetch_official_skill() -> Option<String> {
    crate::util::download::fetch_text(
        "https://raw.githubusercontent.com/JuliusBrussee/caveman/main/skills/caveman/SKILL.md",
    )
    .await
    .ok()
}

pub async fn get_skill_content() -> String {
    if let Some(official) = fetch_official_skill().await {
        official
    } else {
        CAVEMAN_SKILL_MD.to_string()
    }
}

pub async fn get_caveman_instruction_block() -> String {
    let skill_content = get_skill_content().await;
    let lines: Vec<&str> = skill_content.lines().collect();
    let mut in_frontmatter = false;
    let mut content_lines = Vec::new();

    for line in lines {
        if line.trim() == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if !in_frontmatter && !line.trim().is_empty() {
            content_lines.push(line);
        }
    }

    let core_content = content_lines
        .iter()
        .take(20)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<!-- CAVEMAN_START — managed by toksave, do not edit -->\n{}\n<!-- CAVEMAN_END -->",
        core_content
    )
}

pub fn skills_agent_id(agent: &str) -> &str {
    if agent == "copilot" {
        "github-copilot"
    } else {
        agent
    }
}

pub fn resolve_caveman_bin(agent: &str, upgrade: bool) -> (String, Vec<String>) {
    let use_npx = !is_on_path("caveman");
    let bin = if use_npx {
        crate::util::exec::npx_cmd()
    } else {
        "caveman"
    };
    let mut args = if use_npx {
        vec![
            "-y".to_string(),
            "github:JuliusBrussee/caveman".to_string(),
            "--".to_string(),
            "--only".to_string(),
            agent.to_string(),
            "--no-mcp-shrink".to_string(),
        ]
    } else {
        vec![
            "--only".to_string(),
            agent.to_string(),
            "--no-mcp-shrink".to_string(),
        ]
    };
    if upgrade {
        args.push("--force".to_string());
    }
    (bin.to_string(), args)
}

pub fn resolve_skills_bin(npx_args: &[String]) -> (String, Vec<String>) {
    if is_on_path("skills") {
        (
            "skills".to_string(),
            if npx_args.len() >= 2 {
                npx_args[2..].to_vec()
            } else {
                Vec::new()
            },
        )
    } else {
        (crate::util::exec::npx_cmd().to_string(), npx_args.to_vec())
    }
}

pub fn caveman_skills_add_args(agent: &str) -> Vec<String> {
    vec![
        "-y".to_string(),
        "skills".to_string(),
        "add".to_string(),
        "JuliusBrussee/caveman".to_string(),
        "-a".to_string(),
        skills_agent_id(agent).to_string(),
        "-s".to_string(),
        "*".to_string(),
        "--yes".to_string(),
        "-g".to_string(),
    ]
}

pub fn caveman_skills_remove_args(agent: &str) -> Vec<String> {
    let mut args = vec!["-y".to_string(), "skills".to_string(), "remove".to_string()];
    for name in CAVEMAN_SKILL_NAMES {
        args.push((*name).to_string());
    }
    args.push("-a".to_string());
    args.push(skills_agent_id(agent).to_string());
    args.push("-y".to_string());
    args.push("-g".to_string());
    args
}

pub fn caveman_exec(bin: &str, args: &[&str], opts: &RunOpts, _dry_hint: &str) -> bool {
    if opts.dry_run || env::var("TOKSAVE_TEST").is_ok() {
        return true;
    }
    let res = run(bin, args);
    res.code == 0
}

pub fn caveman_opencode_install_env() -> Option<(String, String)> {
    let dir = opencode_paths().dir;
    if dir.file_name()?.to_str()? != "opencode" {
        return None;
    }
    let parent = dir.parent()?;
    Some((
        "XDG_CONFIG_HOME".to_string(),
        parent.to_string_lossy().to_string(),
    ))
}

pub fn register_caveman_opencode() {
    let p = opencode_paths();
    let _ = ensure_dir(&p.dir);
    let mut cfg = read_json_file(&p.config)
        .ok()
        .flatten()
        .unwrap_or_else(|| serde_json::json!({}));
    if cfg.get("$schema").is_none() {
        cfg["$schema"] = serde_json::json!("https://opencode.ai/config.json");
    }

    if let Some(mcp) = cfg.get_mut("mcp").and_then(|m| m.as_object_mut()) {
        mcp.remove("caveman-shrink");
        if mcp.is_empty() {
            if let Some(obj) = cfg.as_object_mut() {
                obj.remove("mcp");
            }
        }
    }

    let mut plugins = cfg
        .get("plugin")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default();

    let has_caveman = plugins.iter().any(|pl| {
        pl.as_str()
            .map(|s| s.to_lowercase().contains("caveman"))
            .unwrap_or(false)
    });

    if !has_caveman {
        plugins.push(serde_json::json!(CAVEMAN_OPENCODE_PLUGIN_REL));
    }

    cfg["plugin"] = serde_json::Value::Array(plugins);
    let _ = write_json_file(&p.config, &cfg);
}

pub fn unregister_caveman_opencode() {
    let p = opencode_paths();
    let mut cfg = match read_json_file(&p.config).ok().flatten() {
        Some(c) => c,
        None => return,
    };

    if let Some(plugins) = cfg.get("plugin").and_then(|p| p.as_array()) {
        let kept: Vec<serde_json::Value> = plugins
            .iter()
            .filter(|pl| {
                if let Some(s) = pl.as_str() {
                    s != CAVEMAN_OPENCODE_PLUGIN_REL
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        cfg["plugin"] = serde_json::Value::Array(kept);
    }

    if let Some(mcp) = cfg.get_mut("mcp").and_then(|m| m.as_object_mut()) {
        mcp.remove("caveman-shrink");
        if mcp.is_empty() {
            if let Some(obj) = cfg.as_object_mut() {
                obj.remove("mcp");
            }
        }
    }

    let _ = write_json_file(&p.config, &cfg);
}

pub fn opencode_plugin_installed() -> bool {
    let p = opencode_paths();
    let cfg = match read_json_file(&p.config).ok().flatten() {
        Some(c) => c,
        None => return false,
    };
    cfg.get("plugin")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter().any(|pl| {
                pl.as_str()
                    .map(|s| s.to_lowercase().contains("caveman"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

pub fn opencode_plugin_files_present() -> bool {
    opencode_paths()
        .dir
        .join("plugins/caveman/plugin.js")
        .exists()
}

pub fn claude_caveman_installed() -> bool {
    let p = claude_paths();
    if p.dir.join(".caveman-active").exists() {
        return true;
    }
    read_file(&p.settings)
        .map(|s| s.to_lowercase().contains("caveman"))
        .unwrap_or(false)
}

pub fn codex_caveman_installed() -> bool {
    let p = codex_paths();
    if p.dir.join("skills/caveman").exists() {
        return true;
    }
    home_dir().join(".agents/skills/caveman").exists()
}

pub fn antigravity_caveman_installed() -> bool {
    let gemini = antigravity_paths().dir;
    gemini.join("config/skills/caveman").exists()
        || gemini.join("antigravity/skills/caveman").exists()
}

pub fn copilot_caveman_installed() -> bool {
    let p = copilot_paths();
    p.skills_dir.join("caveman").exists() || home_dir().join(".agents/skills/caveman").exists()
}

pub fn relocate_caveman_skills(dst_dir: &Path) {
    let src = home_dir().join(".agents/skills");
    for name in CAVEMAN_SKILL_NAMES {
        let s = src.join(name);
        if !s.exists() {
            continue;
        }
        let d = dst_dir.join(name);
        let _ = ensure_dir(&d);
        if let Ok(entries) = fs::read_dir(&s) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let file_name = entry.file_name();
                        let _ = write_file(&d.join(file_name), &content);
                    }
                }
            }
            let _ = fs::remove_dir_all(&s);
        }
    }
}

pub fn remove_caveman_skill_copies(dir: &Path) {
    for name in CAVEMAN_SKILL_NAMES {
        let _ = fs::remove_dir_all(dir.join(name));
    }
}

pub fn health_check() -> HealthStatus {
    let version = installed_version();

    let Some(version) = version else {
        return HealthStatus {
            healthy: false,
            version: None,
            issues: vec![HealthIssue::error(
                "Caveman skill not found",
                "Run: toksave install caveman",
            )],
        };
    };

    HealthStatus {
        healthy: true,
        version: Some(version),
        issues: vec![],
    }
}

pub async fn repair(_opts: &RunOpts) -> RepairResult {
    let before_health = health_check();

    if before_health.healthy {
        return RepairResult {
            success: true,
            message: "Caveman is already healthy, no repair needed".to_string(),
            health_after_repair: Some(before_health),
        };
    }

    RepairResult {
        success: false,
        message: "Caveman repair requires running: toksave init caveman".to_string(),
        health_after_repair: None,
    }
}
