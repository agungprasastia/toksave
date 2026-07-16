# [Phase 5] Commands Parity — disable, index, doctor, update, uninstall, self-update, hooks

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring tokless commands to TokSave: `disable` (surgical per-agent per-tool un-wire), `index` (auto vs manual project indexing, hook variants agy-hook/copilot-hook), `doctor` (probing upstream versions with spinner, repo footer, tree UI), `update` (version diff table, selective upgrade prompt, resyncWiring), `uninstall` (surgical multi-select + purge binaries + cache cleanup), `self-update` (spinner, direct binary download, re-exec), plus internal hook commands (`rtk-hook` variants for copilot/droid/codex/claude/agy, `codex-perm-hook`, `agy-hook codegraph-index`, `copilot-hook codegraph-index`, `runmcp` with pre-index).

**Architecture:** Tokless commands use shared InitOptions {Agents, Tools, Agent (singular for runmcp), DryRun, Yes, Verbose, Upgrade}. Progress UI via SectionProgress/RootSectionProgress tree leaves, colors, spinner (`runStatus`), logger `L`. TokSave currently has simpler Progress and commander. Need to port logic but adapt to TS style: keep boxen/cli-table for version display, add selective prompts, version diff similar to tokless update.go, and re-sync wiring after upgrade. For disable/uninstall surgical: reuse prompt multiSelect for agent+tool selection.

**Tech Stack:** Commander still, but add internal command handling matching tokless main.go dispatch: run-mcp, rtk-hook <agent>, codex-perm codex, agy-hook codegraph-index, copilot-hook codegraph-index. Plus flags --agents, --tools, --dry-run, --verbose, --yes, --offline.

## Global Constraints

- CLI parsing must support tokless syntax: `tokless --agents claude,opencode` (TokSave currently supports -a with space? Need comma split support too)
- DryRun must never write files
- `--yes` skips confirmations (auto-select detected, no prompts)
- `--verbose` enables verbose logger
- All file writes in uninstall/disable must handle partial failure gracefully
- Tests for commands should avoid network and real fs via temp HOME and spyOn exec
- No new deps unless needed (can add ora or existing progress)
- Self-update after success must re-exec process (tokless reexec via exec replacement) — in Node use process.execPath + spawn? Tokless uses direct binary replace then exec. For TokSave (Bun compiled binary) similar: download new binary to temp, chmod, replace current executable, then re-exec with same args.
- For hooks: stdin JSON parsing, no logging to stdout (hooks must be silent unless error) — match tokless.

---

### Task 1: CLI parser parity + disable command

**Files:**
- Modify: `src/cli.ts`
- Create: `src/commands/disable.ts` (new)
- Modify: `src/commands/index.ts`
- Test: `src/__tests__/cli.test.ts`

**Interfaces:**
- Consumes: registry, prompt, paths
- Produces: disable command, surgical selection logic.

Tokless `disable.go` / `uninstall` logic:

- `RunDisable(opts)` → `disableImpl(opts, false, "Disabled")`
- `RunUninstall(opts)` → `disableImpl(opts, true, "Uninstalled")`
- `disableImpl`:
  - Print header: `tokless  disabled|uninstalled`
  - Detect installed agents via `a.Detect().Installed` → `detected []string`
  - If none → "nothing wired." return 0
  - Stage 1: `pickAgents(opts, detected, verb)`:
    - If opts.Agents != nil → filter detected ∩ opts.Agents
    - If !interactive → all detected
    - Else MultiSelect "Select agents to {verb} tokless from" — detected pre-selected
    - Returns chosen agentIDs (empty → "Nothing selected.")
  - Stage 2: `pickTools(opts, allTools, verb)`:
    - If opts.Tools != nil → filter
    - If !interactive → all tools
    - Else MultiSelect "Select tools to {verb}" — all pre-selected (default complete removal)
    - Returns chosen tools
  - Progress bar `NewProgress("")` Start(len agentIDs), for each agent Begin label, `WithSilencedLogs` loop tools unwire via `tool.UnwireFor[id]` if opts.DryRun false, Complete, Done
  - If removeTools && !dryRun && len tools == len allTools && len agentIDs == len detected → `purgeBinaries(opts)` + remove cache dir `~/.cache/tokless`
  - Print result: "✔ Disabled/Uninstalled {toolLabels} from {agentLabels}."
