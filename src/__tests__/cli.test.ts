import { describe, expect, test } from "bun:test";
import { parseCli } from "../cli.js";

describe("CLI parser", () => {
  test("default command is init", () => {
    const cli = parseCli(["node", "toksave"]);
    expect(cli.command).toBe("init");
  });

  test("doctor command", () => {
    const cli = parseCli(["node", "toksave", "doctor"]);
    expect(cli.command).toBe("doctor");
  });

  test("update command", () => {
    const cli = parseCli(["node", "toksave", "update"]);
    expect(cli.command).toBe("update");
  });

  test("uninstall command", () => {
    const cli = parseCli(["node", "toksave", "uninstall"]);
    expect(cli.command).toBe("uninstall");
  });

  test("self-update command", () => {
    const cli = parseCli(["node", "toksave", "self-update"]);
    expect(cli.command).toBe("self-update");
  });

  test("--dry-run flag", () => {
    const cli = parseCli(["node", "toksave", "--dry-run"]);
    expect(cli.opts.dryRun).toBe(true);
  });

  test("--verbose flag", () => {
    const cli = parseCli(["node", "toksave", "--verbose"]);
    expect(cli.opts.verbose).toBe(true);
  });

  test("--yes flag", () => {
    const cli = parseCli(["node", "toksave", "--yes"]);
    expect(cli.opts.yes).toBe(true);
  });

  test("--agents parses comma-separated", () => {
    const cli = parseCli(["node", "toksave", "--agents", "claude,antigravity"]);
    expect(cli.agents).toEqual(["claude", "antigravity"]);
  });

  test("--tools parses comma-separated", () => {
    const cli = parseCli(["node", "toksave", "--tools", "rtk,caveman"]);
    expect(cli.tools).toEqual(["rtk", "caveman"]);
  });

  test("context-mode tool alias", () => {
    const cli = parseCli(["node", "toksave", "--tools", "contextmode"]);
    expect(cli.tools).toEqual(["context-mode"]);
  });

  test("invalid agent ignored", () => {
    const cli = parseCli(["node", "toksave", "--agents", "invalid"]);
    expect(cli.agents).toEqual([]);
  });

  test("combined flags", () => {
    const cli = parseCli([
      "node",
      "toksave",
      "--dry-run",
      "--verbose",
      "--yes",
      "--agents",
      "claude",
      "--tools",
      "rtk,codegraph",
    ]);
    expect(cli.opts.dryRun).toBe(true);
    expect(cli.opts.verbose).toBe(true);
    expect(cli.opts.yes).toBe(true);
    expect(cli.agents).toEqual(["claude"]);
    expect(cli.tools).toEqual(["rtk", "codegraph"]);
  });
});
