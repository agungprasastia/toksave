# [Phase 1] Foundation — Unified Block System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port tokless unified block system (tokless_block.go + instructions.go + separators.go) ke TokSave, supaya multiple tools share 1 file AGENTS.md/CLAUDE.md via owner list.

**Architecture:** Tokless punya single template `agent_instructions.md` yang di-split per section owner via marker headings (`## Principles`, `## Response Style (caveman)` etc). Body dirender dari ordered owner list. Legacy HTML fences (`<!-- CAVEMAN_START -->` etc) di-strip saat migrasi. New API: `writeOwner(agent, owner)`, `removeOwner(agent, owner)`, `hasOwner(agent, owner)`, `ensureSeparators(agentIds)`.

**Tech Stack:** TypeScript, Bun, existing TokSave config/json parsers.

## Global Constraints

- Language: TypeScript + Bun, ESM, strict. No new dependencies tanpa alasan kuat.
- Style: 2 spaces, double quotes, semicolons, trailing commas.
- Agent instruction files under user config dirs — tests must override HOME paths.
- Compatibility: support removing legacy fences dari TokSave current version.
- ToklessOwners order: `["principles","caveman","ponytail","codegraph","context-mode"]` — must match tokless.

---

### Task 1: Create unified agent instructions template

**Files:**
- Create: `src/content/agent-instructions.ts`
- Create: `src/content/sections.ts`

**Interfaces:**
- Consumes: existing content (caveman-skill, ctx-rules, rtk-rules) — will be refactored into template later.
- Produces: `TOKLESS_OWNERS`, `SECTIONS_BY_OWNER`, `TOKLESS_AGENT_BODY(owners)`, `TOKENIZE_BODY(body)`

Tokless source port:
- `internal/util/agent_instructions.md` → full markdown content (principles + caveman + ponytail + codegraph + context-mode)
- `internal/util/instructions.go`: `ToklessOwners`, `SectionsByOwner`, `SectionMarkers`, `ToklessAgentBody`, `TokenizeBody`, `instructionIndexSection`, `instructionSection`
- For now use placeholder content that matches official docs (rtk rules existing can merge)

Kunci: Caveman & ponytail content sudah ada di tokless, ambil dari `internal/util/agent_instructions.md` yang sudah di-read sebelumnya. Port verbatim.

**Important:** Untuk phase 1, ponytail + principles sections bisa placeholder dulu, tapi structure harus ada.

- [ ] **Step 1: Read tokless agent_instructions.md full**

File sudah known dari earlier scan. Content needed:

```
# Agent Instructions
- Principles
- Response Style (caveman)
- Build Discipline (ponytail)
- Code Index (codegraph)
- Context Tools (context-mode)

## Principles
[4 principles from ANDREJ karpathy — think, simplicity, surgical, goal-driven]

## Response Style (caveman)
[terse caveman prose]

## Build Discipline (ponytail)
[lazy senior dev]

## Code Index (codegraph)
[codegraph_explore usage]

## Context Tools (context-mode)
[ctx_ sandbox tools]
```

Reference final yang sudah ada di memory dari tokless scan — use that.

- [ ] **Step 2: Write failing test for ToklessOwners order**

```typescript
// src/__tests__/unified-block.test.ts
import { describe, it, expect } from "bun:test";
import { TOKLESS_OWNERS } from "../content/agent-instructions.js";

describe("unified block foundation", () => {
  it("owners in registry order", () => {
    expect(TOKLESS_OWNERS).toEqual(["principles","caveman","ponytail","codegraph","context-mode"]);
  });
});
```

- [ ] **Step 3: Run test confirm fail (not defined)**

Run: `bun test src/__tests__/unified-block.test.ts -t "owners"`

- [ ] **Step 4: Implement `src/content/agent-instructions.ts`**

