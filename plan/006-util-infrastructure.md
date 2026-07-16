# [Phase 6] Util Infrastructure Parity

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port remaining util infrastructure from tokless to TokSave: progress tree UI, colors extended, deps ensure (Node + Git checks with upgrade prompt), paths completeness (WindowsHomeFromWSL, selfHealPath, binary resolvers), npm install robust (retry strategies, fallback tarball, timeout, old node handling), MCP spawn (PickMcpSpawn, WrapAutoIndex, McpSpawn struct, binary healthy checks), version gathering (GatherVersions, GatherVersionsForce, SemverCompare/Gte), logger structured (L), prompt multiSelect with hints/disabled, etc.

**Architecture:** Tokless util package is large (~30 files). Many helpers are small but needed for parity: `preflight.go` (git detection), `versions.go` (gather), `npminstall.go` (global install with retries), `mcpspawn.go`, `paths.go`, `pathsetup.go`, `colors.go`, `progress.go`, `prompt.go`, `exec.go`, `jsonc.go`, `toml.go`, `isatty_*.go`, `rawmode_*.go`, `suspend_*.go`, `process_alive_*.go`, `spawn_bg_*.go`, etc. For TokSave, we only need TS equivalents for cross-platform parity of features actually used in commands/tools, not all OS-specific rawmode.

**Tech Stack:** TS, Bun, Node fs, existing utils.

## Global Constraints

- Don't over-port: only what is needed for commands/tools parity. Skip Unix rawmode, suspend, process_alive if not needed in Node ecosystem (but keep conceptual).
- Keep existing TokSave util APIs backward compatible — extend, not replace.
- Tests must not require real npm/git/node installs — mock via spyOn which/path checks.
- Progress UI: should remain compatible with existing Progress class but add tree variants if needed.

---

### Task 1: Progress & Colors extended

**Files:**
- Modify: `src/util/progress.ts`
- Modify: `src/util/colors.ts`
- Test: `src/__tests__/progress.test.ts` (maybe new)

**Interfaces:**
- Consumes: picocolors
- Produces: Tree progress UI, spinner frames, sym constants, rule line, erase line.

Tokless progress:

- `NewRootSectionProgress(label)`, `NewSectionProgress(label)`, `NewProgress("")`
- Methods: `Start(n)`, `Begin(label)`, `Step(phase, frac)`, `Complete(note)`, `Fail(reason)`, `Done(msg)`
- Tree rendering: uses symbols `Sym.Check`, `Sym.Warn`, `Sym.Bullet`, box drawing tree `TreeLeaf`, `TreeCorner`, `TreeCornerStyled`, `TreeFooter`
- Spinner `runStatus(label,fn)` with frames `["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]` ticker 80ms
- Methods like `EraseStyledLine(label)` erases previously printed status line for update

TokSave existing Progress: simple single line.

Upgrade:

Add methods to Progress to support section mode? Or create new classes `SectionProgress` and `RootSectionProgress` wrapping existing.

Simplify: Keep existing Progress for simple cases, add new functions for tree:

```typescript
export function TreeLeaf(msg: string): void
export function TreeCorner(msg: string): void
export function TreeCornerStyled(msg: string): void
export function TreeFooter(width: number): void
export function Rule(width: number): string // line of ─
export function EraseStyledLine(s: string): string // ansi erase code + spaces
```

And SectionProgress:

```typescript
export class SectionProgress {
  label: string;
  Start(n: number)
  Begin(label: string)
  Step(phase: string, frac: number) // updates same line with phase
  Complete(note?: string)
  Fail(reason: string)
  Done(msg: string)
}
```

Implementation: track current index, total, use stdout write with `\r`? For simplicity can just use console.log with prefixes for non-TTY, and fancy for TTY.

Colors:

