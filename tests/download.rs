use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use toksave::util::download::{
    DownloadOptions, download_zip, fetch_json, fetch_with_retry, is_safe_archive_path,
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
    if cfg!(windows) {
        assert!(!is_safe_archive_path("C:/abs", &dest));
    }
}

#[tokio::test]
async fn fetch_404_not_retried_and_errors() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/missing");
    let h = std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let resp = tiny_http::Response::from_string("not found").with_status_code(404);
            req.respond(resp).ok();
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

fn make_valid_zip() -> Vec<u8> {
    let cursor = std::io::Cursor::new(vec![0u8; 0]);
    let mut zw = zip::ZipWriter::new(cursor);
    let opts = zip::write::SimpleFileOptions::default();
    zw.start_file("ok.txt", opts).unwrap();
    zw.write_all(b"hello").unwrap();
    zw.finish().unwrap().into_inner()
}

#[tokio::test]
async fn download_zip_good_checksum_succeeds() {
    let bytes = make_valid_zip();
    let hash = format!("{:x}", Sha256::digest(&bytes));

    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/good.zip");
    let h = std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            req.respond(tiny_http::Response::from_data(bytes)).ok();
        }
    });
    let dest = tmp_dir().join("checksum_good");
    let opts = DownloadOptions {
        checksum: Some(hash),
        ..Default::default()
    };
    let res = download_zip(&url, &dest, &opts).await;
    assert!(res.is_ok(), "expected Ok, got {:?}", res);
    h.join().ok();
    let _ = std::fs::remove_dir_all(&dest);
}

#[tokio::test]
async fn download_zip_bad_checksum_errors() {
    let bytes = make_valid_zip();

    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/bad.zip");
    let h = std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            req.respond(tiny_http::Response::from_data(bytes)).ok();
        }
    });
    let dest = tmp_dir().join("checksum_bad");
    let opts = DownloadOptions {
        checksum: Some("deadbeef".to_string()),
        ..Default::default()
    };
    let res = download_zip(&url, &dest, &opts).await;
    assert!(res.is_err());
    h.join().ok();
    let _ = std::fs::remove_dir_all(&dest);
}