- `pickAgents` and `pickTools` handle --agents, --tools flags vs interactive.
- `purgeBinaries(opts)`:
  - If dryRun log would purge
  - If TOKLESS_TEST=1 return 0
  - If !Yes && interactive → Confirm "Also remove binaries/packages tokless installed (rtk, npm globals)?" default false
  - If purge: `rtk init --uninstall` via exec? Actually ResolveRtkBin then `rtk init --uninstall` capture, rm binary. npm uninstall -g context-mode, @colbymchenry/codegraph. Count removals.
- `runPurge()` removes rtk binary + npm globals.

For TokSave: port similar, but:

- RTK binary removal + npm uninstall for codegraph, context-mode, ponytail, caveman? In Tokless purge removes rtk + context-mode + codegraph only. Ponytail also npm global, caveman also. Should include all npm tools.
- For disable: verb "Disabled" is per-tool/per-agent removal, not full purge. If tools == all and agents == all and removeTools (i.e. uninstall command), then also purge binaries.
- Current TokSave `uninstall.ts` exists — modify to surgical.

New command `disable`: wired similarly but `removeTools=false` so never purges binaries. Use same `disableImpl` helper.

Main CLI: need add disable command:

```typescript
program.command("disable").description("Disable one or more agents/tools") action -> command disable
```

Update `CommandType` union add "disable".

- [ ] **Step 1: Write tests**

```typescript
describe("disable command", () => {
  it("parses --agents copilot filter", () => {
    const cli = parseCli(["node","toksave","disable","--agents","copilot,claude"]);
    expect(cli.command).toBe("disable");
    expect(cli.agents).toContain("copilot");
  });
  it("pickAgents filters by detected when --agents supplied", () => {});
  it("pickTools all when no flag and non-interactive", () => {});
});
```

- [ ] **Step 2: Implement `src/commands/disable.ts`**

```typescript
export async function run(agentsFilter, toolsFilter, opts): Promise<number> {
  return disableImpl(agentsFilter, toolsFilter, opts, false, "Disabled");
}
export async function runUninstallSurgical(..., removeTools boolean, verb) { /* shared */ }

function disableImpl(...) {
  // port tokless logic
  // detect installed agents
  // pickAgents: if agentsFilter non-empty use filter, else if yes||!interactive all detected, else prompt
  // pickTools similarly
  // progress bar via Progress class
  // for each agent, for each tool: await unwireTool(agent, tool, opts)
  // if verb === "Uninstalled" && selected all == detected all && tools all == allTools => purgeBinaries if confirmed
  // print result
}

async function purgeBinaries(opts): Promise<number> {
  // if dryRun || test mode skip
  // confirm if not yes and interactive
  // for each binary: try rtk init --uninstall, npm uninstall -g
}
```

Also update existing `src/commands/uninstall.ts` to call shared `disableImpl` with `removeTools=true, verb="Uninstalled"` OR move logic there and have disable.ts import.

Simpler: Keep uninstall.ts as wrapper over disable.ts `runUninstallSurgical`.

- [ ] **Step 3: CLI integration**

```typescript
// cli.ts
export type CommandType = ... | "disable"

program.command("disable")...
// also map "init" default still
```

In `src/index.ts` (entrypoint) dispatch:

