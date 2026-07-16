# [Phase 3] New Tools — Ponytail + Principles (and Karpathy-skills label)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two missing tools from tokless: `ponytail` (build discipline skill/plugin) and `principles` (instruction-only meta-rules). Parity dengan `internal/tools/ponytail.go` + `principles.go`. Principles is essentially karpathy-skills rebranded — instruction-only, no install, just unified block owner.

**Architecture:** Principles = instruction-only tool, install always returns true, wire = writeOwner(agent,"principles"), verify = hasOwner(...). Ponytail = npm global tool + plugin install for OpenCode (`@dietrichgebert/ponytail`) and Claude plugin marketplace (`DietrichGebert/ponytail`). Wiring untuk all agents (claude, opencode, codex, antigravity, copilot, droid) via unified block + plugin/marketplace registration. Reuse npm install util existing. For Claude: `claude plugin marketplace add DietrichGebert/ponytail && claude plugin install ponytail@ponytail`. For OpenCode: npm install -g @dietrichgebert/ponytail + add to opencode.json plugins (before context-mode if present). Verify per agent.

**Tech Stack:** TS, Bun, existing util/npm.ts, util/paths.ts, util/exec.ts

## Global Constraints

- Principles is instruction-only: Install() reports "instruction-only", no side effects.
- Ponytail install: npm -g @dietrichgebert/ponytail, version tracking via npm. Fallback if npm missing -> fail gracefully with guidance.
- Plugin order: ponytail plugin should be before context-mode in opencode.json (tokless logic insertBeforeOpencodeContextMode)
- Claude: marketplace add + plugin install, needs claude CLI
- All skills sync to central handles: Verify via plugin present / unified block owner / skill dir
- Keep content templates: ponytail section already in agent_instructions.ts template from phase 1
- No new dependencies

### Tokless reference

**ponytail.go structure:**

- const ponytailRepo = "DietrichGebert/ponytail"
- const ponytailOpencodePkg = "@dietrichgebert/ponytail"
- ponytailExec(bin,args,opts,dryHint,env...) wrapper
- registerPonytailOpencode(): adds @dietrichgebert/ponytail to opencode.json plugin[] (insertBefore context-mode)
- unregisterPonytailOpencode()
- ponytailAlreadyInOpencode(), insertBeforeOpencodeContextMode()
- writePonytailAgentsMd(ocDir) via writeOwner
- removePonytailModeState() — remove ~/.config/ponytail + XDG + APPDATA
- ponytailOpencodeInstalled(), ponytailOpencodeFilesPresent() (checks npm root -g for pkg)
- claudePonytailInstalled() — checks ~/.claude/plugins/marketplaces/ponytail OR settings.json contains ponytail OR .ponytail-active file
- codexPonytailInstalled(), antigravityPonytailInstalled(), copilotPonytailInstalled()
- claudePluginListHasPonytail() — claude plugin list contains ponytail
- ponytailWireClaude: if already installed skip unless upgrade, exec claude marketplace add + install, stamp version, WriteOwner
- ponytailUnwireClaude: claude plugin uninstall + RemoveOwner
- ponytailWireOpencode: register + WriteOwner, check installed+filesPresent unless upgrade, npm install -g, stamp version
- ponytailUnwireOpencode: unregister + rm plugins/ponytail dir + .ponytail-active + removeModeState + RemoveOwner
- Similar per agent: ponytailWireCodex, ponytailWireAntigravity, ponytailWireCopilot, ponytailWireDroid — all essentially writeOwner + skill/file presence
- Unified per tool manifest:
  var ponytail = &core.ToolManifest{
    ID:"ponytail", Label:"Ponytail", Description:"..."
    Install: npm install -g
    WireFor: map[agent] ponytailWireX
    UnwireFor: ...
    VerifyFor: ...
  }

**principles.go:**

- principlesWireFor(agent): WriteOwner(agent,"principles") + SyncCopilotIdeInstructions if copilot
- principlesUnwireFor(agent): RemoveOwner
- principlesVerifyFor(agent): HasOwner
- principles ToolManifest:
  ID:"principles" (tokless) — but tokless actually registers as "principles" not "karpathy-skills"? README says karpathy-skills but tool ID is principles.
  InstructionOnly: true, NotTrackable: true
  Install: report "instruction-only" + return true

