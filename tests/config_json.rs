use std::fs;
use std::path::PathBuf;
use toksave::util::json::{
    add_to_array_if_missing, get_or_create_object, read_json_file, remove_from_array,
    write_json_file,
};
use toksave::util::paths::write_file;

fn tmp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("toksave-config-test-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir
}

#[test]
fn read_json_file_missing_returns_none() {
    let dir = tmp_dir().join("missing");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("nope.json");
    assert!(read_json_file(&p).unwrap().is_none());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn write_read_round_trip() {
    let dir = tmp_dir().join("round_trip");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("test.json");
    write_json_file(&p, &serde_json::json!({ "foo": "bar", "num": 42 })).unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["foo"], "bar");
    assert_eq!(read["num"], 42);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn strips_jsonc_comments() {
    let dir = tmp_dir().join("comments");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("commented.json");
    write_file(&p, "{\n  // comment\n  /* block */\n  \"key\": \"val\"\n}").unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["key"], "val");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn strips_trailing_commas() {
    let dir = tmp_dir().join("trailing");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("trailing.json");
    write_file(&p, "{ \"a\": 1, \"b\": [2, 3,], }").unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["a"], 1);
    assert_eq!(read["b"][1], 3);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn unparseable_config_is_error_not_fallback() {
    let dir = tmp_dir().join("unparseable");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("bad.json");
    write_file(&p, "{ this is not valid json").unwrap();
    assert!(read_json_file(&p).is_err());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn string_with_comment_like_content_survives() {
    let dir = tmp_dir().join("strings");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("strings.json");
    write_file(
        &p,
        "{ \"url\": \"http://example.com/x\", \"path\": \"a/*b\" }",
    )
    .unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["url"], "http://example.com/x");
    assert_eq!(read["path"], "a/*b");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn get_or_create_object_creates_nested() {
    let mut obj = serde_json::json!({});
    let sub = get_or_create_object(&mut obj, "mcpServers");
    sub["test"] = serde_json::json!(true);
    assert_eq!(obj["mcpServers"]["test"], true);
}

#[test]
fn add_to_array_avoids_duplicates() {
    let mut arr = serde_json::json!(["a", "b"]);
    let items = arr.as_array_mut().unwrap();
    add_to_array_if_missing(items, serde_json::json!("b"));
    add_to_array_if_missing(items, serde_json::json!("c"));
    assert_eq!(items.len(), 3);
}

#[test]
fn remove_from_array_removes_entry() {
    let mut arr = serde_json::json!(["a", "b", "c"]);
    let items = arr.as_array_mut().unwrap();
    remove_from_array(items, &serde_json::json!("b"));
    assert_eq!(items.len(), 2);
}
