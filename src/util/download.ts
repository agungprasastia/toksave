import { createHash } from "node:crypto";
import { chmodSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, normalize, relative } from "node:path";
import AdmZip from "adm-zip";
import * as tar from "tar";
import { DownloadError, IntegrityError, NetworkError } from "./errors.js";
import { userAgent } from "./version.js";

export interface DownloadOptions {
  retries?: number;
  timeout?: number;
  onProgress?: (downloaded: number, total: number) => void;
  checksum?: string;
  fallbackUrls?: string[];
}

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function verifyChecksum(buffer: Buffer, expectedChecksum: string, url: string): void {
  const actualChecksum = createHash("sha256").update(buffer).digest("hex");
  if (actualChecksum !== expectedChecksum) {
    throw new IntegrityError("downloaded file", {
      message: `Checksum mismatch for ${url}`,
      expected: expectedChecksum,
      actual: actualChecksum,
      remediation:
        "The downloaded file may be corrupted or tampered with. Try again or verify the source.",
    });
  }
}

async function fetchWithRetry(url: string, opts: DownloadOptions = {}): Promise<Response> {
  const urls = [url, ...(opts.fallbackUrls ?? [])];
  let lastError: Error | null = null;

  for (const currentUrl of urls) {
    const retries = opts.retries ?? 3;
    const timeout = opts.timeout ?? 120_000;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const resp = await fetch(currentUrl, {
          headers: { "User-Agent": userAgent() },
          signal: AbortSignal.timeout(timeout),
        });

        if (resp.ok) return resp;

        // Don't retry 404s
        if (resp.status === 404) {
          throw new DownloadError("resource", currentUrl, {
            message: `HTTP ${resp.status} ${resp.statusText}`,
            statusCode: resp.status,
            remediation: "URL not found. Check if the resource exists or try a different version.",
          });
        }

        throw new Error(`HTTP ${resp.status} ${resp.statusText}`);
      } catch (err) {
        lastError = err as Error;

        // Don't retry 404s or on last attempt
        if (
          attempt === retries ||
          (err instanceof DownloadError && err.context.statusCode === 404)
        ) {
          break;
        }

        // Exponential backoff: 1s, 2s, 4s
        const backoffMs = 1000 * 2 ** attempt;
        await sleep(backoffMs);
      }
    }
  }

  throw lastError;
}

async function fetchBuffer(resp: Response, opts?: DownloadOptions): Promise<Buffer> {
  const contentLength = Number(resp.headers.get("content-length")) || 0;
  let downloaded = 0;

  const reader = resp.body?.getReader();
  if (!reader) throw new Error("No response body");

  const chunks: Uint8Array[] = [];

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    chunks.push(value);
    downloaded += value.length;

    if (opts?.onProgress) {
      opts.onProgress(downloaded, contentLength);
    }
  }

  return Buffer.concat(chunks);
}

function isSafeArchivePath(entryPath: string, destDir: string): boolean {
  if (isAbsolute(entryPath)) return false;
  const normalized = normalize(entryPath);
  if (normalized.startsWith("..") || normalized.includes("..\\") || normalized.includes("../")) {
    return false;
  }
  const targetPath = join(destDir, entryPath);
  return !relative(destDir, targetPath).startsWith("..");
}

/** Download a URL to a file path. */
export async function downloadFile(
  url: string,
  dest: string,
  opts?: DownloadOptions,
): Promise<void> {
  try {
    const resp = await fetchWithRetry(url, opts);

    const parent = dirname(dest);
    if (!existsSync(parent)) mkdirSync(parent, { recursive: true });

    const buffer = await fetchBuffer(resp, opts);
    if (opts?.checksum) {
      verifyChecksum(buffer, opts.checksum, url);
    }
    await Bun.write(dest, buffer);
  } catch (err) {
    if (err instanceof DownloadError || err instanceof IntegrityError) throw err;

    throw new NetworkError("file", {
      message: `Network error downloading from ${url}`,
      cause: err,
      url,
      remediation:
        "Check your internet connection. If behind a proxy, ensure proxy settings are configured.",
    });
  }
}

/** Download and extract a .tar.gz to a destination directory. */
export async function downloadTarGz(
  url: string,
  destDir: string,
  opts?: DownloadOptions,
): Promise<void> {
  try {
    const resp = await fetchWithRetry(url, opts);

    if (!existsSync(destDir)) mkdirSync(destDir, { recursive: true });

    const buffer = await fetchBuffer(resp, opts);
    if (opts?.checksum) {
      verifyChecksum(buffer, opts.checksum, url);
    }
    const tmpFile = join(destDir, `__toksave_tmp_${process.pid}_${Date.now()}.tar.gz`);
    await Bun.write(tmpFile, buffer);

    await tar.x({
      file: tmpFile,
      cwd: destDir,
      filter: (entryPath) => isSafeArchivePath(entryPath, destDir),
    });

    const { unlinkSync } = await import("node:fs");
    try {
      unlinkSync(tmpFile);
    } catch {
      /* ignore */
    }
  } catch (err) {
    if (err instanceof DownloadError || err instanceof IntegrityError) throw err;

    throw new NetworkError("tar.gz", {
      message: `Failed to download and extract tar.gz from ${url}`,
      cause: err,
      url,
      remediation: "Check your internet connection or try a different installation method.",
    });
  }
}

/** Download and extract a .zip to a destination directory. */
export async function downloadZip(
  url: string,
  destDir: string,
  opts?: DownloadOptions,
): Promise<void> {
  try {
    const resp = await fetchWithRetry(url, opts);

    if (!existsSync(destDir)) mkdirSync(destDir, { recursive: true });

    const buffer = await fetchBuffer(resp, opts);
    if (opts?.checksum) {
      verifyChecksum(buffer, opts.checksum, url);
    }
    const zip = new AdmZip(buffer);

    // Validate and extract entries to prevent zip slip
    for (const entry of zip.getEntries()) {
      const entryPath = entry.entryName;

      if (!isSafeArchivePath(entryPath, destDir)) {
        throw new DownloadError("zip", url, {
          message: `Zip entry escapes destination: ${entryPath}`,
          remediation: "The downloaded archive contains malicious entries. Aborting extraction.",
        });
      }

      const targetPath = join(destDir, entryPath);

      // Extract entry
      if (entry.isDirectory) {
        mkdirSync(targetPath, { recursive: true });
      } else {
        const parent = dirname(targetPath);
        if (!existsSync(parent)) mkdirSync(parent, { recursive: true });
        writeFileSync(targetPath, entry.getData());
      }
    }
  } catch (err) {
    if (err instanceof DownloadError || err instanceof IntegrityError) throw err;

    throw new NetworkError("zip", {
      message: `Failed to download and extract zip from ${url}`,
      cause: err,
      url,
      remediation: "Check your internet connection or try a different installation method.",
    });
  }
}

/** Fetch JSON from a URL. */
export async function fetchJson(url: string): Promise<unknown> {
  const resp = await fetchWithRetry(url, { timeout: 10_000 });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} for ${url}`);
  return resp.json();
}

/** Make a file executable (Unix only). */
export function makeExecutable(path: string): void {
  if (process.platform !== "win32") {
    try {
      chmodSync(path, 0o755);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      throw new Error(
        `Failed to make ${path} executable: ${msg}. Check file permissions and try: chmod +x ${path}`,
      );
    }
  }
}
