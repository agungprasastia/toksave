import { chmodSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import AdmZip from "adm-zip";
import * as tar from "tar";
import { DownloadError, NetworkError } from "./errors.js";

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

async function fetchWithRetry(url: string, opts: DownloadOptions = {}): Promise<Response> {
  const retries = opts.retries ?? 3;
  const timeout = opts.timeout ?? 120_000;
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const resp = await fetch(url, {
        headers: { "User-Agent": "toksave" },
        signal: AbortSignal.timeout(timeout),
      });

      if (resp.ok) return resp;

      // Don't retry 404s
      if (resp.status === 404) {
        throw new DownloadError("resource", url, {
          message: `HTTP ${resp.status} ${resp.statusText}`,
          statusCode: resp.status,
          remediation: "URL not found. Check if the resource exists or try a different version.",
        });
      }

      throw new Error(`HTTP ${resp.status} ${resp.statusText}`);
    } catch (err) {
      lastError = err as Error;

      // Don't retry 404s or on last attempt
      if (attempt === retries || (err as any)?.statusCode === 404) {
        throw err;
      }

      // Exponential backoff: 1s, 2s, 4s
      const backoffMs = 1000 * 2 ** attempt;
      await sleep(backoffMs);
    }
  }

  throw lastError;
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

    const buffer = Buffer.from(await resp.arrayBuffer());
    await Bun.write(dest, buffer);
  } catch (err) {
    if (err instanceof DownloadError) throw err;

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

      if (opts?.onProgress && contentLength > 0) {
        opts.onProgress(downloaded, contentLength);
      }
    }

    const buffer = Buffer.concat(chunks);
    const tmpFile = join(destDir, "__toksave_tmp.tar.gz");
    await Bun.write(tmpFile, buffer);

    await tar.x({ file: tmpFile, cwd: destDir });

    const { unlinkSync } = await import("node:fs");
    try {
      unlinkSync(tmpFile);
    } catch {
      /* ignore */
    }
  } catch (err) {
    if (err instanceof DownloadError) throw err;

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

      if (opts?.onProgress && contentLength > 0) {
        opts.onProgress(downloaded, contentLength);
      }
    }

    const buffer = Buffer.concat(chunks);
    const zip = new AdmZip(buffer);
    zip.extractAllTo(destDir, true);
  } catch (err) {
    if (err instanceof DownloadError) throw err;

    throw new NetworkError("zip", {
      message: `Failed to download and extract zip from ${url}`,
      cause: err,
      url,
      remediation: "Check your internet connection or try a different installation method.",
    });
  }
}

/** Fetch JSON from a URL. */
export async function fetchJson(url: string): Promise<any> {
  const resp = await fetch(url, {
    headers: {
      "User-Agent": "toksave",
      Accept: "application/json",
    },
    signal: AbortSignal.timeout(10_000),
  });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} for ${url}`);
  return resp.json();
}

/** Make a file executable (Unix only). */
export function makeExecutable(path: string): void {
  if (process.platform !== "win32") {
    try {
      chmodSync(path, 0o755);
    } catch {
      /* ignore */
    }
  }
}