```typescript
case "disable": return disable.run(...)
case "uninstall": return disable.runUninstall(...) or uninstall.run(...)
```

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/cli.test.ts`

- [ ] **Step 5: Commit**

`feat: add disable command (surgical per-agent per-tool unwire) + uninstall parity with purge`

---

### Task 2: Index command — auto vs manual + hooks

**Files:**
- Modify: `src/commands/build-index.ts` (or rename `src/commands/index.ts`)
- Modify: `src/cli.ts` (auto flag, agent flag)
- Create/Modify: `src/commands/codegraph-index-hook.ts` (agy-hook + copilot-hook)
- Test: `src/__tests__/cli.test.ts`, `src/__tests__/index.test.ts` (new)

Tokless `index.go` detail:

- `projectMarkers = [".git","package.json","go.mod","Cargo.toml","pyproject.toml","pom.xml","build.gradle","tsconfig.json","requirements.txt"]`
- `looksLikeProject(dir)` checks markers
- `findProjectDir(dir)` walks up looking for markers
- `RunIndex(opts, auto bool)`:
  - get CWD via os.Getwd
  - if auto: `dir = findProjectDir(dir)`; if !looksLikeProject(dir) return 0 (silent)
  - Collect indexable tools: those with `IndexProject != nil` (currently only codegraph)
  - If !auto: print header "tokless index  build per-project indexes in {dir}"
  - If len indexable ==0: if !auto print "no tools need per-project index." return 0
  - ro = RunOpts{DryRun, Agent: opts.Agent}
  - For each indexable:
    - if IndexReady != nil && !IndexReady() → if !auto print "{Label} not installed — run tokless first" + failed++
    - else `ok, err = t.IndexProject(dir, ro)`; if auto continue (silent); else switch ok/err print check / cross / warn
  - If auto return 0
  - If failed==0 print "✔ Project indexed." else "⚠ Some tools could not index."
  - Return 1 if failed>0 else 0

- `RunCodegraphIndexHook()` handles `tokless agy-hook codegraph-index` and `copilot-hook codegraph-index`:
  - Read stdin JSON: {workspacePaths:[], cwd}
  - `resolveHookProjectDirFromInput(input)`:
    - if input has workspacePaths[0] → findProjectDir(that)
    - else if cwd → findProjectDir(cwd)
    - else Getwd + findProjectDir
  - If .codegraph exists in dir → `codegraph sync` async spawn (detached, Start not Wait)
  - Else → `codegraph init` async spawn
  - return 0

- `resolveCodegraphBin()` tries ResolveCodegraphBin, plus glob ~/.nvm/versions/node/*/bin added to PATH (NVM workaround) then resolve again.

For TokSave parity:

- Existing `src/commands/build-index.ts` likely does manual indexing per tool. Need upgrade to match tokless auto logic + hooks.
- Add `auto` flag handling (internal --auto used by hooks, not user-facing? In tokless, `index` has --auto internal for hooks but also user can run `tokless index` manual)
- Add hook command handlers for `agy-hook codegraph-index` and `copilot-hook codegraph-index`

In `src/cli.ts`, add:

```typescript
// internal: not shown in help? but handled at entry main.go early if argv[1]=="agy-hook" && argv[2]=="codegraph-index"
```

In `src/index.ts` entrypoint dispatch currently only handles runmcp, rtk-hook, codex-perm-hook, context-mode-hook, etc. Need add handling for `agy-hook` and `copilot-hook` with subcommand `codegraph-index`.

Check tokless main.go for hook dispatch:

```go
if len(os.Args) >= 3 && os.Args[1] == "agy-hook" && os.Args[2] == "codegraph-index" {
  return commands.RunCodegraphIndexHook()
}
if len(os.Args) >= 3 && os.Args[1] == "copilot-hook" && os.Args[2] == "codegraph-index" {
  return commands.RunCodegraphIndexHook()
}
```

So for TokSave, add similar early returns before commander parse.

Implementation of RunCodegraphIndexHook in TS:

```typescript
export function runCodegraphIndexHook(): number {
  let input = "";
  try { input = fs.readFileSync(0,"utf-8"); } catch {}
  const dir = resolveHookProjectDir(input);
  if (!dir) return 0;
  const codegraphBin = resolveCodegraphBin();
  if (!codegraphBin) return 0;
  if (existsSync(join(dir,".codegraph"))) {
    // spawn detached: codegraph sync
    spawn(codegraphBin, ["sync"], { cwd: dir, detached: true, stdio:"ignore" }).unref();
  } else {
    spawn(codegraphBin, ["init"], { cwd: dir, detached: true, stdio:"ignore" }).unref();
  }
  return 0;
}

