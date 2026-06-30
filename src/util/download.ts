import { chmodSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import AdmZip from "adm-zip";
import * as tar from "tar";

/** Download a URL to a file path. */
export async function downloadFile(url: string, dest: string): Promise<void> {
  const resp = await fetch(url, {
    headers: { "User-Agent": "toksave" },
    signal: AbortSignal.timeout(120_000),
  });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} for ${url}`);

  const parent = dirname(dest);
  if (!existsSync(parent)) mkdirSync(parent, { recursive: true });

  const buffer = Buffer.from(await resp.arrayBuffer());
  await Bun.write(dest, buffer);
}

/** Download and extract a .tar.gz to a destination directory. */
export async function downloadTarGz(url: string, destDir: string): Promise<void> {
  const resp = await fetch(url, {
    headers: { "User-Agent": "toksave" },
    signal: AbortSignal.timeout(120_000),
  });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} for ${url}`);

  if (!existsSync(destDir)) mkdirSync(destDir, { recursive: true });

  const buffer = Buffer.from(await resp.arrayBuffer());
  const tmpFile = join(destDir, "__toksave_tmp.tar.gz");
  await Bun.write(tmpFile, buffer);

  await tar.x({ file: tmpFile, cwd: destDir });

  const { unlinkSync } = await import("node:fs");
  try {
    unlinkSync(tmpFile);
  } catch {
    /* ignore */
  }
}

/** Download and extract a .zip to a destination directory. */
export async function downloadZip(url: string, destDir: string): Promise<void> {
  const resp = await fetch(url, {
    headers: { "User-Agent": "toksave" },
    signal: AbortSignal.timeout(120_000),
  });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} for ${url}`);

  if (!existsSync(destDir)) mkdirSync(destDir, { recursive: true });

  const buffer = Buffer.from(await resp.arrayBuffer());
  const zip = new AdmZip(buffer);
  zip.extractAllTo(destDir, true);
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
