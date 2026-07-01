import { expect, test } from "bun:test";
import { writeFileSync, unlinkSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { isNodeShebangScript } from "../commands/runmcp.js";

test("isNodeShebangScript correctly identifies shebang", () => {
  const p = join(tmpdir(), "toksave-test-shebang.txt");
  try {
    writeFileSync(p, "#!/usr/bin/env node\nconsole.log('hi');");
    expect(isNodeShebangScript(p)).toBe(true);

    writeFileSync(p, "console.log('no shebang');");
    expect(isNodeShebangScript(p)).toBe(false);
  } finally {
    try { unlinkSync(p); } catch {}
  }
});

test("isNodeShebangScript handles invalid or inaccessible paths safely without leaking", () => {
  // Pass a path that definitely doesn't exist, it should return false gracefully
  const p = join(tmpdir(), "toksave-test-does-not-exist.txt");
  expect(isNodeShebangScript(p)).toBe(false);
});
