import { parse, stringify } from "comment-json";
import { existsSync, readFileSync } from "node:fs";
import { writeFile } from "../util/paths.js";

/** Read a JSON/JSONC file. */
export function readJsonFile(path: string): unknown | null {
  if (!existsSync(path)) return null;
  try {
    const raw = readFileSync(path, "utf-8");
    return parse(raw);
  } catch {
    return null;
  }
}

/** Write a JSON value to a file, pretty-printed. */
export function writeJsonFile(path: string, value: unknown): void {
  writeFile(path, `${stringify(value, null, 2)}\n`);
}

/** Get or create a nested object. */
export function getOrCreateObject(
  parent: Record<string, unknown>,
  key: string,
): Record<string, unknown> {
  if (typeof parent !== "object" || parent === null) return {};
  if (!parent[key] || typeof parent[key] !== "object" || Array.isArray(parent[key])) {
    parent[key] = {};
  }
  return parent[key] as Record<string, unknown>;
}

/** Get or create a nested array. */
export function getOrCreateArray(parent: Record<string, unknown>, key: string): unknown[] {
  if (typeof parent !== "object" || parent === null) return [];
  if (!Array.isArray(parent[key])) {
    parent[key] = [];
  }
  return parent[key] as unknown[];
}

/** Add a string to a JSON array if not already present. */
export function addToArrayIfMissing(arr: unknown[], entry: unknown): void {
  if (!arr.includes(entry)) {
    arr.push(entry);
  }
}

/** Remove a string from a JSON array. */
export function removeFromArray(arr: unknown[], entry: unknown): void {
  const idx = arr.indexOf(entry);
  if (idx !== -1) arr.splice(idx, 1);
}

/** Check if an object has a key. */
export function hasKey(obj: Record<string, unknown>, key: string): boolean {
  return typeof obj === "object" && obj !== null && key in obj;
}

/** Remove a key from an object. */
export function removeKey(obj: Record<string, unknown>, key: string): boolean {
  if (typeof obj === "object" && obj !== null && key in obj) {
    delete obj[key];
    return true;
  }
  return false;
}