```typescript
export const TOKLESS_OWNERS = ["principles","caveman","ponytail","codegraph","context-mode"] as const;
export type ToklessOwner = typeof TOKLESS_OWNERS[number];

export const SECTIONS_BY_OWNER: Record<ToklessOwner,string> = {
  principles: "## Principles",
  caveman: "## Response Style (caveman)",
  ponytail: "## Build Discipline (ponytail)",
  codegraph: "## Code Index (codegraph)",
  "context-mode": "## Context Tools (context-mode)",
};

// Legacy markers that should also match as same owner
export const LEGACY_SECTIONS: Record<ToklessOwner,string[]> = {
  principles: ["## 1. Principles","## Principles (craft) →","## Principles (craft)"],
  caveman: ["## 2. Response Style","## Response Style","## Style","## Caveman Style","## Caveman","## Voice (caveman)","## Response Style (caveman)"],
  ponytail: ["## 3. Build Discipline","## Build Discipline","## Build Less","## Ponytail","## Ponytail: Build Less","## Reuse Ladder (ponytail)","## Lazy Ladder (ponytail)","## Build Discipline (ponytail)"],
  codegraph: ["## 4. Code Search","## Codegraph","## Codegraph — MUST USE FOR CODE","## Code Index (codegraph)"],
  "context-mode": ["## 5. Context Control","## Context Tools","## Context Tools — MUST USE FOR DATA","## Context Tools (context-mode)"],
};

export function sectionMarkers(owner: ToklessOwner): string[] {
  const primary = SECTIONS_BY_OWNER[owner];
  const legacy = LEGACY_SECTIONS[owner] ?? [];
  return [primary, ...legacy];
}

export function sectionPresent(body: string, owner: ToklessOwner): boolean {
  return sectionMarkers(owner).some(m => body.includes(m));
}

function hasOwner(owners: ToklessOwner[], want: string): boolean {
  return owners.includes(want as ToklessOwner);
}

// Full template — port dari tokless internal/util/agent_instructions.md
const AGENT_INSTRUCTIONS_TEMPLATE = `... actual content dari tokless ...`;

function instructionIndexSection(): string {
  const body = AGENT_INSTRUCTIONS_TEMPLATE.trimEnd();
  const idx = body.indexOf("\n## ");
  if (idx < 0) return body;
  return body.slice(0, idx);
}

function instructionSection(owner: ToklessOwner): string {
  const marker = SECTIONS_BY_OWNER[owner];
  if (!marker) return "";
  const body = AGENT_INSTRUCTIONS_TEMPLATE.trimEnd();
  let start = body.indexOf(marker);
  if (start < 0) return "";
  if (start > 0) {
    const lastNl = body.lastIndexOf("\n", start - 1);
    start = lastNl + 1;
  }
  const rest = body.slice(start);
  const nextIdx = rest.slice(1).indexOf("\n## ");
  if (nextIdx >= 0) {
    return rest.slice(0, nextIdx + 1).trimEnd();
  }
  return rest.trimEnd();
}

export function toklessAgentBody(owners: ToklessOwner[]): string {
  let b = "";
  if (owners.length >= 2) {
    b += instructionIndexSection() + "\n\n";
  }
  if (owners.length > 0) {
    b += instructionSection("principles") + "\n\n";
  }
  if (hasOwner(owners, "caveman")) b += instructionSection("caveman") + "\n\n";
  if (hasOwner(owners, "ponytail")) b += instructionSection("ponytail") + "\n\n";
  if (hasOwner(owners, "codegraph")) b += instructionSection("codegraph") + "\n\n";
  if (hasOwner(owners, "context-mode")) b += instructionSection("context-mode") + "\n\n";
  return b.trimEnd();
}

export function tokenizeBody(body: string): ToklessOwner[] {
  const out: ToklessOwner[] = [];
  for (const owner of TOKLESS_OWNERS) {
    if (sectionPresent(body, owner)) out.push(owner as ToklessOwner);
  }
  return out;
}
```

Copy actual template content dari scan earlier tokless (internal/util/agent_instructions.md).

- [ ] **Step 5: Run test pass**

`bun test src/__tests__/unified-block.test.ts`

- [ ] **Step 6: Commit**

`git add ... && git commit -m "feat: add unified agent instructions template (TOKLESS_OWNERS + body render)"`

---

### Task 2: Implement unified block file manager (WriteOwner/RemoveOwner/HasOwner)

**Files:**
- Create: `src/util/unified-block.ts`
- Test: `src/__tests__/unified-block.test.ts` (expand)

