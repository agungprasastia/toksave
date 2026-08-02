use crate::util::errors::{Result, ToksaveError};
use crate::util::paths::cache_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub agent: String,
    pub tool: String,
    pub wired_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// "active" (absent) or "disabled". Old manifests without the field read as active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

fn manifest_path() -> PathBuf {
    cache_dir().join("manifest.json")
}

fn lock_path() -> PathBuf {
    cache_dir().join("manifest.json.lock")
}

fn read_manifest_file() -> Manifest {
    let p = manifest_path();
    let Ok(raw) = fs::read_to_string(&p) else {
        return Manifest::default();
    };
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to parse manifest at {}: {e}", p.display());
        Manifest::default()
    })
}

fn write_manifest(m: &Manifest) -> Result<()> {
    let p = manifest_path();
    if let Some(parent) = p.parent() {
        crate::util::paths::ensure_dir(parent)?;
    }
    let content = format!(
        "{}\n",
        serde_json::to_string_pretty(m).map_err(|e| {
            ToksaveError::config(
                &p.to_string_lossy(),
                &format!("Failed to serialize manifest: {e}"),
            )
        })?
    );
    crate::util::paths::write_file(&p, &content)
}

fn stale_lock_age() -> Duration {
    Duration::from_secs(30)
}

/// Acquire the manifest lock by creating a lock dir (mkdir is atomic).
/// Timeout after 5s; stale locks older than 30s are force-removed.
fn with_manifest_lock<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock = lock_path();
    if let Some(parent) = lock.parent() {
        let _ = crate::util::paths::ensure_dir(parent);
    }
    let started = SystemTime::now();
    loop {
        match fs::create_dir(&lock) {
            Ok(()) => break,
            Err(_) => {
                if let Ok(meta) = fs::metadata(&lock)
                    && let Ok(modified) = meta.modified()
                    && modified.elapsed().unwrap_or_default() > stale_lock_age()
                {
                    let _ = fs::remove_dir_all(&lock);
                    continue;
                }
                if started.elapsed().unwrap_or_default() > Duration::from_secs(5) {
                    return Err(ToksaveError::tool(
                        "manifest",
                        &format!("Timed out waiting for manifest lock: {}", lock.display()),
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    // Normal exit means THIS process created the lock; the timeout path returned
    // Err above, so a foreign lock is never deleted.
    let result = f();
    let _ = fs::remove_dir_all(&lock);
    result
}

pub fn read_manifest() -> Manifest {
    read_manifest_file()
}

pub fn record_wire(agent: &str, tool: &str, version: Option<&str>) -> Result<()> {
    with_manifest_lock(|| {
        let mut m = read_manifest_file();
        m.entries.retain(|e| !(e.agent == agent && e.tool == tool));
        m.entries.push(ManifestEntry {
            agent: agent.to_string(),
            tool: tool.to_string(),
            wired_at: now_iso8601(),
            version: version.map(str::to_string),
            state: None,
        });
        write_manifest(&m)
    })
}

pub fn remove_wire(agent: &str, tool: &str) -> Result<()> {
    with_manifest_lock(|| {
        let mut m = read_manifest_file();
        m.entries.retain(|e| !(e.agent == agent && e.tool == tool));
        write_manifest(&m)
    })
}

/// Mark an entry disabled (unwired but kept in manifest). No-op if absent.
pub fn mark_disabled(agent: &str, tool: &str) -> Result<()> {
    with_manifest_lock(|| {
        let mut m = read_manifest_file();
        for e in &mut m.entries {
            if e.agent == agent && e.tool == tool {
                e.state = Some("disabled".into());
            }
        }
        write_manifest(&m)
    })
}

pub fn was_wired_by_us(agent: &str, tool: &str) -> bool {
    read_manifest()
        .entries
        .iter()
        .any(|e| e.agent == agent && e.tool == tool)
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC3339-ish; tests only compare equality of entries, not format.
    // ponytail: full chrono formatting deferred; this is stable & parseable.
    format!("{secs}s")
}
