import { existsSync, readFileSync } from "fs";
import { parse, stringify } from "smol-toml";
import { writeFile } from "../util/paths.js";

/** Read a TOML file, returning the parsed object. */
export function readTomlFile(path: string): Record<string, any> {
  if (!existsSync(path)) return {};
  try {
    const raw = readFileSync(path, "utf-8");
    return parse(raw) as Record<string, any>;
  } catch {
    return {};
  }
}

/** Write a TOML object to a file. */
export function writeTomlFile(path: string, doc: Record<string, any>): void {
  writeFile(path, stringify(doc) + "\n");
}

/** Upsert a dotted table path with key-value pairs. */
export function upsertTable(
  doc: Record<string, any>,
  tablePath: string,
  pairs: Record<string, string>
): void {
  const parts = tablePath.split(".");
  let current = doc;
  for (const part of parts) {
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part];
  }
  for (const [key, value] of Object.entries(pairs)) {
    current[key] = value;
  }
}

/** Upsert a table with a boolean value. */
export function upsertTableBool(
  doc: Record<string, any>,
  tablePath: string,
  key: string,
  value: boolean
): void {
  const parts = tablePath.split(".");
  let current = doc;
  for (const part of parts) {
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part];
  }
  current[key] = value;
}

/** Set a top-level key. */
export function setTopKey(doc: Record<string, any>, key: string, value: string): void {
  doc[key] = value;
}

/** Set an array of strings in a dotted table path. */
export function setTableArray(
  doc: Record<string, any>,
  tablePath: string,
  key: string,
  values: string[]
): void {
  const parts = tablePath.split(".");
  let current = doc;
  for (const part of parts) {
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part];
  }
  current[key] = values;
}

/** Check if a dotted table path exists. */
export function hasTable(doc: Record<string, any>, tablePath: string): boolean {
  const parts = tablePath.split(".");
  let current: any = doc;
  for (const part of parts) {
    if (!current || typeof current !== "object" || !(part in current)) {
      return false;
    }
    current = current[part];
  }
  return typeof current === "object";
}

/** Remove a dotted table path. */
export function removeTable(doc: Record<string, any>, tablePath: string): void {
  const parts = tablePath.split(".");
  if (parts.length === 0) return;
  if (parts.length === 1) {
    delete doc[parts[0]];
    return;
  }
  let current: any = doc;
  for (let i = 0; i < parts.length - 1; i++) {
    if (!current || typeof current !== "object") return;
    current = current[parts[i]];
  }
  if (current && typeof current === "object") {
    delete current[parts[parts.length - 1]];
  }
}
