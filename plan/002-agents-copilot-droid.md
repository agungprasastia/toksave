# [Phase 2] New Agents — GitHub Copilot + Factory Droid

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Copilot CLI + VSCode IDE and Factory Droid support as agents, parity with tokless `copilot.go` and `droid.go`. Wire semua tools (rtk,caveman,codegraph,context-mode, plus future ponytail/principles) into them.

**Architecture:** Tokless copilot has dual wiring: CLI (`~/.copilot/`) dan IDE (`~/.vscode/ + {project}.vscode/` plus global vs-code copilot hooks). Droid has `~/.factory/` dir dengan hooks.json + mcp config + AGENTS.md via unified block. Port ke TS dengan reuse existing `paths.ts` + `unified-block.ts` + `json.ts/toml.ts` helpers. Copilot hooks ada flat + grouped formats. Sync IDE instructions.

**Tech Stack:** TS, Bun, Node fs, existing paths/util.

## Global Constraints

- No assumptions about Copilot CLI install — check CLI bin + config dir
- IDE wiring optional but needed for parity (VSCode copilot chat hooks)
- All writes atomic with rollback if multi-file (tokless requirement: for antagravity copilot unwire)
- Tests must use temp env overrides (HOME, USERPROFILE, etc) — never touch real config
- Reuse existing agent pattern: detect(), wire(tool,opts), unwire(tool,opts), verify(tool)

### Reference tokless files:

- `internal/agents/copilot.go` (~1200 LOC)
- `internal/agents/droid.go` (~600 LOC)
- `internal/util/paths.go`: CopilotPathsResolved(), Droid via factory dir

Key patterns tokless copilot:
```
~/.copilot/hooks/tokless-rtk.json        flat format {type,command,timeout}
~/.copilot/hooks/context-mode.json       hook entries
~/.copilot/hooks/tokless-codegraph-index.json
~/.copilot/mcp-config.json               mcp_servers
~/.copilot/copilot-instructions.md       unified block via WriteOwner("copilot")
~/.copilot/skills/caveman,ponytail       skill dirs
~/.copilot/settings.json                 (not always)

VSCode IDE variant:
- project .vscode/settings.json + .github/copilot-instructions.md
- ~/.vscode/ global? per tokless: Configures both CLI and IDE
- Functions: ConfigureCopilotMcp, ConfigureCopilotIdeMcp, SyncCopilotIdeInstructions
- CopilotIdeHooksFile, CopilotIdeMcpHas, HasCopilotIdeCodegraphIndexHook etc
- Hooks: InstallCopilotRtkHook (flat), InstallCopilotCodegraphIndexHook, Remove...

Droid patterns:
~/.factory/hooks.json                    hooks grouped format
~/.factory/mcp.json atau settings        mcp config
~/.factory/AGENTS.md                     unified block
~/.factory/skills/...                    skill dirs
~/.factory/rules/*                       optional
Functions: droidDir(), droidHooksFile(), InstallDroidRtkHook, InstallDroidCodegraphIndexHook,
           DroidMcpHas, HasDroidCodegraphIndexHook, ConfigureDroidMcp, RemoveDroidMcp etc
```

---

### Task 1: Paths — Add Copilot + Droid path resolvers

**Files:**
- Modify: `src/util/paths.ts`
- Test: `src/__tests__/paths.test.ts`
- Test: `src/__tests__/agents.test.ts` (add detect tests for new agents)

**Interfaces:**
- Consumes: existing paths module
- Produces: `copilotPaths()`, `droidPaths()`, with hooks dir, skills dir, instructions, mcp config, etc.

Port dari tokless `internal/util/paths.go` CopilotPaths + droid:

```typescript
// copilotPaths(): { dir, mcpConfig, settings, hooksDir, skillsDir, instructions }
// - dir: ~/.copilot OR COPILOT_HOME OR XDG_CONFIG_HOME/copilot
// - mcpConfig: dir/mcp-config.json
// - settings: dir/settings.json (optional)
// - hooksDir: dir/hooks
// - skillsDir: dir/skills
// - instructions: dir/copilot-instructions.md
// Copilot IDE:
// - ideInstructionsDir? Actually global ide path not fixed; tokless uses vscode global dir + project .vscode

// droidPaths(): { dir, hooksFile, mcpConfig, instructions, skillsDir, rulesDir }
// - dir: ~/.factory OR FACTORY_HOME?
// - hooksFile: dir/hooks.json
// - mcpConfig: dir/settings.json or mcp.json? Need check tokless
// - instructions: dir/AGENTS.md (factory uses AGENTS.md per tokless_block.go)
// - skillsDir: dir/skills
```

Check tokless droid.go path specifics:

From earlier scan: `droidDir() = filepath.Join(util.Home(), ".factory")`
`droidHooksFile() = filepath.Join(droidDir(), "hooks.json")`
Instruction: `filepath.Join(util.Home(), ".factory", "AGENTS.md")` per tokless_block.go

Also need VSCode IDE paths for copilot:
- `copilotIdeHooksFile(name)` in tokless: need find
- Look at tokless copilot.go: `ideRoot()`, `ideProjectRoot` var, `copilotIdeHooksFile`

For TokSave, implement minimal but correct:
- Copilot CLI paths (primary)
- Copilot IDE sync (optional, best-effort)
- Droid paths

- [ ] **Step 1: Read tokless copilot.go and droid.go path functions fully**

Use Read if needed — critical for exact file locations.

Reference from earlier partial scan:
```
copilot.go:
func copilotHooksFile(name string) → CopilotPathsResolved().HooksDir + name
ideProjectRoot global, ideRoot() → if ideProjectRoot != "" use it else CWD/.vscode? Or home?
Copilot IDE: ~/.vscode? Or project .vscode?
CopilotPathsResolved: dir, mcp-config, settings, hooks, skills, instructions as in paths.go

droid.go:
droidDir() → ~/.factory
droidHooksFile() → ~/.factory/hooks.json
Droid instructions → ~/.factory/AGENTS.md
```

- [ ] **Step 2: Write tests for new paths**

```typescript
import { copilotPaths, droidPaths } from "../util/paths.js";
describe("copilot paths", () => {
  it("respects COPILOT_HOME env", () => {
    process.env.COPILOT_HOME = "/tmp/fake-copilot";
    const p = copilotPaths();
    expect(p.dir).toContain("fake-copilot");
    delete process.env.COPILOT_HOME;
  });
  it("dir defaults to ~/.copilot", () => {
    // temp HOME override
  });
});
describe("droid paths", () => {
  it("dir is ~/.factory", () => {
    const p = droidPaths();
    expect(p.dir).toContain(".factory");
  });
});
```

- [ ] **Step 3: Implement in `src/util/paths.ts`**

Add:
```typescript
export function copilotPaths() {
  const dir = coalesce(
    env("COPILOT_HOME"),
    xdg("copilot"),
    join(home(), ".copilot")
  );
  return {
    dir,
    mcpConfig: join(dir, "mcp-config.json"),
    settings: join(dir, "settings.json"),
    hooksDir: join(dir, "hooks"),
    skillsDir: join(dir, "skills"),
    instructions: join(dir, "copilot-instructions.md"),
  };
}

// For VSCode IDE variant — reuse but with project aware
export function copilotIdePaths(projectRoot?: string) {
  const root = projectRoot ?? process.cwd();
  return {
    hooksFile: (name: string) => join(root, ".vscode", "hooks", name), // approximate? need check tokless
    // Actually tokless: ideRoot() + "/hooks"? and .github/hooks/tokless-rtk.json flat alternative
    // Simpler: implement as tokless does: .github/hooks/tokless-rtk.json + ~/.copilot/hooks
    // For parity, keep CLI hooks plus IDE dir hooks
  };
}

export function droidPaths() {
  const dir = join(home(), ".factory");
  return {
    dir,
    hooksFile: join(dir, "hooks.json"),
    mcpConfig: join(dir, "mcp.json"), // or settings.json — check real tokless usage
    settingsFile: join(dir, "settings.json"),
    instructions: join(dir, "AGENTS.md"),
    skillsDir: join(dir, "skills"),
    rulesDir: join(dir, "rules"),
  };
}
```

