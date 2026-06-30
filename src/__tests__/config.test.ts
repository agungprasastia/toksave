import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  addToArrayIfMissing,
  getOrCreateObject,
  readJsonFile,
  removeFromArray,
  writeJsonFile,
} from "../config/json.js";
import { hasTable, readTomlFile, removeTable, upsertTable, writeTomlFile } from "../config/toml.js";

const TMP = join(import.meta.dir, "__tmp_config__");

beforeEach(() => {
  mkdirSync(TMP, { recursive: true });
});

afterEach(() => {
  rmSync(TMP, { recursive: true, force: true });
});

describe("JSON config", () => {
  test("readJsonFile returns null for missing file", () => {
    expect(readJsonFile(join(TMP, "nope.json"))).toBeNull();
  });

  test("writeJsonFile + readJsonFile round-trip", () => {
    const path = join(TMP, "test.json");
    const data = { foo: "bar", num: 42 };
    writeJsonFile(path, data);
    const read = readJsonFile(path);
    expect(read).toEqual(data);
  });

  test("strips JSONC comments", () => {
    const path = join(TMP, "commented.json");
    writeFileSync(path, '{\n  // comment\n  "key": "val"\n}');
    const read = readJsonFile(path);
    expect(read).toEqual({ key: "val" });
  });

  test("getOrCreateObject creates nested key", () => {
    const obj: any = {};
    const sub = getOrCreateObject(obj, "mcpServers");
    sub.test = true;
    expect(obj.mcpServers.test).toBe(true);
  });

  test("addToArrayIfMissing avoids duplicates", () => {
    const arr = ["a", "b"];
    addToArrayIfMissing(arr, "b");
    addToArrayIfMissing(arr, "c");
    expect(arr).toEqual(["a", "b", "c"]);
  });

  test("removeFromArray removes entry", () => {
    const arr = ["a", "b", "c"];
    removeFromArray(arr, "b");
    expect(arr).toEqual(["a", "c"]);
  });
});

describe("TOML config", () => {
  test("readTomlFile returns empty for missing file", () => {
    expect(readTomlFile(join(TMP, "nope.toml"))).toEqual({});
  });

  test("writeTomlFile + readTomlFile round-trip", () => {
    const path = join(TMP, "test.toml");
    const data = { name: "test", version: "1.0" };
    writeTomlFile(path, data);
    const read = readTomlFile(path);
    expect(read.name).toBe("test");
    expect(read.version).toBe("1.0");
  });

  test("upsertTable creates nested table", () => {
    const doc: Record<string, any> = {};
    upsertTable(doc, "mcp_servers.codegraph", { command: "codegraph" });
    expect(doc.mcp_servers.codegraph.command).toBe("codegraph");
  });

  test("hasTable checks existence", () => {
    const doc = { a: { b: { c: 1 } } };
    expect(hasTable(doc, "a.b")).toBe(true);
    expect(hasTable(doc, "a.x")).toBe(false);
  });

  test("removeTable deletes nested path", () => {
    const doc = { a: { b: { c: 1 }, d: 2 } };
    removeTable(doc, "a.b");
    expect(doc.a.d).toBe(2);
    expect((doc.a as any).b).toBeUndefined();
  });
});
