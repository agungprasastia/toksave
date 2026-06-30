import { existsSync, readFileSync } from "node:fs";
import { writeFile } from "../util/paths.js";

/** Strip single-line // comments from JSON (JSONC → JSON). */
function stripComments(input: string): string {
  let out = "";
  let inString = false;
  let isEscaped = false;

  for (let i = 0; i < input.length; i++) {
    const ch = input[i];

    if (isEscaped) {
      out += ch;
      isEscaped = false;
      continue;
    }
    if (ch === "\\" && inString) {
      out += ch;
      isEscaped = true;
      continue;
    }
    if (ch === '"') {
      inString = !inString;
      out += ch;
      continue;
    }
    if (!inString && ch === "/" && input[i + 1] === "/") {
      // Skip until end of line
      while (i < input.length && input[i] !== "\n") i++;
      out += "\n";
      continue;
    }
    out += ch;
  }
  return out;
}

/** Read a JSON/JSONC file. */
export function readJsonFile(path: string): unknown | null {
  if (!existsSync(path)) return null;
  try {
    const raw = readFileSync(path, "utf-8");
    return JSON.parse(stripComments(raw));
  } catch {
    return null;
  }
}

/** Write a JSON value to a file, pretty-printed. */
export function writeJsonFile(path: string, value: unknown): void {
  writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
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
