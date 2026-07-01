import { join } from "node:path";
import type { RunOpts } from "../registry.js";
import { verbose } from "../util/colors.js";
import { isOnPath } from "../util/detect.js";
import { downloadTarGz, downloadZip, makeExecutable } from "../util/download.js";
import { run, runStdout } from "../util/exec.js";
import { ensureDir, localBin } from "../util/paths.js";

/** Platform-specific asset name for prebuilt binary. */
function assetName(): string | null {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "darwin" && arch === "x64") return "rtk-x86_64-apple-darwin.tar.gz";
  if (platform === "darwin" && arch === "arm64") return "rtk-aarch64-apple-darwin.tar.gz";
  if (platform === "linux" && arch === "x64") return "rtk-x86_64-unknown-linux-musl.tar.gz";
  if (platform === "linux" && arch === "arm64") return "rtk-aarch64-unknown-linux-musl.tar.gz";
  if (platform === "win32" && arch === "x64") return "rtk-x86_64-pc-windows-msvc.zip";
  return null;
}

/** Install RTK (prebuilt binary from GitHub releases). */
export async function install(opts: RunOpts): Promise<boolean> {
  if (isOnPath("rtk") && !opts.upgrade) return true;
  if (opts.dryRun) return true;

  const asset = assetName();
  if (asset) {
    const url = `https://github.com/rtk-ai/rtk/releases/latest/download/${asset}`;
    const dest = localBin();
    ensureDir(dest);

    verbose(`Downloading RTK from ${url}`, opts.verbose);

    if (asset.endsWith(".tar.gz")) {
      await downloadTarGz(url, dest);
      makeExecutable(join(dest, "rtk"));
    } else if (asset.endsWith(".zip")) {
      await downloadZip(url, dest);
    }

    // Run rtk init -g to activate global shell integration
    verbose("Running rtk init -g", opts.verbose);
    const rtkPath = join(dest, process.platform === "win32" ? "rtk.exe" : "rtk");
    run(rtkPath, ["init", "-g"]);

    return true;
  }

  // Fallback: official install script (Unix only)
  if (process.platform !== "win32") {
    if (isOnPath("curl") && isOnPath("sh")) {
      verbose("Falling back to RTK install script", opts.verbose);
      const r = run("sh", [
        "-c",
        "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh",
      ]);
      if (r.code === 0) {
        run("rtk", ["init", "-g"]);
        return true;
      }
    }
  }

  // Fallback: cargo install
  if (isOnPath("cargo")) {
    verbose("Falling back to cargo install", opts.verbose);
    const r = run("cargo", ["install", "--git", "https://github.com/rtk-ai/rtk"]);
    if (r.code === 0) {
      run("rtk", ["init", "-g"]);
      return true;
    }
  }

  throw new Error("Cannot install rtk. See https://github.com/rtk-ai/rtk for manual install.");
}

/** Get installed RTK version. */
export function installedVersion(): string | null {
  return runStdout("rtk", ["--version"])?.trim() ?? null;
}

/** Get latest RTK version from GitHub. */
export async function latestVersion(): Promise<string | null> {
  try {
    const res = await fetch("https://api.github.com/repos/rtk-ai/rtk/releases/latest", {
      headers: { "User-Agent": "toksave" }
    });
    if (!res.ok) return null;
    const json = await res.json();
    return json.tag_name?.replace(/^v/, "").trim() || null;
  } catch {
    return null;
  }
}