**Interfaces:**
- Consumes: `agent-instructions.ts` (TOKLESS_OWNERS, SECTIONS_BY_OWNER, toklessAgentBody, tokenizeBody, sectionMarkers, LEGACY_SECTIONS)
- Produces: `writeOwner(agent, owner)`, `removeOwner(agent, owner)`, `hasOwner(agent, owner)`, `hasOwnerInRaw(raw, owner)`, `ownersFromRaw(raw)`

Port dari `internal/tools/tokless_block.go`:
- `instructionPath(agent)` → map agent id ke file path (use `paths.ts`)
- `stripLegacy(raw)` — remove legacy fences + legacy block headings
- `fileParts(raw)` — split head/managed/tail
- `blocksFromLines`, `ownerOf(line)`, `ownersFromBlocks`
- `writeOwnerInPath`, `removeOwnerInPath`, `joinFile`, `joinEmpty`, `sortOwnersByRegistry`
- `stripIndexPreamble`
- `instructionConflict` handling — keep simple for phase 1: auto-append (no prompt), but preserve head/tail

Legacy fences list (tokless):
```go
var legacyFences = [][2]string{
  {"<!-- caveman-begin -->", "<!-- caveman-end -->"},
  {"<!-- CODEGRAPH_START -->", "<!-- CODEGRAPH_END -->"},
  {"<!-- CONTEXT-MODE_START -->", "<!-- CONTEXT-MODE_END -->"},
  {"<!-- tokless:owners=", ""},
}
var legacyBlockHeadings = []string{"## Process Noise"}
```

TokSave legacy tambahan: `CAVEMAN_START`, `RTK_START`, `CONTEXT-MODE_START`, `CODEGRAPH_START` — include juga.

For TokSave TS version:
- Agent path mapping via existing `paths.ts` (claudePaths, opencodePaths, codexPaths, antigravityPaths + new copilot+droid for future compat)
- Use `fs` existence checks.

- [ ] **Step 1: Write failing tests**

```typescript
it("writeOwner creates file with single owner section", () => {
  const tmp = tempDir();
  // mock paths to tmp, call writeOwner("claude","caveman"), verify file contains caveman marker
});
it("writeOwner adds second owner preserving order", () => {
  // write caveman then codegraph, verify both present and order matches TOKLESS_OWNERS
});
it("removeOwner removes owner leaving other", () => {
  // write caveman+codegraph, remove caveman, verify only codegraph remains
});
it("stripLegacy removes old caveman fences", () => {
  // raw with <!-- CAVEMAN_START --> should be stripped
});
```

Use existing pattern from `agents.test.ts`: temp HOME override.

- [ ] **Step 2: Run fail**

`bun test src/__tests__/unified-block.test.ts`

- [ ] **Step 3: Implement `src/util/unified-block.ts`**

Port logic tokless_block.go dengan TS idioms:

```typescript
import { readFile, writeFile, ensureDir, exists } etc from paths.ts
import { TOKLESS_OWNERS, SECTIONS_BY_OWNER, toklessAgentBody, sectionMarkers, LEGACY_SECTIONS } from "../content/agent-instructions.js"
import * as paths from "./paths.js"

function instructionPath(agent: string): string {
  switch(agent){
    case "claude": return paths.claudePaths().agentsMd;
    case "opencode": return paths.opencodePaths().agentsMd;
    case "codex": return paths.codexPaths().instructions;
    case "antigravity": return paths.antigravityPaths().instructions;
    case "copilot": return paths.copilotPaths().instructions; // future
    case "droid": return join(paths.home(), ".factory", "AGENTS.md");
    default: return "";
  }
}

// legacy fences
const LEGACY_FENCES: [string,string][] = [
  ["<!-- caveman-begin -->","<!-- caveman-end -->"],
  ["<!-- CODEGRAPH_START -->","<!-- CODEGRAPH_END -->"],
  ["<!-- CONTEXT-MODE_START -->","<!-- CONTEXT-MODE_END -->"],
  ["<!-- CAVEMAN_START",""], // TokSave current format partial
  ["<!-- RTK_START",""],
  ["<!-- tokless:owners=",""],
];
const LEGACY_HEADINGS = ["## Process Noise"];

function stripLegacy(raw: string): string { /* port */ }
function stripLegacyHeading(raw: string, heading: string): string { /* port */ }
function fileParts(raw: string): {head:string[], blocks:ManagedSection[], tail:string[]} { /* port */ }
function ownerOf(line: string): ToklessOwner | "" { /* check SectionsByOwner + legacy */ }
... rest ports
```