function resolveHookProjectDir(input: string): string {
  try {
    const req = JSON.parse(input);
    if (req.workspacePaths?.[0]) return findProjectDir(req.workspacePaths[0]);
    if (req.cwd) return findProjectDir(req.cwd);
  } catch {}
  return findProjectDir(process.cwd());
}
```

Also need `findProjectDir`, `looksLikeProject` helpers.

- [ ] **Step 1: Write tests for index helpers**

```typescript
describe("findProjectDir", () => {
  it("walks up to git root", () => { /* create tmp with .git at parent, cwd child, expect parent */ });
  it("returns dir if has marker", () => {});
  it("auto returns 0 if not project (silent)", () => {});
});
describe("codegraph index hook", () => {
  it("parses workspacePaths from stdin", () => {});
});
```

- [ ] **Step 2: Implement `src/commands/build-index.ts` upgraded**

Add `findProjectDir`, `looksLikeProject`, `projectMarkers`.

Update `run()` to support `auto` boolean (internal). Export second function `runAuto()` or `runIndex(opts, auto)`.

User-facing `toksave index` → manual (auto=false) with header output.

Hook `agy-hook` → auto=true style (silent, uses findProjectDir + check markers).

Also wire `codegraph` tool's `indexProject` method if needed — for TokSave registry tool `indexProject` is `ToolManifest.IndexProject`. Check existing `src/tools/codegraph.ts` already has `indexProject`? Probably. Need ensure it does background sync spawn.

- [ ] **Step 3: Add early dispatch in `src/index.ts` for agy-hook and copilot-hook**

```typescript
if (argv[1]==="agy-hook" && argv[2]==="codegraph-index") return runCodegraphIndexHook();
if (argv[1]==="copilot-hook" && argv[2]==="codegraph-index") return runCodegraphIndexHook();
```

Or handle inside cli parse as command.

Simplest: add before commander parse in index.ts.

- [ ] **Step 4: CLI auto flag**

In commander, `index` command option `--auto` hidden? For internal. Add boolean.

- [ ] **Step 5: Tests pass**

`bun test src/__tests__/index.test.ts` (new) + `bun test src/__tests__/cli.test.ts`

- [ ] **Step 6: Commit**

`feat: index command parity (auto vs manual + agy/copilot hooks)`

---

### Task 3: Doctor command parity

**Files:**
- Modify: `src/commands/doctor.ts`
- Test: `src/__tests__/cli.test.ts`, `src/__tests__/doctor.test.ts` (if exists)

Tokless doctor.go detail:

- Prints header "tokless doctor  quick health check"
- List tools = ListTools()
- Reports: for each agent in ListAgents(), det := Detect(), if !installed → report not installed, else check each tool VerifyFor[agent.ID]; if r != nil && !*r → missing label. Report struct {label, installed, wired, missing[]}
- `doctorSummary(r)` prints mark:
  - not installed: gray bullet + "not installed"
  - wired (len missing==0): green check + "all tools wired"
  - else: yellow warn + "missing: {joined missing}"
  - Format: `  {mark} {padEnd(label,14)} {status}`

- If !offline && !TOKLESS_TEST=1:
  - Show spinner/probing: `checking for updates…` if TTY else raw
  - `versions := GatherVersions()` → contains installed + latest per tool (network)
  - Erase spinner line if TTY
  - `listToolVersions(tools, v, false)` — prints one row per tool with outdated marker:
    - Marks: if installed != nil && latest != nil && semverCompare(installed,latest)<0 → Yellow "↑" + suffix " → upgrade" (add to changed)
    - If installed==nil && latest!=nil → Yellow "+" + " → install" (add changed)
    - If up to date → Green check + "(up to date)"
    - Calls toolVersionDisplayLine per tool
  - Then if outdated>0 warn "N tools available — run tokless update" else "All up to date."

- Count broken: reports where installed && !wired → broken count
- If broken>0: print "Run tokless to fix."
- `printRepoFooter(false)` → rule line 52 chars + star repo URL + issues URL

- `toolVersionOutdated`, `toolVersionDisplayLine`, `listToolVersions` helpers.

For TokSave:

- Existing doctor already does health check, version table maybe.
- Need add: repo footer, probing spinner (can skip spinner, just print "checking for updates…"), version display per func similar, outdated count, broken handling, offline flag support.

TokSave already has `offline` and `fix` flags. Keep fix flag: doctor --fix repairs unhealthy tools (existing). Ensure compatibility with new tools.

Also update doctor to use verification that includes ponytool/principles and copilot/droid agents.

- [ ] **Step 1: Write tests for version display line**

```typescript
describe("toolVersionDisplayLine", () => {
  it("shows up arrow when outdated", () => {
    const line = toolVersionDisplayLine(tool, {installed:"1.0.0", latest:"1.1.0"});
    expect(line).toContain("↑");
  });
  it("shows check when up to date", () => {});
  it("returns empty for instruction-only", () => {
    const t = { id:"principles", instructionOnly:true };
    expect(toolVersionDisplayLine(t, ...)).toBe("");
  });
  it("shows installed for notTrackable", () => {});
});
```

- [ ] **Step 2: Implement doctor upgrade**

Edit `src/commands/doctor.ts`:

- For each agent detection (including copilot/droid) via `ALL_AGENTS` + `detectAgent`
- Build report array, call `doctorSummary` helper ported
- If offline skip version probing, else gather versions (use existing `toolLatestVersion` + `installedVersion` + versioncache)
- Print versions per tool using `toolVersionDisplayLine`
- Count outdated
- Print repo footer: function `printRepoFooter(tree: boolean)` prints rule + star URL + issues URL (should use toksave repo URL, not tokless)
- Broken handling

Need helper `doctorSummary` function.

Also handle probing UI: if stdout is TTY, print inline? Can simplify to just log "checking for updates…"

- [ ] **Step 3: Tests pass**

`bun test src/__tests__/doctor.test.ts` (create if missing) + other tests

- [ ] **Step 4: Commit**

`feat: doctor command parity (version diff display + repo footer + spinner)`

---

### Task 4: Update command parity

**Files:**
- Modify: `src/commands/update.ts`
- Test: `src/__tests__/cli.test.ts`

Tokless update.go detail (full read earlier):

- Header "tokless update  refresh tools to latest"
- If DryRun: Info "Dry run — would probe registries and reinstall changed tools only."
- Probing line: "probing upstream…" with spinner if TTY else raw
- `versions := GatherVersionsForce()` (force refresh, ignoring cache? but uses cache? Force means ignore cache?)
- Loop tools: display installed vs latest, mark:
  - installed != nil && latest != nil && compare <0 → Yellow "↑" + suffix " → upgrade", changed add
  - installed==nil && latest !=nil → Yellow "+" + " → install", changed add
  - installed !=nil && latest !=nil → Green check " (up to date)"
  - else gray bullet " (latest unknown)"
- Print lines: `  {mark} {padEnd(id,14)} {padEnd(installed,10)} → {padEnd(latest,10)} {suffix}`
- Blank line
- If dryRun: if len changed>0 Info "Would upgrade: {joined}" else "Everything up to date." return 0
- If len changed==0 → Ok "Everything up to date." return 0
- If !Yes && interactive: MultiSelect "Select tools to update" with picked tools pre-selected (each with installed → latest hint). Selected becomes new changed list. If none selected → Info "No tools selected." return 0
- Print "Upgrading: {joined}" + header "tokless  global token-saver"
- Check deps: needNode, needGit, minNode from changed tools; EnsureDeps; if not ok filter out keep list.
- If len changed==0 after deps filter → Err "Missing dependencies..."
- Tools filter: those whose ID in changed
- Progress bar `NewProgress("")` Start(len tools), for each tool Begin label, report func, WithSilencedLogs tool.Install(Upgrade:true), Fail/Complete, Done
- After install loop: `ConfigureInstructionConflicts(true)` + `resyncWiring(tools)` + `ConfigureInstructionConflicts(false)` — re-run WireFor for each upgraded tool only on agents where already wired (VerifyFor true), syncing version pins without newly wiring
- BustVersionCache()
- Ok "Updated {joined}." return 0

- `resyncWiring(tools)`:
  - agents = ListAgents()
  - for each tool in upgraded tools:
    - for each agent in agents:
      - wire, ok := tool.WireFor[agent.ID]
      - if !ok or !agent.Detect().Installed → continue
      - if verify, vok := tool.VerifyFor[agent.ID]; vok { if r := verify(); r==nil || !*r continue } // only re-wire where already wired
      - WithSilencedLogs wire(RunOpts{Upgrade:true})

For TokSave parity:

- Update command already exists — enhance to match logic.
- Add version diff table with marks ↑/+ etc before upgrade.
- Add selective prompt (if interactive and >1 changed? Or always? Tokless prompts if !Yes && IsInteractive)
- Add resyncWiring after upgrade: re-wire only agents where verify true
- Add dry-run mode that only probes and displays would-upgrade list
- Add deps check
- Bust version cache after success

- [ ] **Step 1: Write tests for update logic**

```typescript
describe("update version diff", () => {
  it("detects outdated when installed < latest", () => {
    expect(isOutdated("1.0.0","1.1.0")).toBe(true);
  });
  it("detects install needed when not installed", () => {});
});
describe("resyncWiring", () => {
  it("only re-wires agents already wired", async () => {
    // setup home with wired agent, call resyncWiring, verify hook still present not newly added to unwired agent
  });
});
```

- [ ] **Step 2: Implement update.ts upgrade**

```typescript
export async function run(agentsFilter, toolsFilter, opts): Promise<number> {
  banner("toksave update");
  if (opts.dryRun) log Info dry run would probe...

  // probing
  const versions = await gatherVersionsForce(); // need implement that bypasses cache? use toolLatestVersion with force? Could clear cache then gather.

  const changed: ToolId[] = [];
  for (const tool of ALL_TOOLS) {
    if (toolsFilter.length && !toolsFilter.includes(tool.id)) continue;
    const installed = toolInstalledVersion(tool.id);
    const latest = await toolLatestVersion(tool.id); // or from versions map
    // display row, determine changed
  }

  if (opts.dryRun) { ... }

  if (changed.length===0) { ok "Everything up to date"; return 0; }

  if (!opts.yes && isInteractive()) {
    // prompt multiSelect filtered changed
  }

  // deps check via checkNode
  // progress bar install:
  const s = new Progress();
  for (const toolId of changed) {
    s.start...
    await installTool(toolId, {...opts, upgrade:true});
    s.stop...
  }

  // resyncWiring
  await resyncWiring(changed, opts);

  // bust cache
  bustVersionCache? // if exists
  ok Updated...
  return 0;
}

