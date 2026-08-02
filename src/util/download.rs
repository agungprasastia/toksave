use crate::util::errors::{Result, ToksaveError};
use crate::util::version::toksave_version;
use futures_util::StreamExt;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

/// Optional live progress bar for large binary downloads. Set per-install by
/// the caller (init/update); `fetch_with_retry_inner` updates it as chunks
/// stream in. A global sink avoids threading a callback through the whole
/// install chain — only RTK downloads binaries and installs run sequentially.
static PROGRESS_BAR: Mutex<Option<indicatif::ProgressBar>> = Mutex::new(None);

pub fn set_download_progress_bar(bar: indicatif::ProgressBar) {
    if let Ok(mut g) = PROGRESS_BAR.lock() {
        *g = Some(bar);
    }
}

pub fn clear_download_progress_bar() {
    if let Ok(mut g) = PROGRESS_BAR.lock() {
        *g = None;
    }
}

fn report_progress(done: u64, total: Option<u64>) {
    if let Ok(g) = PROGRESS_BAR.lock()
        && let Some(bar) = g.as_ref()
    {
        if let Some(total) = total {
            bar.set_length(total);
        }
        bar.set_position(done);
    }
}

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub retries: usize,
    pub timeout: Duration,
    pub checksum: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            retries: 3,
            timeout: Duration::from_secs(120),
            checksum: None,
        }
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(format!("toksave-rs/{}", toksave_version()))
        .build()
        .expect("reqwest client")
}

async fn fetch_with_retry_inner(url: &str, opts: &DownloadOptions) -> Result<Vec<u8>> {
    let client = client();
    let mut last_err: Option<ToksaveError> = None;
    for attempt in 0..=opts.retries {
        let resp = client.get(url).timeout(opts.timeout).send().await;
        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    let total = r.content_length();
                    let mut bytes = Vec::with_capacity(total.unwrap_or(0) as usize);
                    let mut stream = r.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.map_err(|e| {
                            ToksaveError::network("download", "failed to read body", url, None)
                                .with_source(e)
                        })?;
                        bytes.extend_from_slice(&chunk);
                        report_progress(bytes.len() as u64, total);
                    }
                    return Ok(bytes);
                }
                if r.status().as_u16() == 404 {
                    return Err(ToksaveError::download(
                        "download",
                        "HTTP 404 Not Found",
                        url,
                        Some(
                            "URL not found. Check if the resource exists or try a different version.",
                        ),
                    ));
                }
                last_err = Some(ToksaveError::download(
                    "download",
                    &format!(
                        "HTTP {} {}",
                        r.status().as_u16(),
                        r.status().canonical_reason().unwrap_or("")
                    ),
                    url,
                    None,
                ));
            }
            Err(e) => {
                last_err = Some(ToksaveError::network("download", &e.to_string(), url, None));
            }
        }
        if attempt < opts.retries {
            let backoff = Duration::from_millis(1000 * (1 << attempt));
            tokio::time::sleep(backoff).await;
        }
    }
    Err(last_err.unwrap_or_else(|| ToksaveError::network("download", "unknown error", url, None)))
}

pub async fn fetch_with_retry(url: &str, opts: &DownloadOptions) -> Result<Vec<u8>> {
    fetch_with_retry_inner(url, opts).await
}