Mapping decision for TokSave: Keep existing tool ids? Tokless uses "principles" id, TokSave currently has 4 tools: rtk,caveman,codegraph,context-mode. The README tokless table labels first tool as karpathy-skills but ID is principles. For TokSave we should add both "ponytail" and "principles" (or alias "karpathy-skills"?). Easiest: add "principles" as ID matching tokless, and keep backward compat parse "karpathy-skills" alias → principles.

Check parseToolId already handles aliases. Add "principles" + aliases "karpathy-skills","karpathy","principles", etc.
Similarly ponytail has no alias.

---

### Task 1: Principles tool (instruction-only)

**Files:**
- Create: `src/tools/principles.ts`
- Create: `src/content/principles.ts` OR use agent-instructions.ts section (principles section already there)
- Modify: `src/registry.ts` (add principles tool, handle alias karpathy-skills)
- Test: `src/__tests__/principles.test.ts` (or unify with registry test)

**Interfaces:**
- Consumes: unified-block WriteOwner/RemoveOwner/HasOwner, paths.
- Produces: Tool module with install(), installedVersion(), latestVersion(), healthCheck(), repair(), plus wire integration via agents (actually wiring now central unified, but principles tool still needs per-agent wire impl that uses unified-block? In TokSave architecture, tools don't wire directly — agents wire. So principles.ts will have no wire? Wait TokSave pattern: registry.wireTool delegates to agentModules[agent].wire(tool). So agent wire implementation must handle principles.
  However in tokless pattern: ToolManifest.WireFor[agent] = principlesWireFor(agent). So wiring is owned by tool manifest per agent.
  In TokSave pattern: agent modules have switch(tool) case. So need update agent modules to handle principles tool: writeOwner.
  So task includes updating all agent files for principles case.

- [ ] **Step 1: Write failing tests**

```typescript
// src/__tests__/principles.test.ts
import { describe, it, expect } from "bun:test";
import * as principles from "../tools/principles.js";
describe("principles tool", () => {
  it("install returns true (instruction-only)", async () => {
    const ok = await principles.install({ dryRun:false, upgrade:false, verbose:false, yes:false });
    expect(ok).toBe(true);
  });
  it("installedVersion returns null (not trackable)", () => {
    expect(principles.installedVersion()).toBeNull();
  });
  it("healthCheck returns healthy", () => {
    const h = principles.healthCheck();
    expect(h.status).toBe("healthy");
  });
});
```

- [ ] **Step 2: Run fail**

`bun test src/__tests__/principles.test.ts`

- [ ] **Step 3: Implement `src/tools/principles.ts`**

```typescript
import type { RunOpts } from "../registry.js";
import type { HealthStatus, RepairResult } from "../util/health.js";

export async function install(_opts: RunOpts): Promise<boolean> {
  // instruction-only
  return true;
}

export function installedVersion(): string | null {
  return null; // not trackable
}

export async function latestVersion(): Promise<string | null> {
  return null;
}

export function healthCheck(): HealthStatus {
  return { status: "healthy", issues: [] };
}

export async function repair(_opts: RunOpts): Promise<RepairResult> {
  return { repaired: true, status: { status: "healthy", issues: [] } };
}
```

For agent wiring, we will update agent modules in Task 3 of this phase (since all agents need handle principles). But minimally need unified-block integration: agent wire for principles = `writeOwner(agent,"principles")`.

Alternatively if agent wiring already via unified-block after phase 1, then principles case is simply writeOwner.

- [ ] **Step 4: Tests pass**

`bun test ...`

- [ ] **Step 5: Commit**

`feat: add principles (karpathy-skills) instruction-only tool`

---

### Task 2: Ponytail tool module

**Files:**
- Create: `src/tools/ponytail.ts`
- Modify: `src/content/agent-instructions.ts` (ensure ponytail section content present — already from phase 1, but verify)
- Test: `src/__tests__/ponytail.test.ts`

**Interfaces:**
- Consumes: exec util, npm util, paths, versioncache
- Produces: install (npm -g @dietrichgebert/ponytail), installedVersion via npm list, latestVersion via npm view, healthCheck.

Port from tokless ponytail.go install logic:

Tokless ponytail install not explicit as separate? Check: `install` for ponytail in tokless?

From earlier scan the file started with `const ponytailRepo...` but install method inside manifest? Not yet read full. Rest search needed for ponytail Install func.

Likely install: `npm install -g @dietrichgebert/ponytail`

Simplify:

```typescript
// src/tools/ponytail.ts

export async function install(opts: RunOpts): Promise<boolean> {
  if (opts.dryRun) return true;
  // Check already installed unless upgrade
  if (!opts.upgrade && installedVersion() !== null) return true;
  // npm install -g
  const npm = whichNpm();
  if (!npm) { warn; return false; }
  const res = await exec([npm, "install", "-g", "@dietrichgebert/ponytail@latest"]);
  return res.code === 0;
}
export function installedVersion(): string | null {
  // npm list -g @dietrichgebert/ponytail --depth=0 --json OR via npm util existing
  return getInstalledVersion("@dietrichgebert/ponytail");
}
export async function latestVersion(): Promise<string | null> {
  // npm view @dietrichgebert/ponytail version
}
export function healthCheck(): HealthStatus {
  // check installedVersion() != null OR via binary/cfg presence
}
export async function repair(opts): Promise<RepairResult> {
  // re-run install with upgrade flag
}
```

Also ponytail-specific helpers for agent verification:

- `ponytailAlreadyInOpencode(plugins)` equivalent TS: function checking plugin array
- `insertBeforeContextMode` — logic to insert before context-mode plugin
- `ponytailOpencodeInstalled`, `ponytailOpencodeFilesPresent`

But these helpers should live in agent module or shared? In tokless they live in tools/ponytail.go and called from wire functions that also update opencode config. In TokSave architecture, agent modules own config writing. So ponytail-specific config registration should be in `agents/opencode.ts` wire case for ponytail. However to avoid duplicating logic, we can have helper exported from `tools/ponytail.ts` that agent module calls: `registerPonytailOpencode()`, `unregister...`, `ponytailOpencodeInstalled()` etc.

Simplify: move helpers to tool module but agent wire imports them:

```typescript
// in ponytail.ts export:
export function registerOpencodePlugin(): void { /* read opencode.jsonc, insert @dietrichgebert/ponytail before context-mode */ }
export function unregisterOpencodePlugin(): void { ... }
export function opencodePluginInstalled(): boolean
```

Then agent opencode wire for ponytail = registerOpencodePlugin + writeOwner("opencode","ponytail")

For claude ponytail:
- Check claude CLI presence
- Run `claude plugin marketplace add DietrichGebert/ponytail` + `claude plugin install ponytail@ponytail` (via exec)
- writeOwner

Tokless also stamps version after install via `util.StampPonytailVersion`. TokSave uses versioncache. Use existing.

- [ ] **Step 1: Read ponytail.go full for install manifest**

`Read D:\KULIAH\token\TokSave\tokless\internal\tools\ponytail.go` offset after ~200 lines to find Install.

- [ ] **Step 2: Write failing tests**

```typescript
describe("ponytail tool", () => {
  it("installedVersion null when not installed", () => {
    // mock which
  });
  it("healthCheck unhealthy when not installed", () => {});
  it("registerOpencode inserts before context-mode", () => {
    const cfg = { plugin: ["context-mode"] };
    registerOpencodePlugin(cfg); // or operate on temp file
    expect(cfg.plugin).toEqual(["@dietrichgebert/ponytail","context-mode"]);
  });
});
```

- [ ] **Step 3: Implement `src/tools/ponytail.ts`**

Full implementation:

- install: npm -g @dietrichgebert/ponytail (cached)
- installedVersion: use existing npm util `getGlobalPackageVersion` or similar
- latestVersion: npm view
- healthCheck: return healthy if installedVersion() exists
- repair: install with upgrade
- opencode helpers: read opencode config via `config/json.ts`, get plugin array, insertBefore, write back.

For insertBefore logic:

```typescript
function insertBeforeContextMode(plugins: unknown[], entry: string): unknown[] {
  for (let i=0; i<plugins.length; i++) {
    const p = plugins[i];
    if (typeof p === "string" && p.toLowerCase().includes("context-mode")) {
      return [...plugins.slice(0,i), entry, ...plugins.slice(i)];
    }
  }
  return [...plugins, entry];
}
```

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/ponytail.test.ts`

- [ ] **Step 5: Commit**

`feat: add ponytail tool (install + opencode plugin helpers)`

---

### Task 3: Wire ponytail + principles into all agents

**Files:**
- Modify: `src/agents/claude.ts`
- Modify: `src/agents/opencode.ts`
- Modify: `src/agents/codex.ts`
- Modify: `src/agents/antigravity.ts`
- Modify: `src/agents/copilot.ts` (new from phase2)
- Modify: `src/agents/droid.ts` (new)
- Modify: `src/agents/index.ts`
- Test: `src/__tests__/agents.test.ts`

**Interfaces:**
- Consumes: unified-block, ponytail helpers, principles concept
- Produces: all agents handle new tools.

Per agent changes:

**claude.ts:**

Existing has wire switch for 4 tools. Add:

```typescript
case "ponytail":
  return wirePonytail(opts); // marketplace add + install + writeOwner
case "principles":
  writeOwner("claude","principles"); return true;
```

Implement `wirePonytail` helper in claude.ts that mirrors tokless:

```typescript
async function wirePonytail(opts): Promise<boolean> {
  if (opts.dryRun) return true;
  if (!opts.upgrade && isPonytailInstalledForClaude()) {
    writeOwner("claude","ponytail");
    return true;
  }
  if (!isOnPath("claude") && !opts.dryRun) { warn; return false; }
  // exec via Bun.spawn: claude plugin marketplace add DietrichGebert/ponytail
  // then claude plugin install ponytail@ponytail
  writeOwner("claude","ponytail");
  return true;
}
function isPonytailInstalledForClaude(): boolean {
  // check ~/.claude/plugins/marketplaces/ponytail exists OR settings.json contains ponytail OR .ponytail-active file
  // Use paths.claudePaths().dir / plugins / marketplaces
}
function hasPonytail(): boolean { return hasOwner("claude","ponytail"); } // or plugin check
```

Similarly unwire: `claude plugin uninstall ponytail@ponytail` + removeOwner.

**opencode.ts:**

```typescript
case "ponytail":
  if (!opts.dryRun) {
    registerPonytailOpencode(); // from tools/ponytail.ts
    writeOwner("opencode","ponytail");
  }
  return true;
case "principles":
  writeOwner("opencode","principles"); return true;
```

Unwire ponytail: unregister + rm plugins/ponytail dir + .ponytail-active + removeModeState + removeOwner.

**codex.ts, antigravity.ts, copilot.ts, droid.ts:**

All same pattern: `writeOwner(agent,"ponytail")` for wire, `removeOwner` for unwire, `hasOwner` for verify (or skill present).

Antigravity also cleanup dead ide hooks after wire.

- [ ] **Step 1: Update agents.test.ts expectations**

Add tests:

```typescript
describe("ponytail wiring", () => {
  it("claude wire ponytail creates owner", () => {});
  it("opencode wire ponytail registers plugin + owner", () => {});
  it("codex wire ponytail", () => {});
  it("antigravity wire ponytail", () => {});
});
describe("principles wiring", () => {
  it("all agents wire principles via unified owner", () => {});
});
```

- [ ] **Step 2: Implement per agent**

Edit each agent file, add cases for ponytail, principles.

Note: principles already always included when any owner exists in tokless body render logic, but explicit owner still needed for tracking. Tokless TokenizeBody skips principles when counting owners? Check: `ownersFromBlocks` skips principles when building owner list? From earlier code:

```go
func ownersFromBlocks(blocks []managedSection) []string {
  var out []string
  for _, b := range blocks {
    if b.owner == "principles" {
      continue
    }
    out = append(out, b.owner)
  }
  return out
}
```

Wait it skips principles when counting? That seems odd — probably principles is meta always present, not counted as owner? But `ToklessAgentBody` adds principles section whenever len(owners)>0 regardless of owner list containing principles. Let me re-evaluate: In tokless, principles is not stored as separate owner? Actually it skips principles in ownersFromBlocks, but adds it always. Means principles not tracked as removable owner individually? But ToolManifest for principles has WireFor that calls WriteOwner("principles"). However ownersFromBlocks ignores principles. This is conflict. Need full reading of writeOwnerInPath vs tokenize.

Wait: `ToklessAgentBody` takes owners slice which includes non-principles owners, then always adds principles section first if owners>0. So owner "principles" is never stored in blocks? But HasOwner checks for principles via sectionPresent? And `ownersFromBlocks` skips principles to avoid counting it as ownable? Hmm.

Simplify for TokSave: Keep principles as owner that can be explicitly removed, but ToklessAgentBody always includes principles when any owner exists. So hasOwner("principles") = sectionPresent ? That matches tokless behavior: principlesVerify checks HasOwner(agent,"principles") which checks body contains principles marker. So if any owner present, principles marker present automatically, so HasOwner("principles") returns true whenever file has any tokless body. That's intentional: principles is always present as long as tokless is wired.

For TokSave to match, implement same: tokenize includes principles detection? But ownersFromBlocks skips? Better to implement same as tokless: principles section is always rendered when any owner present, not tracked as separate removable item for body rendering, but WriteOwner/RemoveOwner for principles handled via whether to treat as meta?

Simplified approach for phase 3: 
- principles wire = ensure at least one owner present? No, we want principles independent also.
- Easiest parity: treat principles like caveman — explicit owner in list, body includes it. But keep existing TokSave templating idempotent: if principles owner removed, principles section should disappear only if no other owners.

Tokless actual logic: `ownersFromBlocks` filters out principles because principles section is not considered an owner choice — it's always present if any tool is wired. So you cannot have only principles? Wait you can call WriteOwner(agent,"principles") — what happens? owners = ownersFromBlocks(blocks) → excludes principles. Then containsOwner check for "principles" will fail? Then appends principles, sorts, renders body with principles only? Let's trace:
- cur body empty, ownersFromBlocks = [], containsOwner([], "principles") false, appends -> owners = ["principles"], sort, ToklessAgentBody(["principles"]) -> len>=2? no (len=1) so no index section, but hasOwner principles check? Actually ToklessAgentBody checks len>0 adds instructionSection("principles"). So body = principles section. So it works: principles as sole owner results in file containing only principles section (no index).
- Now if you later add caveman: ownersFromBlocks on file that only has principles? Since ownersFromBlocks skips principles, owners = [], but file still has principles heading. Then writeOwner for caveman: head = non-managed before, owners = [], containsOwner([...], "caveman") false, owners=["caveman"], body = ToklessAgentBody(["caveman"]) which includes principles + caveman + index because now len>=2? Wait len 1, index not. But previous file had principles. Joined file would replace with principles+caveman. That's upgrade from single principles to 2 tools with index? Actually tokless index section appears when len owners >=2. If we have ["caveman"] only, len 1, index not. But file currently principles only. So after adding caveman, body would be principles + caveman (since hasOwner("caveman") true). That matches.

- If we later remove caveman leaving only principles file: removeOwnerInPath for "caveman": ownersFromBlocks returns [] (since principles skipped), containsOwner([],"caveman") false -> early return, file not deleted. But principles content remains? That's maybe intentional? Hmm.

This complexity suggests implementation must exactly port ownersFromBlocks skipping principles.

Simpler for TokSave: we will port exact tokless logic for owners handling including principles skip, to achieve correct behavior.

So implement helper `ownersFromBlocks` that skips principles for listing, but body render includes principles whenever len(owners)>=1? Keep tokless exact.

Document in code comment.

- [ ] **Step 3: Update verify per agent**

For principles: `hasOwner(agent,"principles")`
For ponytail: `hasOwner(agent,"ponytail")` or plugin present.

- [ ] **Step 4: Run tests**

`bun test src/__tests__/agents.test.ts`

Fix failures.

- [ ] **Step 5: Commit**

`feat: wire ponytail + principles into all agents`

---

### Task 4: Registry updates for new tools

**Files:**
- Modify: `src/registry.ts`
- Modify: `src/cli.ts` (help shows ponytail + principles)
- Test: `src/__tests__/registry.test.ts`, `src/__tests__/cli.test.ts`

**Interfaces:**
- Consumes: new tool modules
- Produces: ALL_TOOLS includes ponytail+principles, ToolId expanded, parseToolId handles aliases.

Implementation:

```typescript
export type ToolId = "rtk" | "caveman" | "codegraph" | "context-mode" | "ponytail" | "principles";

export const ALL_TOOLS: ToolInfo[] = [
  ...existing plus:
  { id:"ponytail", label:"Ponytail", homepage:"https://github.com/DietrichGebert/ponytail", channel:"npm", minNodeMajor:0 },
  { id:"principles", label:"Principles", homepage:"https://github.com/multica-ai/andrej-karpathy-skills", channel:"skill" (or GitHub), minNodeMajor:0 },
];

export function parseToolId(s: string): ToolId | null {
  const map: Record<string,ToolId> = {
    rtk:"rtk",
    caveman:"caveman",
    codegraph:"codegraph",
    "context-mode":"context-mode",
    contextmode:"context-mode",
    ponytail:"ponytail",
    principles:"principles",
    "karpathy-skills":"principles",
    karpathy:"principles",
    "karpathy-skills":"principles",
  };
  return map[lower] ?? null;
}

// dispatch tables:
const toolModules = {
  rtk: rtkTool,
  caveman: cavemanTool,
  codegraph: codegraphTool,
  "context-mode": contextModeTool,
  ponytail: ponytailTool,
  principles: principlesTool,
};
```

Also update `toolHealthCheck` and `toolRepair` to work for new tools.

Update CLI help description listing tools.

- [ ] **Step 1: Write failing test**

```typescript
it("ALL_TOOLS includes ponytail and principles", () => {
  expect(ALL_TOOLS.map(t=>t.id)).toEqual(expect.arrayContaining(["ponytail","principles"]));
});
it("parseToolId karpathy-skills alias", () => {
  expect(parseToolId("karpathy-skills")).toBe("principles");
});
```

- [ ] **Step 2: Implement registry changes**

- [ ] **Step 3: Tests pass**

`bun test src/__tests__/registry.test.ts src/__tests__/cli.test.ts`

- [ ] **Step 4: Commit**

`feat: registry now includes ponytail + principles (alias karpathy-skills)`

---

### Task 5: Manifest + versioncache handling for new tools

**Files:**
- Modify: `src/util/manifest.ts` (if it tracks versions per tool)
- Modify: `src/util/versioncache.ts` (if needs entries)
- Test: `src/__tests__/manifest.test.ts`

Ponytail: trackable via npm version (unlike principles which NotTrackable).
Principles: NotTrackable + InstructionOnly — no version.

Tokless has `NotTrackable` and `InstructionOnly` flags but TokSave registry doesn't have those yet. Should add flags to ToolInfo to skip version display?

Options:
- Add `notTrackable?: boolean` and `instructionOnly?: boolean` to ToolInfo
- In version table display skip those? Or show "installed" not version.

In Tokless toolVersionDisplayLine:
- if InstructionOnly => return ""
- if NotTrackable && installed != nil => check installed
- if NotTrackable && present => "installed"

For TokSave: add same handling.

- [ ] **Step 1: Test manifest records ponytail**

`bun test src/__tests__/manifest.test.ts`

- [ ] **Step 2: Implement flags**

```typescript
export interface ToolInfo {
  ...
  notTrackable?: boolean;
  instructionOnly?: boolean;
}

// ponytail: notTrackable false
// principles: notTrackable true, instructionOnly true
```

Update version table printing in `init.ts` and `doctor.ts` to skip instructionOnly maybe.

- [ ] **Step 3: Commit**

`feat: version tracking flags for instruction-only tools`