async function resyncWiring(toolIds, opts) {
  for (const toolId of toolIds) {
    for (const agent of ALL_AGENTS) {
      if (!detectAgent(agent.id).installed) continue;
      const verified = verifyTool(agent.id, toolId);
      if (!verified) continue; // only if already wired
      await wireTool(agent.id, toolId, {...opts, upgrade:true});
    }
  }
}
```

Need to also handle ConfigureInstructionConflicts(true) — for TokSave, we could set a global flag that suppresses overwrite prompts during resync (autoAppend). Similar to tokless: `ConfigureInstructionConflicts(true)` sets autoAppend. Implement similar global in unified-block: `configureInstructionConflicts(true)`.

- [ ] **Step 3: Tests pass**

`bun test src/__tests__/cli.test.ts`

- [ ] **Step 4: Commit**

`feat: update command parity (version diff + selective upgrade + resync wiring)`

---

### Task 5: Self-update + hooks (rtk-hook variants, codex-perm-hook, runmcp)

**Files:**
- Modify: `src/commands/self-update.ts`
- Modify: `src/commands/rtk-hook.ts`
- Modify: `src/commands/codex-perm-hook.ts`
- Modify: `src/commands/context-mode-hook.ts` (maybe rename)
- Modify: `src/commands/runmcp.ts`
- Modify: `src/index.ts` (early dispatch)
- Test: `src/__tests__/rtk-hook.test.ts`, `src/__tests__/runmcp.test.ts`

Tokless self-update details (selfupdate.go):

- Constants owner/repo/installSh/installPs1
- `RunSelfUpdate()`:
  - local = ToklessVersion()
  - latest = fetchLatestReleaseTag() → GET https://api.github.com/repos/HoangP8/tokless/releases/latest tag_name
  - if local >= latest → print "✔ tokless v{local}" return 0
  - else runSelfUpdateWithStatus(latest) → spinner "tokless updating…" runs runSelfUpdateTo(latest)
    - if TOKLESS_TEST=1 return true
    - if Win: find pwsh/powershell, run `irm installPs1 | iex` with CI=1 env, capture, code 0 true else print fallback cmd and false
    - else Unix: selfUpdateUnixDirect(ctx, latest):
      - asset = toklessReleaseAsset() (linux/darwin x64/arm64)
      - exe path via os.Executable + EvalSymlinks, dir, CreateTemp in same dir "tokless-update-*"
      - download https://github.com/{owner}/{repo}/releases/download/v{latest}/{asset} to tmp
      - chmod 755, rename tmp to exe (atomic replace)
  - If ok print "✔ tokless v{local} → v{latest} updated" else fail

- `MaybeSelfUpdate(opts)` called at beginning of init if not dryer+not dev version:
  - selfUpdateRule() prints rule 52 chars
  - prints Bold "Tokless"
  - fetchLatestReleaseTagWithStatus(local) with spinner "tokless v{local} checking updates…"
  - if latest empty return
  - if SemverGte local latest → print check local return
  - else runSelfUpdateWithStatus(latest): if interrupted -> RestoreConsoleCP and exit 130, if ok print check updated + reexecAfterSelfUpdate() (which re-executes binary with same args)

- `runStatus(label, fn)` displays spinner frames ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"] 80ms ticker, erases line.

- `reexecAfterSelfUpdate()` in selfupdate_reexec.go: exec replacement via syscall Exec on Unix, CreateProcess on Windows.

For TokSave self-update parity upgrade:

- Add spinner status UI (can reuse Progress or simple console)
- Implement direct binary download + replace for Unix (use `toksave` release asset naming? Check toksave release script `scripts/build-release.sh` — need confirm asset names)
- Windows via PowerShell installer
- Add `MaybeSelfUpdate` check in init command at start (similar to tokless: if not dryRun and not -dev, check updates, if new version update then reexec)
- Implement reexecAfterSelfUpdate for Bun: spawn new process with same args? Or use `process.execPath` replacement?

Simplify for TokSave: Keep existing self-update that uses installer script, but add spinner and direct binary path option.

Also need self-update version check function `fetchLatestReleaseTag()` — GET GitHub API.

- [ ] **Step 1: Write tests for self-update logic**

```typescript
describe("fetchLatestReleaseTag", () => {
  it("parses tag_name stripping v", async () => {
    // mock fetch
  });
});
describe("toklessReleaseAsset parity for toksave", () => {
  it("maps platform arch to asset name", () => {});
});
```

- [ ] **Step 2: Implement improved self-update.ts**

Enhance existing with:

- spinner (simple via setInterval with frames)
- fetchLatestReleaseTag with timeout 5s
- selfUpdateUnixDirect: download asset from github releases of toksave repo (need repo owner/name constants for toksave, not tokless — should be agungprasastia/toksave)
- Windows branch via PowerShell
- reexec logic: after successful update, re-exec current process with same argv (use `spawn` with same args and exit? For true exec, Bun doesn't have exec, but spawn + exit works)

- [ ] **Step 3: Hooks parity**

**rtk-hook:**

Tokless has `rtk_hook.go` with many variants:

- `RunRtkHook()` for antigravity (toolCall JSON: name run_command, args CommandLine)
- `RunRtkHookCodex()` — maybe codex specific?
- `RunRtkHookCopilot()`
- `RunRtkHookDroid()`
- `RunRtkHookClaude()`?

From main.go:
```
if tokless rtk-hook agy -> RunRtkHook()
   rtk-hook codex|claude -> RunRtkHookCodex()
   rtk-hook copilot -> RunRtkHookCopilot()
   rtk-hook droid -> RunRtkHookDroid()
