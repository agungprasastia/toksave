import { rmSync } from "node:fs";
import {
  sectionMarkers,
  TOKLESS_OWNERS,
  type ToklessOwner,
  toklessAgentBody,
} from "../content/agent-instructions.js";
import * as paths from "./paths.js";

type ManagedSection = {
  owner: ToklessOwner;
  lines: string[];
};

// Legacy fences from tokless + TokSave old format
const LEGACY_FENCES: [string, string][] = [
  ["<!-- caveman-begin -->", "<!-- caveman-end -->"],
  ["<!-- CODEGRAPH_START -->", "<!-- CODEGRAPH_END -->"],
  ["<!-- CONTEXT-MODE_START -->", "<!-- CONTEXT-MODE_END -->"],
  ["<!-- CAVEMAN_START", ""], // old TokSave partial — until -->
  ["<!-- RTK_START", ""],
  ["<!-- CONTEXT-MODE_START", ""],
  ["<!-- CODEGRAPH_START", ""],
  ["<!-- tokless:owners=", ""],
];

const LEGACY_HEADINGS = ["## Process Noise"];

// ─── Conflict handling (simplified: auto-append) ──────────────

let _autoAppend = true;

export function configureInstructionConflicts(auto: boolean): void {
  _autoAppend = auto;
}

function instructionPath(agent: string): string {
  switch (agent) {
    case "claude":
      return paths.claudePaths().agentsMd;
    case "opencode":
      return paths.opencodePaths().agentsMd;
    case "codex":
      return paths.codexPaths().instructions;
    case "antigravity":
      return paths.antigravityPaths().agentsMd;
    case "copilot":
      return paths.copilotPaths().instructions;
    case "droid":
      return paths.droidPaths().instructions;
    default:
      return "";
  }
}

function stripLegacy(raw: string): string {
  let out = raw;
  for (const [start, end] of LEGACY_FENCES) {
    if (end === "") {
      // fence like <!-- XXX_START ... --> until closing -->
      // Remove from start marker to next -->
      for (;;) {
        const i = out.indexOf(start);
        if (i < 0) break;
        let j = out.indexOf("-->", i);
        if (j < 0) {
          out = out.slice(0, i);
          break;
        }
        j += "-->".length;
        // consume trailing newlines
        while (j < out.length && out[j] === "\n") j++;
        out = out.slice(0, i) + out.slice(j);
      }
      continue;
    }
    for (;;) {
      const i = out.indexOf(start);
      if (i < 0) break;
      let j = out.indexOf(end, i + start.length);
      if (j < 0) break;
      j += end.length;
      // Trim one preceding newline gap slightly to avoid double blank
      let si = i;
      if (si > 0 && out[si - 1] === "\n") si--;
      while (j < out.length && out[j] === "\n") j++;
      out = out.slice(0, si) + out.slice(j);
    }
  }
  for (const h of LEGACY_HEADINGS) {
    out = stripLegacyHeading(out, h);
  }
  // Also strip old TokSave fences with regex for safety
  out = out.replace(/\r?\n?<!--\s*CAVEMAN_START[\s\S]*?CAVEMAN_END\s*-->\r?\n?/g, "\n");
  out = out.replace(/\r?\n?<!--\s*RTK_START[\s\S]*?RTK_END\s*-->\r?\n?/g, "\n");
  out = out.replace(/\r?\n?<!--\s*CONTEXT-MODE_START[\s\S]*?CONTEXT-MODE_END\s*-->\r?\n?/g, "\n");
  out = out.replace(/\r?\n?<!--\s*CODEGRAPH_START[\s\S]*?CODEGRAPH_END\s*-->\r?\n?/g, "\n");
  return out;
}