- Extend `colors.ts` to export `C` equivalent? Tokless `util.C.*`: Bold, Cyan, Gray, Yellow, Green, Red, Dim etc via picocolors. Already have via picocolors but need wrapper.
- Add `Sym`: Check="✔", Warn="⚠" or "!", Bullet="•" etc
- Add `VisibleLen(s)` strips ANSI for length
- Add `StdoutANSI()`, `StdoutIsTTY()`, `StdoutIsTTY` check
- Add `IsInteractive()` (already in prompt.ts) but also colors util

Check tokless `colors.go` and `colors_test.go` for exact symbols.

- [ ] **Step 1: Read tokless util/colors.go and util/progress.go**

`Read D:\KULIAH\token\TokSave\tokless\internal\util\colors.go` + `progress.go`

- [ ] **Step 2: Write tests for new progress**

```typescript
describe("SectionProgress", () => {
  it("Start and Complete without error", () => {
    const p = new SectionProgress("Tools");
    p.Start(2);
    p.Begin("RTK");
    p.Complete("v1.0.0");
    p.Done("");
  });
});
describe("Tree helpers", () => {
  it("TreeLeaf prints", () => {});
});
```

- [ ] **Step 3: Implement**

Update `src/util/progress.ts`:

```typescript
export const Sym = {
  Check: "✔",
  Warn: "⚠",
  Bullet: "•",
};

export function TreeLeaf(msg: string): void { console.log(`  ${msg}`); } // placeholder
export function TreeCorner(msg: string): void { console.log(`  └─ ${msg}`); }
export function TreeCornerStyled(msg: string): void { TreeCorner(msg); }
export function TreeFooter(width: number): void { console.log("  " + "─".repeat(width)); }
export function Rule(width: number): string { return "─".repeat(width); }
export function VisibleLen(s: string): number { return stripAnsi(s).length; }
export function EraseStyledLine(s: string): string { return "\r\x1b[2K"; } // vt100 erase
export function StdoutIsTTY(): boolean { return process.stdout.isTTY ?? false; }
export function StdoutANSI(): boolean { return process.stdout.isTTY && !process.env.CI; }

export class SectionProgress { /* ... */ }
export class RootSectionProgress extends SectionProgress {}
```

Update `colors.ts` to export `C` object mimicking tokless:

```typescript
import pc from "picocolors";
export const C = {
  Bold: pc.bold,
  Cyan: pc.cyan,
  Gray: pc.gray,
  Yellow: pc.yellow,
  Green: pc.green,
  Red: pc.red,
  Dim: pc.dim,
};
export const CHECK = Sym.Check; etc
```

