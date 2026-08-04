use crate::util::errors::{Result, ToksaveError};
use crate::util::paths::read_file;
use std::path::Path;

/// Remove // and /* */ comments, respecting string literals.
fn strip_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                in_string = true;
                out.push(b);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // line comment
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // block comment
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            _ => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Remove trailing commas before `}` or `]`, respecting string literals.
fn strip_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push(b);
            i += 1;
            continue;
        }
        if b == b',' {
            // look ahead past whitespace; if next non-ws is } or ], skip the comma
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                i += 1; // drop comma
                continue;
            }
            out.push(b);
            i += 1;
            continue;
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn read_json_file(path: &Path) -> Result<Option<serde_json::Value>> {
    let Some(raw) = read_file(path) else {
        return Ok(None);
    };
    let cleaned = strip_trailing_commas(&strip_comments(&raw));
    serde_json::from_str(&cleaned).map(Some).map_err(|e| {
        ToksaveError::config(
            &path.to_string_lossy(),
            &format!("Failed to parse JSON: {e}"),
        )
    })
}

pub fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let s = serde_json::to_string_pretty(value).map_err(|e| {
        ToksaveError::config(
            &path.to_string_lossy(),
            &format!("Failed to serialize JSON: {e}"),
        )
    })?;
    crate::util::paths::write_file(path, &format!("{s}\n"))
}

/// Write a JSON config, pruning empty top-level containers first and removing
/// the file entirely when the result is an empty object (or has no object
/// members). Keeps user-owned scalar/array keys like `$schema` or
/// `hooks.version`.
pub fn write_json_pruned(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut value = value.clone();
    if let Some(obj) = value.as_object_mut() {
        let empty: Vec<String> = obj
            .iter()
            .filter(|(_, v)| {
                v.as_object().is_some_and(|o| o.is_empty())
                    || v.as_array().is_some_and(|a| a.is_empty())
            })
            .map(|(k, _)| k.clone())
            .collect();
        for k in empty {
            obj.remove(&k);
        }
    }
    if value.is_object() && value.as_object().is_some_and(|o| o.is_empty()) {
        if path.exists() {
            std::fs::remove_file(path).map_err(crate::util::errors::ToksaveError::from)?;
        }
        return Ok(());
    }
    write_json_file(path, &value)
}

pub fn get_or_create_object<'a>(
    parent: &'a mut serde_json::Value,
    key: &str,
) -> &'a mut serde_json::Value {
    if !parent.is_object() {
        *parent = serde_json::json!({});
    }
    let obj = parent.as_object_mut().expect("object now");
    if !obj
        .get(key)
        .map(serde_json::Value::is_object)
        .unwrap_or(false)
    {
        obj.insert(key.to_string(), serde_json::json!({}));
    }
    obj.get_mut(key).expect("inserted")
}

pub fn add_to_array_if_missing(arr: &mut Vec<serde_json::Value>, entry: serde_json::Value) {
    if !arr.contains(&entry) {
        arr.push(entry);
    }
}

pub fn remove_from_array(arr: &mut Vec<serde_json::Value>, entry: &serde_json::Value) {
    if let Some(idx) = arr.iter().position(|v| v == entry) {
        arr.remove(idx);
    }
}

pub fn has_key(obj: &serde_json::Value, key: &str) -> bool {
    obj.is_object() && obj.get(key).is_some()
}

/// Merge a toksave-managed hook entry into a flat `PreToolUse` array at the
/// top level of a JSON hook config (warp/droid/devin/antigravity shape).
/// Drops existing toksave-managed entries (matched by `matched_by` substring on
/// `hooks[0].command`) and keeps user entries untouched, appending the new one.
pub fn merge_pretool_use(
    cfg: &mut serde_json::Value,
    entry: serde_json::Value,
    managed_marker: &str,
) {
    merge_hook_group(cfg, "PreToolUse", entry, managed_marker);
}

/// Merge a toksave-managed hook entry into a specified hook array (`PreToolUse`,
/// `PermissionRequest`, etc.) in a JSON hook config. Drops existing entries
/// containing `managed_marker` in `hooks[].command` and keeps user entries.
pub fn merge_hook_group(
    parent: &mut serde_json::Value,
    key: &str,
    entry: serde_json::Value,
    managed_marker: &str,
) {
    let arr = parent
        .as_object_mut()
        .expect("config object")
        .entry(key)
        .or_insert_with(|| serde_json::json!([]));
    let Some(items) = arr.as_array_mut() else {
        return;
    };
    items.retain(|e| !is_managed_hook(e, managed_marker));
    items.push(entry);
}