Need verify mcpConfig file for droid: tokless droid.go shows ConfigureDroidMcp writes to settings? Let's read full file in implementation.
For now design API flexible: handle both mcp.json and settings.json.

Also add known bin dirs for detect:
```typescript
export function copilotKnownBinDirs(): string[] { return [/* npm global bin */]; }
export function droidKnownBinDirs(): string[] { return [join(home(), ".factory","bin")]; }
```

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/paths.test.ts`

- [ ] **Step 5: Commit**

`feat: add Copilot + Droid path resolvers`

---

### Task 2: Agent module — Droid (simpler first)

**Files:**
- Create: `src/agents/droid.ts`
- Test: `src/__tests__/agents.test.ts`

**Interfaces:**
- Consumes: paths.ts droidPaths, unified-block.ts, json.ts
- Produces: detect(), wire(tool), unwire(tool), verify(tool) — with specific hooks/MCP/skill logic

Port dari tokless `internal/agents/droid.go`:

Key functions tokless droid:
- `droidDir()`, `droidHooksFile()`
- `InstallDroidRtkHook()` — writes hooks.json with PreToolUse matcher run_command? Or factory specific?
- `HasDroidRtkHook() bool`
- `InstallDroidCodegraphIndexHook()` / `HasDroidCodegraphIndexHook()`
- `ConfigureDroidMcp(toolID)` — upsert into mcp config (settings.json has mcp_servers or mcp?)
- `DroidMcpHas(toolID)` / `RemoveDroidMcp(toolID)`
- `DroidMcpPresent?`
- Skills: `codex-like` — `~/.factory/skills/caveman` dst

Per earlier scan Droid rtk hook:
```go
func rtkTestShim(agent string) { // case "droid": InstallDroidRtkHook() }
func rtkWireDroid() { InstallDroidRtkHook() } // writes ~/.factory/hooks.json routing Execute through rtk
```

So droid hooks.json format likely similar to antigravity:
```json
{
  "hooks": {
    "PreToolUse": [{"matcher":"...","hooks":[{"type":"command","command":"tokless rtk-hook droid"}]}]
  }
}
```

Need read droid.go full for exact hook shape + mcp shape.

- [ ] **Step 1: Read droid.go full via Read tool**

`Read D:\KULIAH\token\TokSave\tokless\internal\agents\droid.go` max 600 lines.

- [ ] **Step 2: Write agent module tests (simplified)**

Reuse pattern dari existing agents.test.ts — create isolated temp HOME, call detect/wire/verify/unwire.

```typescript
describe("droid agent", () => {
  it("detect returns installed when ~/.factory exists", () => { /* tmp home with .factory dir */ });
  it("wire rtk creates hooks.json with rtk-hook droid", () => {});
  it("wire codegraph creates mcp entry + index hook", () => {});
  it("wire caveman via unified block writes AGENTS.md section", () => {});
  it("verify reflects wired state", () => {});
  it("unwire removes all", () => {});
});
```

Tests need env override: override HOME to tmp dir.

- [ ] **Step 3: Implement `src/agents/droid.ts`**

Structure:

```typescript
import { existsSync } from "node:fs";
import { join } from "node:path";
import { readJsonFile, writeJsonFile, getOrCreateObject } from "../config/json.js";
import * as paths from "../util/paths.js";
import { writeOwner, removeOwner, hasOwner } from "../util/unified-block.js";

export function detect(): Detection { 
  // check factory CLI binary? tokless checks: binary "droid" or "factory"? 
  // For tokSave: existence of ~/.factory dir OR droid binary
  // Match tokless logic.
}

export async function wire(tool: ToolId, opts: RunOpts): Promise<boolean> {
  switch(tool) {
    case "rtk": installDroidRtkHook(); return true;
    case "codegraph": configureMcp("codegraph"); writeOwner("droid","codegraph"); installCodegraphIndexHook(); return true;
    case "context-mode": configureMcp("context-mode"); writeOwner("droid","context-mode"); return true;
    case "caveman": writeOwner("droid","caveman"); installSkill? ; return true;
    case "ponytail": writeOwner("droid","ponytail"); return true; // future
    case "principles": writeOwner("droid","principles"); return true;
  }
}

// private helpers:
function installDroidRtkHook() { /* write hooks.json PreToolUse */ }
function removeDroidRtkHook() { /* remove from hooks.json */ }
function hasDroidRtkHook(): boolean { /* check hooks.json contains rtk-hook droid */ }
function installCodegraphIndexHook() { /* similar */ }
function configureMcp(toolId) { /* settings.json or mcp.json */ }
function hasMcp(toolId): boolean
function removeMcp(toolId)
```

Also need `ensureDir` creation.

MCP config format for droid: need inspect tokless — likely similar to codex? Maybe uses `mcpServers` JSON. Use best guess with JSON.

Hooks format: likely:
```json
{"hooks": {"PreToolUse": [{"matcher":"Factory::Execute", "hooks":[{"type":"command","command":"toksave rtk-hook droid"}]}]}}
```

Check tokless for exact matcher string: Look at rtk hook and index hook installers.

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/agents.test.ts -t "droid"`

- [ ] **Step 5: Commit**

`feat: add Factory Droid agent support`

---

### Task 3: Agent module — Copilot CLI + IDE

**Files:**
- Create: `src/agents/copilot.ts`
- Test: `src/__tests__/agents.test.ts`
- Modify: `src/agents/index.ts` (add copilot export)

**Interfaces:**
- Consumes: paths, unified-block, json, detect util
- Produces: full copilot agent with dual CLI+IDE wiring

This is largest agent — tokless copilot.go ~1200 LOC because:
- CLI hooks (flat format) vs IDE hooks (grouped)
- MCP dual config: CLI mcp-config.json and IDE .vscode/mcp.json or project variant
- Skills both locations
- Instructions sync between CLI `copilot-instructions.md` and IDE `.github/copilot-instructions.md` or `.vscode`?
- Functions many:
  `copilotHooksFile(name)`, `SetIdeProjectRoot`, `ideRoot`, `copilotIdeHooksFile`,
  `InstallCopilotRtkHook`, `HasCopilotRtkHook`, `RemoveCopilotRtkHook`,
  `InstallCopilotCodegraphIndexHook`, `HasCopilotCodegraphIndexHook`, `Remove...`,
  `ConfigureCopilotMcp`, `CopilotMcpHas`, `RemoveCopilotMcp`,
  `ConfigureCopilotIdeMcp`, `CopilotIdeMcpHas`, `RemoveCopilotIdeMcp`,
  `SyncCopilotIdeInstructions`, etc
  `InstallCopilotIdeRtkHook`, `HasCopilotIdeRtkHook`

Plus caveman/ponytail skills: `~/.copilot/skills/caveman` plus global `.agents/skills/caveman`

Simplify for TokSave parity:

- CLI primary: `~/.copilot/`
- IDE secondary: `.github/copilot-instructions.md` sync + `.vscode/settings.json`? For now sync CLI instructions to IDE project if CWD is project?
- Hooks: Flat format JSON files in `~/.copilot/hooks/` like `tokless-rtk.json` berisi `{"type":"command","command":"toksave rtk-hook copilot",...}` plus legacy grouped format support.

Look at tokless copilot hook structure:

From earlier scan snippet:
```
flat := func() *OrderedMap {
  h := NewOrderedMap()
  h.Set("type",...)
  ... flat format {type,command,timeout} used by both Copilot CLI and VS Code.
}
... 
root := {version:1, hooks: {PreToolUse:[flat()], preToolUse:[flat()], PostToolUse:[flat()], postToolUse:[flat()]}}
```

Wait earlier snippet copilot hooks file creation had both PreToolUse and preToolUse cases (case-sensitive compat).

Actually need full read.

- [ ] **Step 1: Read copilot.go full**

`Read D:\KULIAH\token\TokSave\tokless\internal\agents\copilot.go`

- [ ] **Step 2: Write tests for copilot agent**

At least:

```typescript
describe("copilot agent", () => {
  it("detect true when ~/.copilot exists", () => {});
  it("wire rtk creates hooks/tokless-rtk.json with rtk-hook copilot", () => {});
  it("wire codegraph creates mcp-config.json entry + index hook + dual mcp", () => {});
  it("wire context-mode creates mcp entry for both CLI+IDE + unified block", () => {});
  it("wire caveman writes unified block + skill dir presence or marker", () => {});
  it("verify recovers boolean", () => {});
  it("unwire cleans up", () => {});
});
```

- [ ] **Step 3: Implement `src/agents/copilot.ts`**

Implementation outline:

```typescript
// paths
const p = paths.copilotPaths(); // dir, hooksDir, skillsDir, etc

function copilotHooksFile(name: string) {
  return join(p.hooksDir, name);
}

// RTK hook — flat format file per tokless
function rtkHookCommand(): string {
  const abs = paths.toksaveAbs();
  return `${abs} rtk-hook copilot`; // or handle spaces
}

export function installCopilotRtkHook() {
  const dir = p.hooksDir; ensureDir
  const cmd = rtkHookCommand();
  // flat format per file? tokless writes single tokless-rtk.json containing version+hooks map (both cases)
  const flat = { type: "command", command: cmd, timeout: 10 };
  const hooks = {
    PreToolUse: [flat],
    preToolUse: [flat],
    PostToolUse: [flat],
    postToolUse: [flat],
  };
  writeJsonFile(join(dir, "tokless-rtk.json"), { version: 1, hooks });
  // also .github/hooks/tokless-rtk.json? tokless has flat file alternative path?
}

export function hasCopilotRtkHook(): boolean {
  // check file exists and contains rtk-hook copilot
}

// Codegraph index hook
export function installCopilotCodegraphIndexHook() {
  // writes hooks/tokless-codegraph-index.json
  // command: toksave agy-hook? No copilot-hook codegraph-index
  // tokless: RunCodegraphIndexHook handler for copilot-hook
}
function copilotCodegraphIndexHookCommand() => `${abs} copilot-hook codegraph-index`

// MCP configure
export function configureCopilotMcp(toolId: string) {
  // write into ~/.copilot/mcp-config.json
  // structure: { mcpServers: { toolId: { type:"stdio", command: abs, args:[...] } } }
  // plus _meta?
}

export function configureCopilotIdeMcp(toolId: string) {
  // IDE variant: .vscode/mcp.json or project config
  // For simplicity in Phase 2: write into project root if CWD has .vscode, else skip? 
  // Or create global IDE config if exists.
  // Minimal parity: create {project}/.vscode/mcp.json or CWD/.vscode
}

export function syncCopilotIdeInstructions() {
  // Sync unified block from CLI copilot-instructions.md to IDE locations
  // Read CLI instructions file (via unified block path), copy owner sections to:
  // - {project}/.github/copilot-instructions.md  (if project is git repo)
  // - {project}/.vscode/?? 
  // Check tokless SyncCopilotIdeInstructions implementation
}

// Main agent API
export function detect(): Detection {
  // check copilot binary "copilot" or "gh" with copilot extension? Tokless detect: CLI present? Config dir?
  // Simplified: exists ~/.copilot OR binary copilot
}

export async function wire(tool, opts) {
  switch(tool) {
    case "rtk": installCopilotRtkHook(); installCopilotIdeRtkHook(); return true;
    case "codegraph": configureCopilotMcp("codegraph"); configureCopilotIdeMcp("codegraph"); writeOwner("copilot","codegraph"); installCopilotCodegraphIndexHook(); installCopilotIdeCodegraphIndexHook(); syncCopilotIdeInstructions(); return true;
    case "context-mode": configureCli+Ide etc; writeOwner; sync; return true;
    case "caveman": writeOwner("copilot","caveman"); sync; return true;
    case "ponytail": writeOwner; sync; return true;
    case "principles": writeOwner; sync; return true;
  }
}

export async function unwire(tool, opts) {
  switch(tool) {
    case "rtk": removeCopilotRtkHook(); removeCopilotIdeRtkHook(); return true;
    case "codegraph": removeMcp+hooks+owner; return true;
    ...
  }
}

export function verify(tool): boolean | null {
  switch(tool) {
    case "rtk": return hasCopilotRtkHook() && hasCopilotIdeRtkHook();
    case "codegraph": return CopilotMcpHas("codegraph") && HasCopilotCodegraphIndexHook() && Ide variants;
    ...
  }
}

// Export extra helpers for tools module use (mirrors tokless)
export { installCopilotRtkHook as InstallCopilotRtkHook, ... } etc
```

Also need helper functions used by caveman tool in tokless: `HasCopilotRtkHook` used inside `rtkWireCopilot` test shim. For TokSave, wire functions will call those.

Note about atomic multi-file writes: tokless requirement for antigravity was rollback if one file write fails. For copilot should follow same for wireMcp etc — but for MVP can ignore rollback, note in code as `ponytail: multi-file rollback omitted, add when copilot wiring reports partial failures`.

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/agents.test.ts -t "copilot"`

- [ ] **Step 5: Commit**

`feat: add GitHub Copilot agent support (CLI + IDE hooks + MCP + unified block)`

---

### Task 4: Update registry + CLI + init flow for new agents

**Files:**
- Modify: `src/registry.ts`
- Modify: `src/cli.ts`
- Modify: `src/commands/init.ts`
- Modify: `src/agents/index.ts`
- Modify: `src/util/paths.ts` (already)
- Test: `src/__tests__/cli.test.ts`, `src/__tests__/registry.test.ts`

**Interfaces:**
- Consumes: new agent modules copilot/droid
- Produces: ALL_AGENTS includes copilot+droid, AgentId union expanded, init command can select them.

Changes:

```typescript
// registry.ts
export type AgentId = "claude" | "opencode" | "codex" | "antigravity" | "copilot" | "droid";
export const ALL_AGENTS: AgentInfo[] = [
  ...existing,
  { id:"copilot", label:"GitHub Copilot", homepage:"https://docs.github.com/en/copilot", cliBin:"copilot" },
  { id:"droid", label:"Factory Droid", homepage:"https://factory.ai", cliBin:"droid" },
];

// import * as copilot from "./agents/copilot.js"; * as droid from "./agents/droid.js";
// const agentModules = { claude, opencode, codex, antigravity, copilot, droid };
```

CLI:
- `--agents` help list should auto-include new ids (from registry). Update description.

Init:
- detection loop already iterates ALL_AGENTS.
- No other change.

Agent index:
- export * as copilot from "./copilot.js"
- export * as droid from "./droid.js"

Tests:
- cli test: parse --agents list includes copilot,droid
- registry test: detectAgent works for new ids

- [ ] **Step 1: Write failing tests**

```typescript
it("ALL_AGENTS includes copilot and droid", () => {
  expect(ALL_AGENTS.map(a=>a.id)).toContain("copilot");
  expect(ALL_AGENTS.map(a=>a.id)).toContain("droid");
});
it("parseCli --agents copilot,droid", () => {
  const parsed = parseCli(["node","toksave","--agents","copilot,droid"]);
  expect(parsed.agents).toEqual(expect.arrayContaining(["copilot","droid"]));
});
```

- [ ] **Step 2: Implement**

- [ ] **Step 3: Run tests**

`bun test src/__tests__/registry.test.ts src/__tests__/cli.test.ts src/__tests__/agents.test.ts`

- [ ] **Step 4: Commit**

`feat: registry + CLI wired for Copilot and Droid agents`
