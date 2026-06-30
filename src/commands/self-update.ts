import pc from "picocolors";
import * as colors from "../util/colors.js";
import { fetchJson } from "../util/download.js";
import { run as execRun, runOk } from "../util/exec.js";
import { toksaveVersion, isUpToDate } from "../util/version.js";

const OWNER = "agungprasastia";
const REPO = "toksave";

/** Run the self-update command. */
export async function run(): Promise<number> {
  colors.banner("toksave self-update", "update the toksave CLI itself");

  const local = toksaveVersion();
  colors.raw(`  local:  ${local}`);

  let latest: string;
  try {
    const json = await fetchJson(
      `https://api.github.com/repos/${OWNER}/${REPO}/releases/latest`
    );
    latest = (json?.tag_name ?? "").replace(/^v/, "");
    if (!latest) throw new Error("No tag_name");
  } catch {
    colors.err("Could not reach GitHub Releases. Try again later.");
    colors.warn("Manual update:");
    colors.raw(
      `  ${pc.cyan(`curl -fsSL https://raw.githubusercontent.com/${OWNER}/${REPO}/main/scripts/install.sh | bash`)}`
    );
    return 1;
  }

  colors.raw(`  latest: ${latest}`);

  if (isUpToDate(local, latest)) {
    colors.ok("Already up to date.");
    return 0;
  }

  colors.raw(`  Updating ${local} → ${latest}…`);

  if (process.platform === "win32") {
    colors.warn("On Windows, run:");
    colors.raw(
      `  ${pc.cyan(`irm https://raw.githubusercontent.com/${OWNER}/${REPO}/main/scripts/install.ps1 | iex`)}`
    );
    return 0;
  }

  // Unix: try curl | bash
  if (runOk("which", ["curl"]) && runOk("which", ["bash"])) {
    const url = `https://raw.githubusercontent.com/${OWNER}/${REPO}/main/scripts/install.sh`;
    const r = execRun("bash", ["-c", `curl -fsSL ${url} | bash`]);
    if (r.code === 0) {
      colors.ok(`Updated to ${latest}. Restart your shell.`);
      return 0;
    }
    colors.err("Auto-update failed.");
  }

  colors.warn("Manual update:");
  colors.raw(
    `  ${pc.cyan(`curl -fsSL https://raw.githubusercontent.com/${OWNER}/${REPO}/main/scripts/install.sh | bash`)}`
  );
  return 0;
}