```

Each variant has different rewriting logic:
- `rtkRewrite` vs `copilotRtkRewrite` etc handling unsafe find flags
- Shell token parsing
- Fixpoint rewrite

TokSave currently has single `rtk-hook.ts` that handles claude? Need expand to support copilot, droid, agy, codex variants per agent.

Need read full rtk_hook.go (we got partial). Need read rest to understand differences.

For parity, implement:

- Base `rtkRewrite` using exec `rtk rewrite <cmdLine>` capture stdout
- For copilot: `copilotRtkRewrite` adds unsafe find check + per-segment fallback + fixpoint
- For each agent entry: parse stdin JSON format according to agent's hook payload (different agents send different JSON shapes? Claude sends PreToolUse with tool_input, Codex sends something else, Antigravity run_command, Copilot maybe similar, Droid maybe)
- Output: JSON with permissionDecision allow and updatedInput? Or hook decision?

Check existing TokSave rtk-hook implementation for Claude — writes JSON with `hookSpecificOutput`? Need read file.

- [ ] **Step 4: Read existing rtk-hook.ts and codex-perm-hook.ts full**

Use Read tool.

- [ ] **Step 5: Implement variant support**

```typescript
// src/commands/rtk-hook.ts
export function runAntigravity(): number { /* parse toolCall.name run_command, CommandLine */ }
export function runCodex(): number { /* parse codex hook input */ }
export function runCopilot(): number { /* parse copilot preToolUse */ }
export function runDroid(): number { /* parse droid */ }
export function runClaude(): number { /* existing */ }

