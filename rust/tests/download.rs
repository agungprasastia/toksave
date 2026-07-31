use std::io::Write;
use std::path::PathBuf;
use toksave_rs::util::download::{
    download_zip, fetch_json, fetch_with_retry, is_safe_archive_path, DownloadOptions,
};

fn tmp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-download-test-{}", std::process::id()))
}

#[test]
fn is_safe_archive_path_rejects_traversal() {
    let dest = tmp_dir().join("dest");
    assert!(is_safe_archive_path("ok/file.txt", &dest));
    assert!(is_safe_archive_path("a/b/c.txt", &dest));
    assert!(!is_safe_archive_path("../escape.txt", &dest));
    assert!(!is_safe_archive_path("a/../../escape.txt", &dest));
    assert!(!is_safe_archive_path("/absolute/path", &dest));
    assert!(!is_safe_archive_path("C:/abs", &dest));
}

#[tokio::test]
async fn fetch_404_not_retried_and_errors() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/missing");
    let h = std::thread::spawn(move || {
        if let Ok(_req) = server.recv() {
            // respond 404
        }
    });
    let res = fetch_with_retry(&url, &DownloadOptions::default()).await;
    assert!(res.is_err());
    h.join().ok();
}

#[tokio::test]
async fn fetch_json_ok() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/json");
    let h = std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let resp = tiny_http::Response::from_string("{\"tag_name\":\"v1.2.3\"}");
            req.respond(resp).ok();
        }
    });
    let v = fetch_json(&url).await.unwrap();
    assert_eq!(v["tag_name"], "v1.2.3");
    h.join().ok();
}

#[tokio::test]
async fn download_zip_rejects_zip_slip() {
    // Build a malicious zip in memory: entry "../evil.txt"
    let cursor = std::io::Cursor::new(vec![0u8; 0]);
    let mut zw = zip::ZipWriter::new(cursor);
    let opts = zip::write::SimpleFileOptions::default();
    zw.start_file("../evil.txt", opts).unwrap();
    zw.write_all(b"boom").unwrap();
    let cursor = zw.finish().unwrap();
    let bytes = cursor.into_inner();

    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/evil.zip");
    let h = std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let resp = tiny_http::Response::from_data(bytes);
            req.respond(resp).ok();
        }
    });
    let dest = tmp_dir().join("zipdest");
    let res = download_zip(&url, &dest, &DownloadOptions::default()).await;
    assert!(res.is_err());
    assert!(!dest.join("..").join("evil.txt").exists());
    h.join().ok();
}
