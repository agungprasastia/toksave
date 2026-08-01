mod common;

use common::setup;
use std::fs;
use toksave::util::manifest::{read_manifest, record_wire, remove_wire, was_wired_by_us};

#[test]
fn read_manifest_empty_for_fresh() {
    let _env = setup();
    let m = read_manifest();
    assert!(m.entries.is_empty());
}

#[test]
fn record_wire_and_was_wired_by_us() {
    let _env = setup();
    record_wire("claude", "rtk", Some("0.43.0")).unwrap();
    assert!(was_wired_by_us("claude", "rtk"));
    assert!(!was_wired_by_us("claude", "caveman"));
}

#[test]
fn record_replaces_existing() {
    let _env = setup();
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
    let _env = setup();
    record_wire("opencode", "codegraph", None).unwrap();
    remove_wire("opencode", "codegraph").unwrap();
    assert!(!was_wired_by_us("opencode", "codegraph"));
}

#[test]
fn record_wire_propagates_manifest_write_error() {
    let env = setup();
    let cache = env.cache();
    fs::create_dir_all(&cache).unwrap();
    fs::create_dir_all(cache.join("manifest.json")).unwrap();
    let result = record_wire("claude", "rtk", Some("0.43.0"));
    assert!(result.is_err());
}

#[test]
fn record_wire_times_out_on_fresh_foreign_lock() {
    let env = setup();
    let cache = env.cache();
    fs::create_dir_all(&cache).unwrap();
    fs::create_dir_all(cache.join("manifest.json.lock")).unwrap();
    let start = std::time::Instant::now();
    let result = record_wire("claude", "rtk", None);
    assert!(result.is_err());
    assert!(start.elapsed() >= std::time::Duration::from_secs(5));
    assert!(cache.join("manifest.json.lock").exists());
}
