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

/// Like `write_toml_file`, but deletes the file when the document is empty so
/// uninstalling every managed table leaves no empty `config.toml` behind.
pub fn write_toml_pruned(path: &Path, doc: &DocumentMut) -> Result<()> {
    if doc.as_table().is_empty() {
        if path.exists() {
            std::fs::remove_file(path).map_err(ToksaveError::from)?;
        }
        return Ok(());
    }
    write_toml_file(path, doc)
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
    if parts.len() == 2
        && let Some(parent) = doc.get_mut(parts[0])
        && let Some(table) = parent.as_table_mut()
    {
        table.remove(parts[1]);
    }
}

/// Remove tables and their parents once they hold no remaining keys, so
/// uninstalling every toksave-managed `mcp_servers` leaves no `[mcp_servers]`
/// ghosts. Repeats until stable so nested empties collapse.
pub fn prune_empty_tables(doc: &mut DocumentMut) {
    loop {
        let mut removed = false;
        let root = doc.as_table_mut();
        let keys: Vec<String> = root.iter().map(|(k, _)| k.to_string()).collect();
        for k in keys {
            let empty = root
                .get(&k)
                .and_then(|i| i.as_table())
                .map(|t| t.is_empty())
                .unwrap_or(false);
            if empty {
                root.remove(&k);
                removed = true;
            }
        }
        if !removed {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prune_empty_tables_drops_ghost_mcp_servers() {
        let mut doc = DocumentMut::new();
        upsert_table(&mut doc, "mcp_servers.codegraph", "/toksave");
        upsert_table(&mut doc, "mcp_servers.context-mode", "/toksave");
        assert!(has_table(&doc, "mcp_servers.codegraph"));

        remove_table(&mut doc, "mcp_servers.codegraph");
        remove_table(&mut doc, "mcp_servers.context-mode");
        prune_empty_tables(&mut doc);
        assert!(!has_table(&doc, "mcp_servers"));
    }

    #[test]
    fn prune_keeps_nonempty_user_tables() {
        let mut doc: DocumentMut = "[user]\nkey = 1\n[mcp_servers]\n[mcp_servers.x]\ncommand = 1\n"
            .parse()
            .unwrap();
        remove_table(&mut doc, "mcp_servers.x");
        prune_empty_tables(&mut doc);
        assert!(!has_table(&doc, "mcp_servers"));
        assert!(has_table(&doc, "user"));
    }

    #[test]
    fn write_toml_pruned_deletes_when_empty() {
        let dir = std::env::temp_dir().join(format!("toksave-toml-prune-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        let mut doc = DocumentMut::new();
        upsert_table(&mut doc, "mcp_servers.x", "/toksave");
        write_toml_pruned(&path, &doc).unwrap();
        assert!(path.exists());

        remove_table(&mut doc, "mcp_servers.x");
        prune_empty_tables(&mut doc);
        write_toml_pruned(&path, &doc).unwrap();
        assert!(!path.exists(), "empty toml should delete the file");

        std::fs::remove_dir_all(&dir).ok();
    }
}