But keep existing named exports for backward compat.

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/progress.test.ts` (if new)

- [ ] **Step 5: Commit**

`feat: extended progress tree UI + color symbols parity`

---

### Task 2: Binary resolvers & path setup

**Files:**
- Modify: `src/util/paths.ts`
- Modify: `src/util/pathfix.ts`
- Create: `src/util/binary.ts` (or extend paths.ts)
- Test: `src/__tests__/paths.test.ts`, `src/__tests__/pathfix.test.ts`

Tokless binary resolvers in `internal/util/paths.go` + `exec.go` + `versions.go` etc:

- `ResolveRtkBin()`: searches PATH + known dirs (`~/.local/bin`, `~/.cargo/bin`, etc) for rtk binary, returns full path or ""
- `ResolveCodegraphBin()`: similar via Which + npm global bin + nvm paths
- `ResolveNpmBinary()`, `ResolveBunBinary()`, `Which(tool)`, `BinaryHealthy(bin)` → runs bin --version and checks code 0
- `CodegraphBinaryHealthy(binName)`
- `NpmInstalledVersionExported(pkg)`, `InstalledVersionFor(toolID)`, `LatestVersionFor(toolID)`, `StampCavemanVersion(v)`, `StampPonytailVersion(v)` etc version cache stamping
- `EnsureProcessPath()`, `PrependProcessPath(dir)`, `SelfHealPath()`, `EnsureNpmGlobalBinOnPath()`
- `WindowsHomeFromWSL()` — detects WSL and returns Windows home from env?
- `pathsetup.go`: `EnsureProcessPath`, `pathsetup_runtime_test.go` etc
- `EnsureDir`, `ReadFileSafe`, `WriteFile`, `Exists` — already in paths.ts? check.

For TokSave, many already exist in paths.ts, pathfix.ts, npm.ts, detect.ts. Need ensure completeness.

Check existing TokSave `paths.ts`: it has `claudePaths()`, `opencodePaths()`, `codexPaths()`, `antigravityPaths()`, `toksaveAbs()`, `ensureDir`, `readFile`, `writeFile`, `home()`, etc plus known bin dirs. But missing Resolve* helpers comprehensive.

Add:

```typescript
export function resolveRtkBin(): string { /* which rtk + known dirs */ }
export function resolveCodegraphBin(): string
export function resolveNpmBinary(): string
export function resolveBunBinary(): string
export function which(bin: string): string // wrapper around which library
export function binaryHealthy(binPath: string): boolean { /* exec --version */ }
```

- `selfHealPath()`: ensures ~/.local/bin and npm global bin on PATH env for current process, returns result for display via formatPathFixResult

TokSave already has `pathfix.ts` with `selfHealPath` and `formatPathFixResult`. Need ensure parity with tokless pathsetup: Adds `~/.local/bin` via selfHeal? Check.

- `WindowsHomeFromWSL()`: if running in WSL, return Windows home path from env? Look at `internal/util/gitwin.go`. Might be used to clean Windows hosts file for opencode plugin? In tokless autoindex unwire, there's WindowsHomeFromWSL for cleaning Windows opencode dir.

Implement minimal:

```typescript
export function windowsHomeFromWSL(): string {
  // If WSL, try get /mnt/c/Users/... from env WSL_DISTRO or check if /mnt/c exists?
  // Simpler: if process.platform !== "win32" and exists "/mnt/c/Windows", check env USERPROFILE from Windows? Or look at tokless impl.
  return "";
}
```

Find tokless `gitwin.go` and `pathsetup.go` for logic.

Also add `EnsureNpmGlobalBinOnPath()` — ensures npm global bin dir on PATH via `npm bin -g` or `npm root -g` + ../bin.

- [ ] **Step 1: Read tokless paths.go complete + pathsetup.go + exec.go for BinaryHealthy**

`Read tokless internal/util/paths.go` (maybe not full earlier). Also `internal/util/exec.go` for Run options.

- [ ] **Step 2: Write tests**

```typescript
describe("resolveRtkBin", () => {
  it("returns empty when not found", () => {
    // mock PATH empty
  });
});
describe("binaryHealthy", () => {
  it("returns false for non-existent", () => {
    expect(binaryHealthy("/nonexistent")).toBe(false);
  });
});
```

- [ ] **Step 3: Implement in paths.ts / binary.ts**

- Implement `which`, `resolveRtkBin`, `resolveCodegraphBin`, `resolveNpmBinary`, `binaryHealthy`, `codegraphBinaryHealthy`

- Enhance `pathfix.ts` for `windowsHomeFromWSL` and `ensureNpmGlobalBinOnPath`

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/paths.test.ts src/__tests__/pathfix.test.ts`

- [ ] **Step 5: Commit**

`feat: binary resolvers + selfHealPath + WSL home parity`

---

### Task 3: Npm install robust + version gathering

**Files:**
- Modify: `src/util/npm.ts`
- Modify: `src/util/version.ts`
- Modify: `src/util/versioncache.ts`
- Test: `src/__tests__/versioncache.test.ts` (existing)

Tokless npm install (`internal/util/npminstall.go`):