Detail: `joinFile` logic persis tokless.
Order sorting by `TOKLESS_OWNERS` index.

- [ ] **Step 4: Test pass**

`bun test src/__tests__/unified-block.test.ts`

Target coverage: write, remove, has, legacy cleanup, order, empty file removal, head preservation.

- [ ] **Step 5: Commit**

`feat: implement unified block manager (WriteOwner/RemoveOwner/HasOwner)`

---

### Task 3: Separator normalization

**Files:**
- Create: `src/util/separators.ts`
- Test: `src/__tests__/separators.test.ts`

**Interfaces:**
- Consumes: path resolution
- Produces: `ensureInstructionSeparators(agentIds: string[])`

Port dari `internal/tools/separators.go`:

```go
endMarkerRe = (?i)<!-- [^>]+[_-]end(?:[^>]*)? -->
startMarkerRe = (?i)<!-- [^>]+[_-](?:start|begin)(?:[^>]*)? -->
```

Normalized: antara `<!-- ...end -->` dan `<!-- ...start/begin -->` harus exactly `\n\n` (2 newlines). Jika bukan 2, fix to `\n\n`.

Note: Setelah migrasi ke unified block, HTML fences udah gak ada, tapi function ini tetap perlu untuk edge case atau legacy files. Keep it.

TokSave variant: check all instruction files for given agents, normalize.

- [ ] **Step 1: Write failing test**

```typescript
it("normalizes single newline between blocks to double", () => {
  // write file with "<!-- A_END -->\n<!-- B_START -->" -> expect "\n\n" after fix
});
```

- [ ] **Step 2: Implement**

```typescript
const END_MARKER_RE = /<!-- [^>]+[_-]end(?:[^>]*)? -->/gi;
const START_MARKER_RE = /<!-- [^>]+[_-](?:start|begin)(?:[^>]*)? -->/gi;

function normalizeSeparators(filePath: string) {
  const raw = readFile(filePath);
  if (!raw) return;
  // find all end markers, check distance to next start, fix if not 2 newlines
  // port countNewlines, ensure \n\n
}
export function ensureInstructionSeparators(agentIds: string[]) { /* loop */ }
```

- [ ] **Step 3: Test pass**

`bun test src/__tests__/separators.test.ts`

- [ ] **Step 4: Commit**

`feat: add instruction separator normalizer`

---

### Task 4: Refactor agent wiring to use unified block

**Files:**
- Modify: `src/agents/claude.ts`
- Modify: `src/agents/opencode.ts`
- Modify: `src/agents/codex.ts`
- Modify: `src/agents/antigravity.ts`
- Modify: `src/content/ctx-rules.ts`
- Modify: `src/content/rtk-rules.ts`
- Modify: `src/content/caveman-skill.ts` (or create wrapper)
- Test: `src/__tests__/agents.test.ts` (update expectations)

**Interfaces:**
- Consumes: unified-block.ts WriteOwner/RemoveOwner/HasOwner
- Produces: all agents wire cve/ctx/rtk via unified system

Perubahan:

Current tokSave:
- caveman: write SKILL.md directly + inject AGENTS.md via `CAVEMAN_START` fence
- ctx-rules: inject `CONTEXT-MODE_START` fence
- rtk-rules: inject `RTK_START` fence
- Each independent block

