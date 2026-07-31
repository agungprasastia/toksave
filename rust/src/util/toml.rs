use crate::util::errors::{Result, ToksaveError};
use crate::util::paths::{read_file, write_file};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Value};

pub fn read_toml_file(path: &Path) -> Result<DocumentMut> {
    let raw = read_file(path).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(DocumentMut::new());
    }
    raw.parse::<DocumentMut>().map_err(|e| {
        ToksaveError::config(
            &path.to_string_lossy(),
            &format!("Failed to parse TOML: {e}"),
        )
    })
}

pub fn write_toml_file(path: &Path, doc: &DocumentMut) -> Result<()> {
    write_file(path, &doc.to_string())
}

pub fn has_table(doc: &DocumentMut, key_path: &str) -> bool {
    let parts: Vec<&str> = key_path.split('.').collect();
    let mut current: &Item = doc.as_item();
    for part in parts {
        match current.get(part) {
            Some(item) => current = item,
            None => return false,
        }
    }
    current.is_table() || current.is_inline_table()
}

pub fn upsert_table(doc: &mut DocumentMut, key_path: &str, command: &str) {
    let parts: Vec<&str> = key_path.split('.').collect();
    let mut current = doc.as_table_mut();
    for p in parts.iter().take(parts.len() - 1) {
        if !current.contains_key(p) {
            current.insert(p, Item::Table(toml_edit::Table::new()));
        }
        current = current.get_mut(p).unwrap().as_table_mut().unwrap();
    }
    let last = parts[parts.len() - 1];
    if !current.contains_key(last) {
        current.insert(last, Item::Table(toml_edit::Table::new()));
    }
    let target = current.get_mut(last).unwrap().as_table_mut().unwrap();
    target.insert("command", toml_edit::value(command));
}

pub fn set_table_array(doc: &mut DocumentMut, key_path: &str, key: &str, args: &[String]) {
    let parts: Vec<&str> = key_path.split('.').collect();
    let mut item: &mut Item = doc.as_item_mut();
    for p in parts {
        match item.get_mut(p) {
            Some(next) => item = next,
            None => return,
        }
    }
    if let Some(tbl) = item.as_table_mut() {
        let mut arr = toml_edit::Array::new();
        for arg in args {
            arr.push(arg.as_str());
        }
        tbl.insert(key, Item::Value(Value::Array(arr)));
    }
}

pub fn remove_table(doc: &mut DocumentMut, key_path: &str) {
    let parts: Vec<&str> = key_path.split('.').collect();
    if parts.len() == 2 {
        if let Some(parent) = doc.get_mut(parts[0]) {
            if let Some(table) = parent.as_table_mut() {
                table.remove(parts[1]);
            }
        }
    }
}