- `NpmGlobalInstall(pkg, tag)` returns version string, ok bool, error? Tries multiple strategies: npm install -g, then fallback tarball? `npm install --strict-peer-deps false`? Also `NpmGlobalInstall` does retry? Look at `npminstall_fallback_test.go`, `npminstall_oldnode_test.go`, `npminstall_path_test.go`, `npminstall_strict_test.go`, `npminstall_timeout_test.go`.
- `NodeMajor()`, `NodeAgeAlreadyChecked()`, `NodeTooOldHint(min)`, `InstallNodeForTools()`, `EnsureDeps(needNode, needGit, minNode)` bool, bool
- `NodeTooOldHint` gives manual upgrade guidance
- `npmattempts_test.go` etc suggests attempts counting
- `ManualNode`, `ManualRtk` tests

Version gathering (`internal/util/versions.go`):

- `VersionInfo {Installed *string, Latest *string, Present bool}`
- `GatherVersions()` returns map toolID -> VersionInfo (uses cache + network)
- `GatherVersionsForce()` forces network fetch (bust cache)
- `InstalledVersionFor(toolID)` -> *string or nil
- `LatestVersionFor(toolID)` -> *string
- `BustVersionCache()`
- `SemverCompare(a,b)` compares semver strings, returns -1/0/1
- `SemverGte(a,b)` bool
- `CountOutdated(v map)` count where installed != nil && latest !=nil && compare <0
- `StampCavemanVersion(v)`, `StampPonytailVersion(v)` etc — writes to version cache?

For TokSave:

- Existing `npm.ts` has `checkNode`, `getInstalledVersion`, maybe `installGlobal`. Need enhance with retry + fallback.
- `version.ts` has `toksaveVersion()`
- `versioncache.ts` has `getCachedLatest`, `setCachedLatest`, `getStaleFallback`, cache 6h TTL.

Need to produce `gatherVersions()` equivalent:

```typescript
export interface VersionInfo { installed: string|null, latest: string|null, present: boolean }

export async function gatherVersions(): Promise<Record<string, VersionInfo>>
export async function gatherVersionsForce(): Promise<Record<string, VersionInfo>> // bust cache then gather
export function installedVersionFor(id: string): string|null // wrapper over toolModules installedVersion
export function latestVersionFor(id: string): string|null // from cache
export function bustVersionCache(): void
export function semverCompare(a: string, b: string): number
export function semverGte(a: string, b: string): boolean
export function countOutdated(versions: Record<string,VersionInfo>): number
```

Also `ensureDeps(needNode, needGit, minNode)`:

- If needNode: check Node major >= minNode? If not, warn and offer upgrade via `InstallNodeForTools`? In tokless, if node too old, prompt confirm and try install node via nvm? Need simplified: just check node version and return false if not ok, printing guidance.

- `nodeMajor()`, `nodeTooOldHint(min)`, `installNodeForTools()` — for TokSave can reuse checkNode + hint.

- `manualNodeTest`, `manualRtkTest` — not needed.

Simplify npm install:

Tokless npm global install has multiple strategies with logs. For TokSave, keep existing but add retry fallback: if npm install -g fails with old node error, try alternative? At minimum log each attempt.

- [ ] **Step 1: Read tokless util/npminstall.go and versions.go**

`Read D:\KULIAH\token\TokSave\tokless\internal\util\npminstall.go` and `versions.go`

- [ ] **Step 2: Write tests for version compare**

```typescript
describe("semverCompare", () => {
  it("compares correctly", () => {
    expect(semverCompare("1.0.0","1.1.0")).toBe(-1);
    expect(semverCompare("1.1.0","1.0.0")).toBe(1);
    expect(semverCompare("1.0.0","1.0.0")).toBe(0);
  });
});
```

- [ ] **Step 3: Implement**

Update `src/util/npm.ts` with retry logic.

Update `src/util/version.ts` to add semver compare (use existing semver lib already dep: "semver" package).

Update `src/util/versioncache.ts` to add gatherVersions, bustVersionCache, etc.