Target (tokless parity):
- All instruction wiring via `writeOwner(agent, owner)` / `removeOwner`
- `caveman` owner = Caveman response style section
- `context-mode` owner = Context Tools section
- `codegraph` owner = Code Index section  
- `principles` owner implicit always when any owner exists? In tokless: principles always included when len(owners)>0
- `rtk` owner? Di tokless, rtk tidak nambah section — rtk hanya hook/plugin. Cek: tokless `SECTIONS_BY_OWNER` gak ada rtk. Jadi RTK instructions tetap harus di-handle — tapi di tokless RTK instructions = bagian dari prosedur? Actually cek tokless: RTK.md dihapus di wiring, rules via AGENTS? Wait existing code: tokless writes owner for everything except rtk. So rtk tidak punya owner. Artinya untuk TokSave, rtk rules mau migrate ke unified atau stay? Saran: migrate rtk rules juga sebagai part of codegraph? No — rtk doesn't have section in template. Jadi rtk rules file (`rtk-rules.ts`) tetap bisa di-handle terpisah, atau jadi part of unified template jika diinginkan. Simplest: keep rtk as-is atau consider menghapus rtk block dan rely pada unified template yang sudah mention rtk di codegraph/context sections? Actually di tokless template tidak mention RTK. Di tokSave current rtk-rules.ts masih ada. Decision: untuk phase 1, keep rtk-rules as part of old system, but ensure no conflict dengan unified. Longer term, rtk section bisa ditambahkan sebagai owner baru atau di-merge.

Better: Untuk Phase 1, fokus migrate caveman, context-mode, codegraph ke unified. RTK tetap via existing fence (atau future migrate). Principles nanti.

Implementation untuk tiap agent:

```typescript
// Before (claude.ts wire caveman):
// write SKILL.md + inject AGENTS.md fence
// After:
// wire via writeOwner("claude","caveman") + still write SKILL.md for Claude skill convention
```

Caveman skill for Claude: tetap butuh SKILL.md file per Claude convention, tapi AGENTS.md block via unified.

Untuk opencode/codex/antigravity: caveman via unified `writeOwner(agent,"caveman")`.

Context-mode: `writeOwner(agent,"context-mode")` instead of fence inject.

Codegraph: `writeOwner(agent,"codegraph")`.

- [ ] **Step 1: Update tests to expect unified markers**

File `src/__tests__/agents.test.ts` saat ini expect `CAVEMAN_START` etc. Update untuk expect unified markers like `## Response Style (caveman)`.

Important: Discovery phase — agent.test verifies wiring creates correct files. Update assertions.

- [ ] **Step 2: Implement refactored wire/unwire/verify for each agent**

Pseudocode claude.ts new wire:

```typescript
import { writeOwner, removeOwner, hasOwner } from "../util/unified-block.js"

async function wireCaveman(opts) {
  // still write skill file
  // plus:
  writeOwner("claude","caveman");
}

function removeCaveman() {
  removeOwner("claude","caveman");
  // rm skill dir
}

function hasCavemanSkill() {
  // return hasOwner("claude","caveman") || exists SKILL.md
}
```

Similarly codegraph, context-mode.

- [ ] **Step 3: Run all agent tests**

`bun test src/__tests__/agents.test.ts`

Karena ini breaking change, banyak tests fail initially. Fix iteratively.

- [ ] **Step 4: Ensure existing tests for other modules still pass**

`bun test` — semua.

- [ ] **Step 5: Commit**

`refactor: migrate instruction wiring to unified block system (WriteOwner/RemoveOwner)`

---

### Task 5: Manifest/head preservation + empty file cleanup

**Files:**
- Modify: `src/util/unified-block.ts` (extra edge cases)

Edge cases dari tokless:
- `stripIndexPreamble` — kalau last owner dihapus, hapus preamble "# Agent Instructions / ## Index" jika itu milik tokless
- `joinFile` trimming logic — single blank line between regions
- `isToklessIndexPreamble` detection
- Kalau file jadi kosong setelah remove last owner → `os.Remove(path)`
- Kalau cleaned jadi whitespace only → remove file
- Preserve user content before first owner heading (head) and after managed body (tail)

- [ ] **Step 1: Write edge case tests**

```typescript
it("preserves user content before managed block", () => {
  // existing file "# My Project\n\nSome notes\n\n## Principles\n..." → write new owner, head preserved
});
it("removes file when last owner removed and no user content", () => {
  // write only tokless block, remove owner, file should be deleted
});
it("keeps user content when last owner removed", () => {
  // file has user + tokless block, remove all owners, user content remains
});
it("joinFile produces exactly one blank line between regions", () => {
  // verify no double blank lines
});
```

- [ ] **Step 2: Fix implementation per tests**

- [ ] **Step 3: Commit**

`fix: unified block edge cases (head/tail preserve, empty cleanup, index preamble)`