/// True when `entry` is a hook group whose inner `hooks[].command` contains `managed_marker`.
fn is_managed_hook(entry: &serde_json::Value, managed_marker: &str) -> bool {
    entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .is_some_and(|c| c.contains(managed_marker))
            })
        })
        .unwrap_or(false)
}

/// Remove toksave-managed entries from `cfg[\"PreToolUse\"]`, dropping the key
/// when the array becomes empty.
pub fn remove_pretool_use(cfg: &mut serde_json::Value, managed_marker: &str) {
    remove_hook_group(cfg, "PreToolUse", managed_marker);
}

/// Remove toksave-managed entries from `parent[key]`, dropping the key
/// when the array becomes empty.
pub fn remove_hook_group(parent: &mut serde_json::Value, key: &str, managed_marker: &str) {
    let Some(obj) = parent.as_object_mut() else {
        return;
    };
    let Some(arr) = obj.get_mut(key).and_then(|v| v.as_array_mut()) else {
        return;
    };
    arr.retain(|e| !is_managed_hook(e, managed_marker));
    if arr.is_empty() {
        obj.remove(key);
    }
}

/// True when `cfg["PreToolUse"]` has at least one entry whose inner
/// `hooks[].command` contains `managed_marker`.
pub fn has_pretool_with_command_marker(cfg: &serde_json::Value, managed_marker: &str) -> bool {
    cfg.get("PreToolUse")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|e| is_managed_hook(e, managed_marker)))
        .unwrap_or(false)
}

#[cfg(test)]
mod pretool_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_pretool_use_keeps_user_entries_and_drops_ours() {
        let mut cfg = json!({
            "PreToolUse": [
                { "matcher": "Bash", "hooks": [{ "type": "command", "command": "echo user" }] },
                { "matcher": "Bash", "hooks": [{ "type": "command", "command": "/toksave rtk-hook warp" }] }
            ]
        });
        let entry = json!({
            "matcher": "Execute",
            "hooks": [{ "type": "command", "command": "/toksave rtk-hook warp v2" }]
        });
        merge_pretool_use(&mut cfg, entry, "toksave rtk-hook");
        let arr = cfg["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["hooks"][0]["command"].as_str().unwrap(), "echo user");
        assert_eq!(
            arr[1]["hooks"][0]["command"].as_str().unwrap(),
            "/toksave rtk-hook warp v2"
        );
    }

    #[test]
    fn merge_pretool_use_creates_key_when_absent() {
        let mut cfg = json!({});
        merge_pretool_use(&mut cfg, json!({ "hooks": [] }), "toksave");
        assert!(cfg["PreToolUse"].is_array());
    }

    #[test]
    fn remove_pretool_use_keeps_user_entries_and_drops_key_when_empty() {
        let mut cfg = json!({
            "PreToolUse": [
                { "matcher": "X", "hooks": [{ "type": "command", "command": "echo user" }] }
            ]
        });
        remove_pretool_use(&mut cfg, "toksave rtk-hook");
        assert_eq!(cfg["PreToolUse"].as_array().unwrap().len(), 1);

        let mut cfg2 = json!({
            "PreToolUse": [
                { "hooks": [{ "type": "command", "command": "/toksave rtk-hook warp" }] }
            ]
        });
        remove_pretool_use(&mut cfg2, "toksave rtk-hook");
        assert!(cfg2.get("PreToolUse").is_none());
    }

    #[test]
    fn write_json_pruned_removes_empty_container_and_deletes_empty_object() {
        let dir = std::env::temp_dir().join(format!("toksave-prune-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hooks.json");

        write_json_pruned(
            &path,
            &json!({ "mcpServers": {}, "PreToolUse": [], "version": 1 }),
        )
        .unwrap();
        let kept: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(kept, json!({ "version": 1 }));

        write_json_pruned(&path, &json!({ "mcpServers": {} })).unwrap();
        assert!(!path.exists(), "empty object config should delete the file");

        std::fs::remove_dir_all(&dir).ok();
    }
}
