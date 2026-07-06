import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  ensureProcessPath,
  expectedBinDirs,
  formatPathFixResult,
  renderUnixBlock,
  upsertShellBlock,
} from "../util/pathfix.js";

let tmp = "";
let oldHome: string | undefined;
let oldPath: string | undefined;
let oldShell: string | undefined;

beforeEach(() => {
  oldHome = process.env.HOME;
  oldPath = process.env.PATH;
  oldShell = process.env.SHELL;

  tmp = mkdtempSync(join(tmpdir(), "toksave-pathfix-test-"));
  process.env.HOME = join(tmp, "home");
  mkdirSync(join(tmp, "home"), { recursive: true });
});

afterEach(() => {
  if (oldHome === undefined) delete process.env.HOME;
  else process.env.HOME = oldHome;
  if (oldPath === undefined) delete process.env.PATH;
  else process.env.PATH = oldPath;
  if (oldShell === undefined) delete process.env.SHELL;
  else process.env.SHELL = oldShell;
  rmSync(tmp, { recursive: true, force: true });
});

describe("ensureProcessPath", () => {
  test("prepends existing expected dirs to PATH", () => {
    const binDir = expectedBinDirs()[0]!;
    mkdirSync(binDir, { recursive: true });

    // Remove binDir from PATH if present
    const sep = process.platform === "win32" ? ";" : ":";
    process.env.PATH = (process.env.PATH ?? "")
      .split(sep)
      .filter((p) => p !== binDir)
      .join(sep);

    const added = ensureProcessPath();
    expect(added).toContain(binDir);
    expect(process.env.PATH?.startsWith(binDir)).toBe(true);
  });

  test("does not add dirs that do not exist", () => {
    // HOME points to temp, so expected dirs don't exist
    process.env.PATH = "/usr/bin";
    const added = ensureProcessPath();
    expect(added).toHaveLength(0);
  });

  test("idempotent — does not duplicate", () => {
    const binDir = expectedBinDirs()[0]!;
    mkdirSync(binDir, { recursive: true });

    const sep = process.platform === "win32" ? ";" : ":";
    process.env.PATH = ["/usr/bin"].join(sep);

    ensureProcessPath();
    const pathAfterFirst = process.env.PATH;
    ensureProcessPath();
    expect(process.env.PATH).toBe(pathAfterFirst);
  });
});

describe("upsertShellBlock", () => {
  const block = renderUnixBlock(["/home/user/.local/bin"], "/home/user");

  test("appends block to empty file", () => {
    const result = upsertShellBlock("", block);
    expect(result).toContain(">>> toksave path >>>");
    expect(result).toContain("<<< toksave path <<<");
    expect(result).toContain("$HOME/.local/bin");
  });

  test("appends block to existing content", () => {
    const result = upsertShellBlock("export FOO=bar\n", block);
    expect(result).toStartWith("export FOO=bar\n");
    expect(result).toContain(">>> toksave path >>>");
  });

  test("replaces existing block on re-run", () => {
    const first = upsertShellBlock("# my rc\n", block);
    const newBlock = renderUnixBlock(
      ["/home/user/.local/bin", "/home/user/.cargo/bin"],
      "/home/user",
    );
    const second = upsertShellBlock(first, newBlock);

    // Only one block
    expect(second.split(">>> toksave path >>>").length - 1).toBe(1);
    expect(second).toContain(".cargo/bin");
  });

  test("preserves surrounding content on replace", () => {
    const before = `# before\nexport A=1\n`;
    const after = `# after stuff\n`;
    const content = `${before}${block}${after}`;
    const newBlock = renderUnixBlock(["/new/dir"], "/home/user");
    const result = upsertShellBlock(content, newBlock);
    expect(result).toStartWith(before);
    expect(result).toContain("# after stuff");
    expect(result).toContain("/new/dir");
  });
});

describe("renderUnixBlock", () => {
  test("uses $HOME for paths under home", () => {
    const block = renderUnixBlock(["/home/user/.local/bin", "/opt/bin"], "/home/user");
    expect(block).toContain('"$HOME/.local/bin"');
    expect(block).toContain('"/opt/bin"');
  });
});

describe("formatPathFixResult", () => {
  test("returns null when nothing changed", () => {
    expect(formatPathFixResult({ added: [], patched: [] })).toBeNull();
  });

  test("reports added dirs", () => {
    const msg = formatPathFixResult({ added: ["/a", "/b"], patched: [] });
    expect(msg).toContain("process PATH updated (+2)");
  });

  test("reports patched rc files", () => {
    const msg = formatPathFixResult({ added: [], patched: ["/home/user/.zshrc"] });
    expect(msg).toContain("persisted to");
    expect(msg).toContain(".zshrc");
  });

  test("reports both added and patched", () => {
    const msg = formatPathFixResult({ added: ["/a"], patched: ["/home/user/.bashrc"] });
    expect(msg).toContain("process PATH updated (+1)");
    expect(msg).toContain("persisted to");
  });
});