Add `src/util/deps.ts` (or extend npm.ts) for `ensureDeps`.

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/versioncache.test.ts`

- [ ] **Step 5: Commit**

`feat: npm install fallback + version gathering + semver compare parity`

---

### Task 4: MCP spawn (PickMcpSpawn, WrapAutoIndex, McpSpawn)

**Files:**
- Create: `src/util/mcpspawn.ts`
- Modify: `src/commands/runmcp.ts`
- Modify: `src/agents/*` (use spawn wrappers)
- Test: `src/__tests__/runmcp.test.ts`

Tokless `mcpspawn.go` details (need read):

- `type McpSpawn struct { Command string, Args []string }`
- `PickMcpSpawn(toolID, ...extraArgs) McpSpawn` → picks best way to spawn MCP server for tool: check binary exists, else node? For codegraph/context-mode, npm global bin vs npx vs direct? Handles node shebang?
- `WrapAutoIndex(agent, spawn) McpSpawn` → wraps spawn to first run index auto then spawn? Or returns spawn that runs wrapper script that does auto-index?
- Used in `ConfigureCodexMcp` for codegraph: `WrapAutoIndex("codex", PickMcpSpawn("codegraph","serve","--mcp"))`

For TokSave, runmcp already handles shebang + auto-index pre-run. So PickMcpSpawn can simply return command = toksaveAbs() + args ["runmcp", toolID, ...extra] to keep consistent via proxy. Or for direct binary mode, return binary path.

Simplify: implement PickMcpSpawn that returns `toksave runmcp <tool>` spawn for all tools, plus WrapAutoIndex that prepends index check via env or adds wrapper? Actually auto-index already done in runmcp before spawn, so WrapAutoIndex could be identity for TokSave (ponytail comment: wrapper no-op because runmcp already pre-indexes, add real wrapper if direct binary path needed without runmcp).

Still need McpSpawn struct for API parity.

- [ ] **Step 1: Read mcpspawn.go and mcp_proxy.go**

`Read tokless internal/util/mcpspawn.go` + `internal/commands/mcp_proxy.go`

- [ ] **Step 2: Write tests**

```typescript
describe("PickMcpSpawn", () => {
  it("returns toksave runmcp spawn for codegraph", () => {
    const s = pickMcpSpawn("codegraph","serve","--mcp");
    expect(s.command).toContain("toksave");
    expect(s.args).toContain("codegraph");
  });
});
```

- [ ] **Step 3: Implement `src/util/mcpspawn.ts`**

```typescript
export interface McpSpawn { command: string; args: string[]; }

export function pickMcpSpawn(toolId: string, ...extra: string[]): McpSpawn {
  const abs = toksaveAbs();
  // for tools that need extra args like codegraph serve --mcp
  return { command: abs, args: ["runmcp", toolId, ...extra] };
}

export function wrapAutoIndex(agent: string, spawn: McpSpawn): McpSpawn {
  // For TokSave, runmcp already does auto index, so return as is
  // ponytail: if direct binary spawn needed without runmcp proxy, add index wrapper
  return spawn;
}
```

For codex special case, maybe add agent flag: `toksave runmcp --agent codex codegraph serve --mcp`

So pick should handle agent? In tokless WrapAutoIndex receives agent string to pass to runmcp? Check.

Tokless PickMcpSpawn signature: `func PickMcpSpawn(toolID string, arg ...string) McpSpawn` and WrapAutoIndex: `func WrapAutoIndex(agent string, spawn McpSpawn) McpSpawn`

So WrapAutoIndex might add `--agent <agent>` prefix to args for runmcp to know which agent triggering index. For TokSave, runmcp command already parses `--agent`.

Thus implement WrapAutoIndex to inject `--agent`:

```typescript
export function wrapAutoIndex(agent: string, spawn: McpSpawn): McpSpawn {
  // if spawn uses toksave runmcp, insert --agent flag after runmcp
  if (spawn.args[0]==="runmcp") {
    return { command: spawn.command, args: ["runmcp","--agent", agent, ...spawn.args.slice(1)] };
  }
  return spawn;
}
```

- [ ] **Step 4: Update runmcp.ts to handle --agent flag**

Ensure runmcp parses `--agent` prefix and calls `RunIndex` with auto=true using agent info.

- [ ] **Step 5: Tests pass**

`bun test src/__tests__/runmcp.test.ts`

- [ ] **Step 6: Commit**

`feat: McpSpawn + WrapAutoIndex parity (PickMcpSpawn via runmcp proxy)`

---

### Task 5: Prompt, logger, isatty, exec tree utils

**Files:**
- Modify: `src/util/prompt.ts`
- Modify: `src/util/colors.ts` (logger L)
- Modify: `src/util/exec.ts`
- Test: existing

Tokless prompt:

- `MultiSelect(label, options)` where option has Value, Label, Hint, Disabled, DisabledReason, Selected
- `SelectOne(label, options)` with Selected
- `SelectOne` vs `MultiSelect` uses rawmode, isatty, suspend handling for interactive TTY
- `Confirm(msg, default)` boolean

TokSave prompt already has multiSelect, but options simpler (value,label,disabled,hint,selected). Need ensure compatibility with tokless features: disabledReason display, hint.

Logger `L`:

- `L.Raw(msg)`, `L.Ok(msg)`, `L.Warn(msg)`, `L.Err(msg)`, `L.Info(msg)`, `L.Sub(msg)`, `L.Debug(msg)`
- `SetQuiet`, `SetVerbose`, `IsInteractive()`, `StdoutIsTTY()`

TokSave colors already has banner, ok, warn, raw etc but logger struct missing.

Add global logger similar:

```typescript
export const L = {
  Raw: (s: string) => console.log(s),
  Ok: (s: string) => console.log(pc.green(s)),
  Warn: (s: string) => console.log(pc.yellow(s)),
  Err: (s: string) => console.error(pc.red(s)),
  Info: (s: string) => console.log(pc.cyan(s)),
  Sub: (s: string) => console.log(pc.dim(s)),
  Debug: (s: string) => { if (verbose) console.log(pc.dim(s)); },
};
```

But keep existing colors.* functions for backward compat.

Exec tree:

Tokless `exec.go` has `Run(bin, args, RunOptions{Capture bool, Env []string, Ctx context, Cwd string, Quiet bool})` returns `{Code, Stdout, Stderr}`.

Tree kill on Unix via `exec_tree_unix.go`.

TokSave `exec.ts` probably simpler. Ensure parity: capture, env, cwd, quiet.

Also `process_alive` checks, `spawn_bg` — less needed for Node but for completeness maybe add `isProcessAlive(pid)`? Not needed for TS version, but we can implement minimal.

For this task, minimal needed:

- Extend prompt to handle DisabledReason display and default Selected handling as tokless.
- Add logger L.
- IsTTY detection.

- [ ] **Step 1: Read tokless util/prompt.go + colors.go + exec.go**

`Read tokless internal/util/prompt.go` (full) + `colors.go`

- [ ] **Step 2: Implement minimal prompt extension**

Update `src/util/prompt.ts`:

- Option type add `disabledReason`, `hint`, `selected`
- Display with gray hint, disabled reason
- MultiSelect returns selected values
- Confirm wrapper

- [ ] **Step 3: Implement logger**

In `src/util/colors.ts`, add export `L` object with Raw, Ok, Warn, Err, Info, Sub, Debug plus SetQuiet etc.

Make `verbose` flag handling work.

- [ ] **Step 4: IsTTY utilities**

Add `isInteractive()`, `stdoutIsTTY()`, `stdoutANSI()` functions.

- [ ] **Step 5: Tests pass for prompt**

`bun test` (general)

- [ ] **Step 6: Commit**

`feat: prompt disabledReason + logger L + isatty helpers parity`