function stripLegacyHeading(raw: string, heading: string): string {
  let out = raw;
  for (;;) {
    const i = out.indexOf(heading);
    if (i < 0) return out;
    // must be at line start (preceded by \n or start)
    if (i > 0 && out[i - 1] !== "\n") {
      // not at line start, avoid false positive inside content — skip
      break;
    }
    let start = i;
    if (start > 0 && out[start - 1] === "\n") {
      // also trim previous blank line
      const _peek = start - 2;
      // Keep one newline separator
      // We'll just trim from previous newline
      start--;
    }
    // Find end: next ## heading or EOF
    let end = out.length;
    const rest = out.slice(i + heading.length);
    // Look for next "## " on its own line
    const nextIdx = rest.search(/\n## /);
    if (nextIdx >= 0) {
      end = i + heading.length + nextIdx + 1;
    }
    // Trim trailing blank lines
    while (end > start && (out[end - 1] === "\n" || out[end - 1] === " ")) {
      if (out[end - 1] === "\n") end--;
      else break;
    }
    out = out.slice(0, start) + out.slice(end);
  }
  return out;
}

function ownerOf(line: string): ToklessOwner | "" {
  const trimmed = line.replace(/\r$/, "");
  for (const o of TOKLESS_OWNERS) {
    for (const marker of sectionMarkers(o as ToklessOwner)) {
      if (trimmed === marker) return o as ToklessOwner;
    }
  }
  return "";
}

function fileParts(raw: string): { head: string[]; blocks: ManagedSection[]; tail: string[] } {
  const normalized = raw.replace(/\r/g, "");
  const lines = normalized.split("\n");
  const ownerIdx: number[] = [];
  for (let i = 0; i < lines.length; i++) {
    if (ownerOf(lines[i] as string)) ownerIdx.push(i);
  }
  if (ownerIdx.length === 0) {
    return { head: lines, blocks: [], tail: [] };
  }
  const head = lines.slice(0, ownerIdx[0]).map((l) => l);
  const blocks: ManagedSection[] = [];
  for (let i = 0; i < ownerIdx.length; i++) {
    const start = ownerIdx[i]!;
    const end = i + 1 < ownerIdx.length ? ownerIdx[i + 1]! : lines.length;
    const seg = lines.slice(start, end);
    const b = blocksFromLines(seg);
    blocks.push(...b);
  }
  // Tail handling: in tokless fileParts, tail is nil when owners exist and last block runs to EOF.
  // Our simplification: when owners exist, tail empty (since blocks run to EOF). That's sufficient.
  return { head, blocks, tail: [] };
}

function blocksFromLines(lines: string[]): ManagedSection[] {
  const out: ManagedSection[] = [];
  let cur: ManagedSection | null = null;
  for (const line of lines) {
    const o = ownerOf(line);
    if (o) {
      if (cur) out.push(cur);
      cur = { owner: o, lines: [line] };
      continue;
    }
    if (cur) cur.lines.push(line);
  }
  if (cur) out.push(cur);
  return out;
}

function ownersFromBlocks(blocks: ManagedSection[]): ToklessOwner[] {
  const out: ToklessOwner[] = [];
  for (const b of blocks) {
    // tokless skips principles when listing owners for render order
    if (b.owner === "principles") continue;
    out.push(b.owner);
  }
  return out;
}

function containsOwner(list: string[], want: string): boolean {
  return list.includes(want);
}

function sortOwnersByRegistry(owners: string[]): void {
  const order = new Map<string, number>();
  for (let i = 0; i < TOKLESS_OWNERS.length; i++) order.set(TOKLESS_OWNERS[i]!, i);
  owners.sort((a, b) => (order.get(a) ?? 999) - (order.get(b) ?? 999));
}

function joinManagedBlocks(blocks: ManagedSection[]): string {
  let b = "";
  for (let i = 0; i < blocks.length; i++) {
    if (i > 0) b += "\n\n";
    b += blocks[i]!.lines.join("\n");
  }
  return b;
}

function joinEmpty(head: string, tail: string): string {
  if (!head && !tail) return "";
  if (!head) return tail;
  if (!tail) return head;
  return `${head}\n\n${tail}`;
}

function joinFile(head: string[], body: string, tail: string[]): string {
  const headStr = head.join("\n").trimEnd();
  const tailStr = tail.join("\n").trimStart();
  if (!headStr && !body && !tailStr) return "";
  if (!body) return joinEmpty(headStr, tailStr);
  if (!headStr && !tailStr) return `${body}\n`;
  if (!headStr) return `${body}\n\n${tailStr}`;
  if (!tailStr) return `${headStr}\n\n${body}\n`;
  return `${headStr}\n\n${body}\n\n${tailStr}`;
}

function stripIndexPreamble(head: string[]): string[] {
  for (let i = 0; i < head.length; i++) {
    const trimmed = head[i]!.trim();
    if (
      (trimmed === "# Agent Instructions" ||
        trimmed === "# Agent Operating System" ||
        trimmed === "## Index" ||
        trimmed === "## Index →") &&
      isToklessIndexPreamble(head.slice(i))
    ) {
      return head.slice(0, i);
    }
  }
  return head;
}

function isToklessIndexPreamble(lines: string[]): boolean {
  const body = lines.join("\n");
  return (
    body.includes("- **Principles**") ||
    body.includes("- **Response Style") ||
    body.includes("- **Code Index")
  );
}

// ─── Public API ──────────────────────────────────────────────

export function writeOwner(agent: string, owner: ToklessOwner): boolean {
  const path = instructionPath(agent);
  if (!path) return false;
  const cur = paths.readFile(path) ?? "";
  return writeOwnerInPath(path, cur, owner);
}

function writeOwnerInPath(filePath: string, cur: string, owner: ToklessOwner): boolean {
  const cleaned = stripLegacy(cur);
  let { head, blocks, tail } = fileParts(cleaned);
  head = stripIndexPreamble(head);

  // All owners currently present (excluding principles for list)
  const owners = ownersFromBlocks(blocks);
  // Also detect principles presence separately to keep logic
  const hasPrinciples = blocks.some((b) => b.owner === "principles");

  // If no owners and file has non-whitespace content that is not already tokless managed,
  // and not autoAppend -> in tokless would prompt. For TokSave we auto-append (preserve head)
  // The existing logic already handles head.

  // Check if owner already present
  const allOwnersIncludingPrinciples = (() => {
    const set = new Set<string>(owners);
    if (hasPrinciples) set.add("principles");
    for (const b of blocks) set.add(b.owner);
    return [...set];
  })();

  // Special: if owner is principles, and it's already covered via tokenize (principles always included when any owner present), we still ensure file has principles
  if (allOwnersIncludingPrinciples.includes(owner)) {
    // Re-render to ensure order/content up to date
    // Gather current real owners list for body rendering
    const _currentOwnersForRender = (() => {
      // Rebuild list from blocks including principles but using full set
      const list: ToklessOwner[] = [];
      // Principles is handled inside toklessAgentBody — we pass non-principles owners
      // However to check if body unchanged, compare rendered body
      for (const o of TOKLESS_OWNERS) {
        if (o === "principles") continue;
        if (
          owners.includes(o as ToklessOwner) ||
          (o as string) === (owner as string) ||
          blocks.some((b) => b.owner === o)
        ) {
          list.push(o as ToklessOwner);
        }
      }
      // If owner is non-principles and already present, list is just owners
      return owners;
    })();

    // For principles case, body is already principles included; check if file unchanged
    const wantBody = toklessAgentBody(owners as ToklessOwner[]).trimEnd();
    const currentBody = joinManagedBlocks(blocks).trimEnd();
    if (wantBody && currentBody === wantBody) {
      return false;
    }
    // If principles owner already present but body different, re-write
    if (owners.length === 0 && hasPrinciples && owner === "principles") {
      return false;
    }
  }

  if (!containsOwner(owners, owner) && owner !== "principles") {
    owners.push(owner as ToklessOwner);
  }

  // If owner is principles and there are no other owners, handle as single principles owner
  let finalOwners: ToklessOwner[];
  if (owner === "principles" && owners.length === 0 && !hasPrinciples) {
    finalOwners = ["principles"];
  } else {
    // Ensure owner included (except principles which is implicit)
    if (owner !== "principles" && !containsOwner(owners, owner)) {
      owners.push(owner as ToklessOwner);
    }
    sortOwnersByRegistry(owners);
    finalOwners = owners as ToklessOwner[];
  }

  const body = toklessAgentBody(finalOwners).trimEnd();
  const content = joinFile(head, body, tail);
  if (content === cur) return false;
  // If body empty and head/tail empty, remove file
  if (!content.trim()) {
    try {
      rmSync(filePath, { force: true });
    } catch {}
    return true;
  }
  paths.writeFile(filePath, content.endsWith("\n") ? content : `${content}\n`);
  return true;
}

export function removeOwner(agent: string, owner: string): void {
  const path = instructionPath(agent);
  if (!path) return;
  const cur = paths.readFile(path);
  if (cur === null) return;
  removeOwnerInPath(path, cur, owner as ToklessOwner);
}

function removeOwnerInPath(filePath: string, cur: string, owner: ToklessOwner): void {
  const cleaned = stripLegacy(cur);
  let { head, blocks, tail } = fileParts(cleaned);
  head = stripIndexPreamble(head);
  const owners = ownersFromBlocks(blocks);

  // Check presence
  const hasRequested =
    owners.includes(owner as ToklessOwner) ||
    (owner === "principles" && blocks.some((b) => b.owner === "principles"));

  if (!hasRequested) return;

  let kept: ToklessOwner[];
  if (owner === "principles") {
    // Removing principles when it's the only content -> remove file or keep empty
    // If other owners exist, principles stays because body always includes it when owners>0 (tokless behavior)
    // For TokSave, if we remove principles alone, we want to remove it.
    // If other owners exist, we still re-render with those owners (which will still include principles section per toklessAgentBody logic)
    // So we treat removal of principles only when it's sole owner
    if (owners.length === 0) {
      // sole principles
      const trimmedHead = stripIndexPreamble(head);
      const s = joinFile(trimmedHead, "", tail);
      if (!s.trim()) {
        try {
          rmSync(filePath, { force: true });
        } catch {}
        return;
      }
      paths.writeFile(filePath, `${s.trimEnd()}\n`);
      return;
    }
    // If there are other owners, principles cannot be removed alone (it's always included)
    // So we keep file as is? In tokless, ownersFromBlocks skips principles, so contains check for principles fails after first install if other owners present?
    // For simplicity, if removing principles while other owners exist, we keep other owners (principles will remain in body due to render)
    // So early return without change.
    return;
  } else {
    kept = owners.filter((o) => o !== owner) as ToklessOwner[];
  }

  sortOwnersByRegistry(kept as string[]);

  if (kept.length === 0) {
    // No owners left — remove managed body, keep head/tail if any, and also remove principles that might be there
    const trimmed = stripIndexPreamble(head);
    const s = joinFile(trimmed, "", tail);
    if (!s.trim()) {
      try {
        rmSync(filePath, { force: true });
      } catch {}
      return;
    }
    paths.writeFile(filePath, `${s.trimEnd()}\n`);
    return;
  }

  const body = toklessAgentBody(kept).trimEnd();
  const out = joinFile(head, body, tail);
  paths.writeFile(filePath, out.endsWith("\n") ? out : `${out}\n`);
}

export function hasOwner(agent: string, owner: string): boolean {
  const path = instructionPath(agent);
  if (!path) return false;
  const raw = paths.readFile(path);
  if (!raw) return false;
  return hasOwnerInRaw(raw, owner);
}

export function hasOwnerInRaw(raw: string, owner: string): boolean {
  const cleaned = stripLegacy(raw);
  const { blocks } = fileParts(cleaned);
  return blocks.some((b) => b.owner === owner);
}

export function ownersFromRaw(raw: string): ToklessOwner[] {
  const cleaned = stripLegacy(raw);
  const { blocks } = fileParts(cleaned);
  const set = new Set<string>();
  for (const b of blocks) set.add(b.owner);
  const list = [...set] as ToklessOwner[];
  // sort by registry order
  list.sort((a, b) => TOKLESS_OWNERS.indexOf(a) - TOKLESS_OWNERS.indexOf(b));
  return list;
}

// For test helpers
export function _internalStripLegacy(raw: string): string {
  return stripLegacy(raw);
}
export function _internalFileParts(raw: string) {
  return fileParts(raw);
}

// Legacy cleanup wrappers for old TokSave fences — used during migration
export function stripLegacyFencesFromFile(agent: string): void {
  const path = instructionPath(agent);
  if (!path) return;
  const raw = paths.readFile(path);
  if (!raw) return;
  const cleaned = stripLegacy(raw);
  if (cleaned !== raw) {
    if (!cleaned.trim()) {
      try {
        rmSync(path, { force: true });
      } catch {}
    } else {
      paths.writeFile(path, cleaned);
    }
  }
}

// Used by autoindex cleanup + separators
export { instructionPath };
