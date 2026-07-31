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
