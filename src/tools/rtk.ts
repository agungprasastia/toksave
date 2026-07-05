import { existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import type { RunOpts } from "../registry.js";
import { verbose } from "../util/colors.js";
import { isOnPath } from "../util/detect.js";
import { downloadTarGz, downloadZip, makeExecutable } from "../util/download.js";
import { DownloadError, InstallError, PlatformError } from "../util/errors.js";
import { run, runStdout } from "../util/exec.js";
import type { HealthIssue, HealthStatus, RepairResult } from "../util/health.js";
import { ensureDir, localBin } from "../util/paths.js";
import { userAgent } from "../util/version.js";

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

export function isInstalledButUnreachable(): boolean {
  if (isOnPath("rtk")) return false;
  const local = join(localBin(), process.platform === "win32" ? "rtk.exe" : "rtk");
  return existsSync(local);
}

/** Install RTK (prebuilt binary from GitHub releases). */
export async function install(opts: RunOpts): Promise<boolean> {
  if (installedVersion() && !opts.upgrade) return true;
  if (opts.dryRun) return true;

  const asset = assetName();
  if (asset) {
    const url = `https://github.com/rtk-ai/rtk/releases/latest/download/${asset}`;
    const dest = localBin();
    ensureDir(dest);

    verbose(`Downloading RTK from ${url}`, opts.verbose);

    try {
      if (asset.endsWith(".tar.gz")) {
        await downloadTarGz(url, dest);
        makeExecutable(join(dest, "rtk"));
      } else if (asset.endsWith(".zip")) {
        await downloadZip(url, dest);
      }

      verbose("Running rtk init -g", opts.verbose);
      const rtkPath = join(dest, process.platform === "win32" ? "rtk.exe" : "rtk");
      const initResult = run(rtkPath, ["init", "-g"]);

      if (initResult.code !== 0) {
        try {
          rmSync(rtkPath, { force: true });
        } catch {
          /* ignore */
        }
        throw new InstallError("rtk", {
          message: "Failed to initialize RTK shell integration",
          cause: initResult.stderr,
          remediation: "Try running 'rtk init -g' manually after installation completes",
        });
      }

      return true;
    } catch (err) {
      const wrapped =
        err instanceof InstallError
          ? err
          : new DownloadError("rtk", url, {
              message: "Failed to download RTK binary",
              cause: err,
              remediation: "Trying fallback installation methods.",
            });
      verbose(`${wrapped.message}; trying fallback installation methods`, opts.verbose);
    }
  }

  // Fallback: official install script (Unix only)
  if (process.platform !== "win32") {
    if (isOnPath("curl") && isOnPath("sh")) {
      verbose("Falling back to RTK install script", opts.verbose);
      try {
        const r = run("sh", [
          "-c",
          "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh",
        ]);
        if (r.code === 0) {
          const init = run("rtk", ["init", "-g"]);
          if (init.code === 0) return true;
          throw new InstallError("rtk", {
            message: "Failed to initialize RTK shell integration",
            cause: init.stderr,
            remediation: "Try running 'rtk init -g' manually after installation completes",
          });
        }
      } catch {
        verbose("Install script failed, trying cargo", opts.verbose);
      }
    }
  }

  // Fallback: cargo install
  if (isOnPath("cargo")) {
    verbose("Falling back to cargo install", opts.verbose);
    try {
      const r = run("cargo", ["install", "--git", "https://github.com/rtk-ai/rtk"]);
      if (r.code === 0) {
        const init = run("rtk", ["init", "-g"]);
        if (init.code === 0) return true;
        throw new InstallError("rtk", {
          message: "Failed to initialize RTK shell integration",
          cause: init.stderr,
          remediation: "Try running 'rtk init -g' manually after installation completes",
        });
      }
    } catch (err) {
      throw new InstallError("rtk", {
        message: "All installation methods failed",
        cause: err,
        remediation: "Install manually: https://github.com/rtk-ai/rtk#installation",
      });
    }
  }

  const platform = `${process.platform}-${process.arch}`;
  throw new PlatformError("rtk", platform, {
    message: `No installation method available for ${platform}`,
    remediation: "Visit https://github.com/rtk-ai/rtk for manual installation instructions",
  });
}

/** Get installed RTK version. */
export function installedVersion(): string | null {
  const pathVersion = runStdout("rtk", ["--version"])?.trim();
  if (pathVersion) return pathVersion;

  const local = join(localBin(), process.platform === "win32" ? "rtk.exe" : "rtk");
  if (!existsSync(local)) return null;
  return runStdout(local, ["--version"])?.trim() ?? null;
}

/** Get latest RTK version from GitHub. */
export async function latestVersion(): Promise<string | null> {
  try {
    const res = await fetch("https://api.github.com/repos/rtk-ai/rtk/releases/latest", {
      headers: { "User-Agent": userAgent() },
      signal: AbortSignal.timeout(10000),
    });
    if (!res.ok) return null;
    const json = await res.json();
    return json.tag_name?.replace(/^v/, "").trim() || null;
  } catch {
    return null;
  }
}

/** Check if RTK is installed and working. */
export function healthCheck(): HealthStatus {
  const issues: HealthIssue[] = [];
  const version = installedVersion();

  if (!version) {
    return {
      healthy: false,
      version: null,
      issues: [
        {
          severity: "error",
          message: "RTK is not installed",
          remediation: "Run: toksave install rtk",
        },
      ],
    };
  }

  if (isInstalledButUnreachable()) {
    const binPath = localBin();
    const instruct =
      process.platform === "win32"
        ? `Add ${binPath} to your system PATH via System Properties or setx PATH "%PATH%;${binPath}"`
        : `Add 'export PATH="$PATH:${binPath}"' to your shell rc file (e.g. ~/.bashrc or ~/.zshrc) and restart your terminal`;

    issues.push({
      severity: "error",
      message: `RTK is installed in ${binPath} but is not on your PATH`,
      remediation: `Enforcement hooks will fail with 'command not found'. ${instruct}`,
    });
  }

  return {
    healthy: issues.length === 0 || issues.every((i) => i.severity === "warning"),
    version,
    issues,
  };
}

/** Attempt to repair a broken RTK installation. */
export async function repair(opts: RunOpts): Promise<RepairResult> {
  try {
    const beforeHealth = healthCheck();

    if (beforeHealth.healthy) {
      return {
        success: true,
        message: "RTK is already healthy, no repair needed",
        healthAfterRepair: beforeHealth,
      };
    }

    await install({ ...opts, upgrade: true });

    const afterHealth = healthCheck();

    if (afterHealth.healthy) {
      return {
        success: true,
        message: "RTK successfully repaired",
        healthAfterRepair: afterHealth,
      };
    }

    return {
      success: false,
      message: "Repair attempted but health check still failing",
      healthAfterRepair: afterHealth,
    };
  } catch (err) {
    return {
      success: false,
      message: `Repair failed: ${err instanceof Error ? err.message : String(err)}`,
    };
  }
}
