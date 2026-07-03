import { existsSync, readFileSync } from "node:fs";
import { parse, stringify } from "smol-toml";
import { writeFile } from "../util/paths.js";

/** Read a TOML file, returning the parsed object. */
export function readTomlFile(path: string): Record<string, unknown> {
  if (!existsSync(path)) return {};
  try {
    const raw = readFileSync(path, "utf-8");
    return parse(raw) as Record<string, unknown>;
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse TOML config at ${path}: ${msg}`);
  }
}

/** Write a TOML object to a file. */
export function writeTomlFile(path: string, doc: Record<string, unknown>): void {
  writeFile(path, `${stringify(doc)}\n`);
}

/** Upsert a dotted table path with key-value pairs. */
export function upsertTable(
  doc: Record<string, unknown>,
  tablePath: string,
  pairs: Record<string, string>,
): void {
  const parts = tablePath.split(".");
  let current: Record<string, unknown> = doc;
  for (const part of parts) {
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part] as Record<string, unknown>;
  }
  for (const [key, value] of Object.entries(pairs)) {
    current[key] = value;
  }
}

/** Upsert a table with a boolean value. */
export function upsertTableBool(
  doc: Record<string, unknown>,
  tablePath: string,
  key: string,
  value: boolean,
): void {
  const parts = tablePath.split(".");
  let current: Record<string, unknown> = doc;
  for (const part of parts) {
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part] as Record<string, unknown>;
  }
  current[key] = value;
}

/** Set a top-level key. */
export function setTopKey(doc: Record<string, unknown>, key: string, value: string): void {
  doc[key] = value;
}

/** Set an array of strings in a dotted table path. */
export function setTableArray(
  doc: Record<string, unknown>,
  tablePath: string,
  key: string,
  values: string[],
): void {
  const parts = tablePath.split(".");
  let current: Record<string, unknown> = doc;
  for (const part of parts) {
    if (!current[part] || typeof current[part] !== "object") {
      current[part] = {};
    }
    current = current[part] as Record<string, unknown>;
  }
  current[key] = values;
}

/** Check if a dotted table path exists. */
export function hasTable(doc: Record<string, unknown>, tablePath: string): boolean {
  const parts = tablePath.split(".");
  let current: Record<string, unknown> = doc;
  for (const part of parts) {
    if (!current || typeof current !== "object" || !(part in current)) {
      return false;
    }
    current = current[part] as Record<string, unknown>;
  }
  return typeof current === "object";
}

/** Remove a dotted table path. */
export function removeTable(doc: Record<string, unknown>, tablePath: string): boolean {
  const parts = tablePath.split(".");
  if (parts.length === 0) return false;
  const lastPart = parts[parts.length - 1];
  if (!lastPart) return false;
  if (parts.length === 1) {
    const existed = lastPart in doc;
    delete doc[lastPart];
    return existed;
  }
  let current: Record<string, unknown> = doc;
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!part || !current || typeof current !== "object" || !(part in current)) return false;
    current = current[part] as Record<string, unknown>;
  }
  if (current && typeof current === "object" && lastPart in current) {
    delete current[lastPart];
    return true;
  }
  return false;
}
