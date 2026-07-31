use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use toksave_rs::util::manifest::{read_manifest, record_wire, remove_wire, was_wired_by_us};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn tmp_cache() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-manifest-test-{}", std::process::id()))
}

fn isolated_cache(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "toksave-manifest-test-{}-{label}",
        std::process::id()
    ))
}

#[test]
fn read_manifest_empty_for_fresh() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old = std::env::var_os("TOKSAVE_CACHE_DIR");
    let cache = isolated_cache("fresh");
    fs::remove_dir_all(&cache).ok();
    std::env::set_var("TOKSAVE_CACHE_DIR", &cache);
    let m = read_manifest();
    assert!(m.entries.is_empty());
    restore(old);
    fs::remove_dir_all(&cache).ok();
}

#[test]
fn record_wire_and_was_wired_by_us() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("TOKSAVE_CACHE_DIR", isolated_cache("record"));
    record_wire("claude", "rtk", Some("0.43.0")).unwrap();
    assert!(was_wired_by_us("claude", "rtk"));
    assert!(!was_wired_by_us("claude", "caveman"));
    restore(std::env::var_os("TOKSAVE_CACHE_DIR"));
}

#[test]
fn record_replaces_existing() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("TOKSAVE_CACHE_DIR", isolated_cache("replace"));
    record_wire("claude", "rtk", Some("0.42.0")).unwrap();
    record_wire("claude", "rtk", Some("0.43.0")).unwrap();
    let m = read_manifest();
    let matches: Vec<_> = m
        .entries
        .iter()
        .filter(|e| e.agent == "claude" && e.tool == "rtk")
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].version.as_deref(), Some("0.43.0"));
}

#[test]
fn remove_wire_clears() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("TOKSAVE_CACHE_DIR", isolated_cache("remove"));
    record_wire("opencode", "codegraph", None).unwrap();
    remove_wire("opencode", "codegraph").unwrap();
    assert!(!was_wired_by_us("opencode", "codegraph"));
}

#[test]
fn record_wire_propagates_manifest_write_error() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old = std::env::var_os("TOKSAVE_CACHE_DIR");
    let cache = isolated_cache("write-error");
    fs::remove_dir_all(&cache).ok();
    fs::create_dir_all(cache.join("manifest.json")).unwrap();
    std::env::set_var("TOKSAVE_CACHE_DIR", &cache);
    let result = record_wire("claude", "rtk", Some("0.43.0"));
    assert!(result.is_err());
    restore(old);
    fs::remove_dir_all(&cache).ok();
}

#[test]
fn record_wire_times_out_on_fresh_foreign_lock() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old = std::env::var_os("TOKSAVE_CACHE_DIR");
    let cache = isolated_cache("timeout");
    fs::remove_dir_all(&cache).ok();
    fs::create_dir_all(cache.join("manifest.json.lock")).unwrap();
    std::env::set_var("TOKSAVE_CACHE_DIR", &cache);
    let start = std::time::Instant::now();
    let result = record_wire("claude", "rtk", None);
    assert!(result.is_err());
    assert!(start.elapsed() >= std::time::Duration::from_secs(5));
    assert!(cache.join("manifest.json.lock").exists());
    restore(old);
    fs::remove_dir_all(&cache).ok();
}

fn restore(v: Option<std::ffi::OsString>) {
    match v {
        Some(v) => std::env::set_var("TOKSAVE_CACHE_DIR", v),
        None => std::env::remove_var("TOKSAVE_CACHE_DIR"),
    }
}

#[test]
fn cleanup() {
    let _guard = ENV_LOCK.lock().unwrap();
    fs::remove_dir_all(tmp_cache()).ok();
    for label in [
        "fresh",
        "record",
        "replace",
        "remove",
        "write-error",
        "timeout",
    ] {
        fs::remove_dir_all(isolated_cache(label)).ok();
    }
    std::env::remove_var("TOKSAVE_CACHE_DIR");
}