// main entry in index.ts dispatch based on second arg
```

Add shell token parsing + unsafe find detection ported from Go:

```typescript
const FIND_UNSUPPORTED = new Set(["-not","!","-or","-o","-and","-a","-exec","-execdir","-delete","-print0",...]);

function shellTokens(s: string): string[] { /* port Go version */ }
function firstSegment(line: string): string { /* outside quotes split at &&||;| */ }
function rtkUnsafeFind(cmdLine: string): boolean { /* find + unsupported flag */ }
```

Implement `rtkRewriteOnce`, `rtkRewriteBySegment`, `rtkRewriteFixpoint`, `stripRtkAbsPath`.

- [ ] **Step 6: codex-perm-hook parity**

Tokless `codex_perm_hook.go` — likely handles Codex permission hooks allowing rtk/codegraph binaries? Need read.

- [ ] **Step 7: runmcp parity**

Tokless `runmcp.go`:

- `RunMcp(argv)`:
  - parse `--agent <id>` prefix if present
  - EnsureProcessPath
  - If argv[0] contains separator -> PrependProcessPath(dir)
  - If isCodegraphCommand and !CodegraphBinaryHealthy and ResolveCodegraphBin exists -> argv[0] = resolved
  - `RunIndex(InitOptions{Agent: agent}, true)` → auto index before mcp proxy (pre-run)
  - LookPath argv[0], then `runMcpProxy(agent, path, argv, env)`

- `runMcpProxy` (platform-specific):
  - Unix: exec with stdin/out/err passthrough? In tokless `runmcp_unix.go` vs windows
  - Needs process group handling for tree kill? Possibly.

For TokSave:

- Existing runmcp.ts already proxies. Need ensure it calls index auto before proxy (RunIndex with auto true). Does current do? Need check.
- Also need handle `--agent` prefix parsing.
- Need handle codegraph binary healthy check + resolve fallback.
- For platform-specific process handling: use Bun.spawn with stdio inherit.

- [ ] **Step 8: Tests pass**

`bun test src/__tests__/rtk-hook.test.ts src/__tests__/runmcp.test.ts`

- [ ] **Step 9: Commit**

`feat: self-update spinner + binary direct + rt-hook variants + codex-perm-hook + runmcp pre-index parity`
