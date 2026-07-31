use std::fs;
use std::path::PathBuf;
use toksave_rs::util::manifest::{read_manifest, record_wire, remove_wire, was_wired_by_us};

fn tmp_cache() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-manifest-test-{}", std::process::id()))
}

#[test]
fn read_manifest_empty_for_fresh() {
    let old = std::env::var_os("TOKSAVE_CACHE_DIR");
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    let m = read_manifest();
    assert!(m.entries.is_empty());
    restore(old);
}

#[test]
fn record_wire_and_was_wired_by_us() {
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    record_wire("claude", "rtk", Some("0.43.0")).unwrap();
    assert!(was_wired_by_us("claude", "rtk"));
    assert!(!was_wired_by_us("claude", "caveman"));
    restore(std::env::var_os("TOKSAVE_CACHE_DIR"));
}

#[test]
fn record_replaces_existing() {
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
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
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    record_wire("opencode", "codegraph", None).unwrap();
    remove_wire("opencode", "codegraph").unwrap();
    assert!(!was_wired_by_us("opencode", "codegraph"));
}

fn restore(v: Option<std::ffi::OsString>) {
    match v {
        Some(v) => std::env::set_var("TOKSAVE_CACHE_DIR", v),
        None => std::env::remove_var("TOKSAVE_CACHE_DIR"),
    }
}

#[test]
fn cleanup() {
    fs::remove_dir_all(tmp_cache()).ok();
    std::env::remove_var("TOKSAVE_CACHE_DIR");
}