pub async fn fetch_text(url: &str) -> Result<String> {
    let opts = DownloadOptions {
        timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let bytes = fetch_with_retry_inner(url, &opts).await?;
    String::from_utf8(bytes)
        .map_err(|e| ToksaveError::network("text", &format!("invalid UTF-8: {e}"), url, None))
}

pub async fn fetch_json(url: &str) -> Result<serde_json::Value> {
    let opts = DownloadOptions {
        timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let bytes = fetch_with_retry_inner(url, &opts).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| ToksaveError::network("json", &format!("invalid JSON: {e}"), url, None))
}

/// Latest published version of an npm package via the registry `latest`
/// endpoint (cheap, single small doc). Returns None on any failure so callers
/// can degrade gracefully (e.g. offline).
pub async fn latest_npm_version(package: &str) -> Option<String> {
    let url = format!(
        "https://registry.npmjs.org/{}/latest",
        urlencoding::encode(package)
    );
    match fetch_json(&url).await {
        Ok(json) => Some(json.get("version").and_then(|v| v.as_str())?.to_string()),
        Err(_) => None,
    }
}

pub fn is_safe_archive_path(entry_path: &str, dest_dir: &Path) -> bool {
    let p = Path::new(entry_path);
    if p.is_absolute() {
        return false;
    }
    let mut components = Vec::new();
    for c in p.components() {
        use std::path::Component;
        match c {
            Component::Normal(s) => components.push(s.to_string_lossy().into_owned()),
            Component::ParentDir => return false,
            Component::CurDir => {}
            _ => return false,
        }
    }
    let target = components
        .iter()
        .fold(dest_dir.to_path_buf(), |acc, part| acc.join(part));
    target.starts_with(dest_dir)
}

pub async fn download_tar_gz(url: &str, dest_dir: &Path, opts: &DownloadOptions) -> Result<()> {
    let bytes = fetch_with_retry_inner(url, opts).await?;
    if let Some(expected) = &opts.checksum {
        verify_checksum_sha256(&bytes, expected, url)?;
    }
    crate::util::paths::ensure_dir(dest_dir)?;

    let mut decoder = flate2::read::GzDecoder::new(bytes.as_slice());
    let mut tar_bytes = Vec::new();
    decoder
        .read_to_end(&mut tar_bytes)
        .map_err(|e| ToksaveError::network("tar.gz", &format!("corrupt gzip: {e}"), url, None))?;

    let mut archive = tar::Archive::new(tar_bytes.as_slice());
    let entries: Vec<_> = archive
        .entries()
        .map_err(|e| ToksaveError::network("tar.gz", &e.to_string(), url, None))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| ToksaveError::network("tar.gz", &e.to_string(), url, None))?;

    for entry in entries {
        let path = entry
            .path()
            .map_err(|e| ToksaveError::network("tar.gz", &e.to_string(), url, None))?
            .to_string_lossy()
            .into_owned();
        if !is_safe_archive_path(&path, dest_dir) {
            return Err(ToksaveError::download(
                "tar.gz",
                &format!("Archive entry escapes destination: {path}"),
                url,
                Some("The downloaded archive contains malicious entries. Aborting extraction."),
            ));
        }
        let target = dest_dir.join(&path);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            crate::util::paths::ensure_dir(&target)?;
        } else if kind.is_file() {
            if let Some(parent) = target.parent() {
                crate::util::paths::ensure_dir(parent)?;
            }
            let mut out = std::fs::File::create(&target)?;
            let mut e = entry;
            std::io::copy(&mut e, &mut out)?;
        }
    }
    Ok(())
}

pub async fn download_zip(url: &str, dest_dir: &Path, opts: &DownloadOptions) -> Result<()> {
    let bytes = fetch_with_retry_inner(url, opts).await?;
    if let Some(expected) = &opts.checksum {
        verify_checksum_sha256(&bytes, expected, url)?;
    }
    crate::util::paths::ensure_dir(dest_dir)?;

    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| ToksaveError::network("zip", &format!("corrupt zip: {e}"), url, None))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ToksaveError::network("zip", &e.to_string(), url, None))?;
        let path = entry.name().to_string();
        if !is_safe_archive_path(&path, dest_dir) {
            return Err(ToksaveError::download(
                "zip",
                &format!("Zip entry escapes destination: {path}"),
                url,
                Some("The downloaded archive contains malicious entries. Aborting extraction."),
            ));
        }
        let target = dest_dir.join(&path);
        if entry.is_dir() {
            crate::util::paths::ensure_dir(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                crate::util::paths::ensure_dir(parent)?;
            }
            let mut out = std::fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

#[cfg_attr(not(unix), allow(unused_variables))]
pub fn make_executable(path: &Path) -> Result<()> {
    if cfg!(windows) {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn verify_checksum_sha256(bytes: &[u8], expected: &str, url: &str) -> Result<()> {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    let actual = format!("{:x}", h.finalize());
    if actual != expected.to_lowercase() {
        return Err(ToksaveError::integrity(
            "downloaded file",
            &format!("Checksum mismatch for {url} (expected {expected}, got {actual})"),
            Some(
                "The downloaded file may be corrupted or tampered with. Try again or verify the source.",
            ),
        ));
    }
    Ok(())
}
