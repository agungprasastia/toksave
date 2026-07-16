# [Phase 4] Upgrade Existing Tools to Tokless Parity

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring TokSave existing 4 tools (rtk, caveman, codegraph, context-mode) to tokless-level wiring fidelity: marketplace installs, prebuilt binary fallback, hooks override, WrapAutoIndex, verify for all agents, cache cleaning, version stamping.

**Architecture:** Each tool module in TokSave currently simplified. Need to port install/verify/wire logic from tokless internal/tools/*.go:

- rtk: platform prebuilt download (github releases asset) + curl sh install fallback + cargo install fallback + ResolveRtkBin + claude hook override to tokless wrapper + strip @RTK.md ref + dedup hook groups + AllowClaudeBashPattern
- caveman: resolveSkillsBin + resolveCavemanBin + skillsAgentID mapping + marketplace + skills add/remove + opencode plugin + relocate + AgentMd via unified + verify per agent (plugin list, skill dir existence)
- codegraph: codegraphRealInstall (install --yes --target), ConfigureMcp per agent (WrapAutoIndex for codex), WriteOwner, unwireAutoIndex legacy cleanup, VerifyFor checks, IndexProject background sync (go codegraphSyncBackground -> Bun background spawn)
- context-mode: NeedsNode 22 check, Node upgrade prompt, NpmGlobalInstall with cache-skew resistance, Plugin bare handling (setContextModePluginBare), cleanAllContextModeCache, runPostinstallInOpenCodeCache, ConfigureClaudeMcp + AllowClaudeMcpToolProjectLocal, Codex manual wiring (toml block + default_tools_approval_mode approve + cleanup hooks in dir + workspace cleanup), VerifyFor, Unwire per triple agents, plus codex bounded hook test exist

**Tech Stack:** TS, Bun exec, download/util existing, npm util.

## Global Constraints

- No breaking existing install flow — improvements incremental
- All agent wiring must go via unified block after Phase 1 (except RTK which has no section)
- VerifyFor must check both MCP and unified owner presence where applicable
- Keep dryRun behavior
- Tests must not call network / exec real in CI — use env override TOKSAVE_TEST=1 like before or spyOn
- Keep Bun.spawn patterns consistent with existing exec util

---

### Task 1: RTK tool parity

**Files:**
- Modify: `src/tools/rtk.ts`
- Modify: `src/agents/claude.ts` (hook override, strip RTK.md ref)
- Modify: `src/agents/codex.ts` (hook pre-trusted, allow entry)
- Modify: `src/agents/antigravity.ts`
- Modify: `src/agents/copilot.ts` + `src/agents/droid.ts` (if not yet full rtk wiring)
- Test: `src/__tests__/rtk.test.ts`, `src/__tests__/agents.test.ts`

**Interfaces:**
- Consumes: download util, paths, exec
- Produces: rtk install with prebuilt fallback, verify across agents, hook override.

Tokless rtk full details from `internal/tools/rtk.go` read earlier highlights:

- `rtkAssetForThisPlatform()`: maps GOOS/GOARCH to asset name
  - darwin x86_64 -> rtk-x86_64-apple-darwin.tar.gz
  - darwin arm64 -> rtk-aarch64-apple-darwin.tar.gz
  - linux arm64 -> rtk-aarch64-unknown-linux-gnu.tar.gz
  - linux x86_64 -> rtk-x86_64-unknown-linux-musl.tar.gz
  - windows x64/arm64 -> rtk-{arch}-pc-windows-msvc.zip
- `rtkEnsureInstalled()`:
  - test mode shim (temp shimDir with fake rtk binary, prepend PATH)
  - if ResolveRtkBin exists and !upgrade -> already installed
  - dryRun log would-download
  - try `rtkInstallPrebuilt(asset, opts)` → download tar.gz from `https://github.com/rtk-ai/rtk/releases/latest/download/{asset}` into `~/.local/bin`, extract, chmod 755, BinaryHealthy probe, PrependProcessPath
  - fallback: if !win and curl+sh exist, run install.sh via `curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh`
  - fallback: cargo install --git https://github.com/rtk-ai/rtk if cargo exists
- `rtkInstallPrebuilt()`:
  - Windows via Invoke-WebRequest + Expand-Archive PowerShell
  - Unix via DownloadAndExtractTarGz
  - After extraction chmod + binary healthy check
- Wire functions:
  - `rtkWireDroid()`, `rtkWireAntigravity()`, `rtkWireCodex()` -> InstallXxxRtkHook
  - Test shim for codex/claude/opencode/antigravity/copilot
  - `overrideClaudeRtkHook()` replaces rtk's own "rtk hook claude" PreToolUse command with tokless wrapper "tokless rtk-hook claude" + AllowClaudeBashPattern "Bash(rtk *)" + Remove RTK.md + stripRtkRefFromMd (remove line @RTK.md from CLAUDE.md etc)
  - `removeClaudeRtkHookGroup()` surgical strip managed PreToolUse group
  - `stripRtkRefFromMd(path)` removes @RTK.md line from markdown
- Tokless also has `rtk-hook` command with shellTokens, findUnsupportedFlags, rewrite fixpoint etc (already exists in TokSave partially)

For TokSave upgrade:

- Port asset mapping to TS using process.arch + process.platform
- Implement `installPrebuiltRtk(asset, opts)` using existing `download.ts` utilities (download tar.gz, extract via tar lib)
- Windows path: use PowerShell if needed (but our download util already handles zip? need check)
- Fallback cargo install via exec
- For Claude hook override:
  - After installing rtk, check existing settings.json for rtk hook with command "rtk hook claude" → replace with toksave wrapper "toksave rtk-hook claude"
  - Deduplicate hook groups (seen command map)
  - Allow Bash(rtk *) pattern
  - Remove RTK.md file + strip @RTK.md from CLAUDE.md
- For `rtkTestShim` — test mode wiring in TokSave already has setup? Need ensure test harness for rtk.

- [ ] **Step 1: Write tests for new rtk logic**

```typescript
describe("rtk platform asset", () => {
  it("maps darwin arm64 to correct asset", () => {
    expect(rtkAssetForPlatform("darwin","arm64")).toBe("rtk-aarch64-apple-darwin.tar.gz");
  });
  it("maps linux x64", () => {
    expect(rtkAssetForPlatform("linux","x64")).toBe("rtk-x86_64-unknown-linux-musl.tar.gz");
  });
  it("windows asset includes pc-windows-msvc", () => {
    expect(rtkAssetForPlatform("win32","x64")).toContain("windows-msvc");
  });
});
describe("rtk hook override", () => {
  it("replaces rtk hook claude with toksave wrapper", () => {
    // write temp settings.json with rtk hook claude command, call overrideClaudeRtkHook, verify replaced
  });
  it("strips @RTK.md line from markdown", () => {
    // write temp CLAUDE.md with "@RTK.md" line, call stripRtkRef, verify removed
  });
});
```

- [ ] **Step 2: Run fail**

`bun test src/__tests__/rtk.test.ts`

- [ ] **Step 3: Implement upgrade in `src/tools/rtk.ts`**

Keep existing structure but add:

```typescript
export function rtkAssetForPlatform(platform: string, arch: string): string {
  // map as tokless
}

async function installPrebuilt(asset: string, opts: RunOpts): Promise<boolean> {
  const url = `https://github.com/rtk-ai/rtk/releases/latest/download/${asset}`;
  const dest = join(home(), ".local","bin");
  await ensureDir(dest);
  // download via downloadAndExtract util
  // per platform: zip vs tar.gz
  // chmod 755
  // binary healthy check via exec `${bin} --version`
  // prepend to PATH via pathfix
}

export async function install(opts: RunOpts): Promise<boolean> {
  if (process.env.TOKSAVE_TEST==="1") { /* shim */ return true; }
  if (opts.dryRun) { /* log */ return true; }
  if (resolveRtkBin() && !opts.upgrade) return true;
  if (asset && await installPrebuilt(asset,opts)) return true;
  // curl | sh fallback
  // cargo fallback
  // error
}
```

Add helper `resolveRtkBin()` search via `findBinaryIn` + known paths.

Also export `overrideClaudeRtkHook`, `stripRtkRefFromMd`, `removeClaudeRtkHookGroup` for reuse in agent wire.

- [ ] **Step 4: Update claude agent wire to include override logic**

In `src/agents/claude.ts` wireRtk case:

```typescript
// after wireRtkHook
overrideClaudeRtkHook(); // replaces "rtk hook claude" with "toksave rtk-hook claude" and dedup
allowBashPattern("Bash(rtk *)");
removeFile(join(paths.claudePaths().dir, "RTK.md"));
stripRtkRefFromMd(paths.claudePaths().agentsMd); // remove @RTK.md ref
```

Implement `allowBashPattern` already exists.

Implement `stripRtkRefFromMd` in util or import from rtk tool.

- [ ] **Step 5: Tests pass**

`bun test src/__tests__/rtk.test.ts src/__tests__/agents.test.ts`

- [ ] **Step 6: Commit**

`feat: rtk prebuilt binary + fallback installers + claude hook override parity`

---

### Task 2: Caveman tool parity

**Files:**
- Modify: `src/tools/caveman.ts`
- Modify: `src/agents/claude.ts`, `opencode.ts`, `codex.ts`, `antigravity.ts`, `copilot.ts`, `droid.ts`
- Test: `src/__tests__/caveman.test.ts`, `src/__tests__/agents.test.ts`

Tokless caveman.go detailed (from earlier scan):

- `cavemanExec(bin,args,opts,dryHint,env...)`
- `cavemanOpencodeInstallEnv()` → returns XDG_CONFIG_HOME=... if dir base != "opencode"
- `resolveSkillsBin(npxArgs)` → if skills binary exists use "skills" else npx
- `resolveCavemanBin(agent, upgrade)` → if caveman binary exists use --only agent --no-mcp-shrink [--force] else npx -y github:JuliusBrussee/caveman -- ...
- `cavemanSkillsAddArgs(agent)` → ["-y","skills","add","JuliusBrussee/caveman","-a",skillsAgentID(agent),"-s","*","--yes","-g"]
- `cavemanSkillsRemoveArgs(agent)` → ["-y","skills","remove",...skillNames,"-a",...]
- `skillsAgentID(agent)` → copilot => "github-copilot" else same
- `relocateCavemanSkills(dstDir)` → copy from ~/.agents/skills/caveman etc to dstDir and remove src
- `removeCavemanSkillCopies(dir)`
- `codexSkillsDir()`, `claudeCavemanMemory()`, `codexCavemanMemory()`, `geminiCavemanMd()`
- `writeCavemanAgentsMd()`, `writeCavemanRuleset(p)`, `removeCavemanRuleset(p)` via WriteOwner
- `registerCavemanOpencode()` — adds "./plugins/caveman/plugin.js" to opencode.json plugin[] (and removes caveman-shrink mcp entry)
- `unregisterCavemanOpencode()`
- `opencodePluginFilesPresent()`, `opencodePluginInstalled()`
- `claudeCavemanInstalled()`, `codexCavemanInstalled()`, `antigravityCavemanInstalled()`, `copilotCavemanInstalled()`
- `claudePluginListHasCaveman()` → `claude plugin list` stdout contains caveman
- Caveman ToolManifest Install: if not upgrade and caveman binary exists -> already installed, else npm install -g github:JuliusBrussee/caveman (or npx fallback)
- WireFor:
  - claude: if claude CLI missing → fail, else `claude plugin marketplace add JuliusBrussee/caveman && claude plugin install caveman@caveman` via cavemanExec, plus check .caveman-active, settings.json contains caveman, write owner
  - opencode: registerCavemanOpencode via EnsureCommandsDir + WriteOwner etc
  - codex: skills add via skills CLI? `npx -y skills add ... -a codex ...` etc
  - antigravity: skills add for gemini/copilot? plus config/skills folder
  - copilot: skills add for github-copilot
  - droid: similar?
- UnwireFor removes plugin entries + skill dirs + ruleset owner

For TokSave parity upgrade:

Current TokSave caveman:
- Writes SKILL.md file directly for Claude (local), not marketplace install.
- Other agents via AGENTS.md fence.

Target tokless parity adds:
- skills CLI integration (npx skills add ...)
- caveman binary resolution (global vs npx)
- opencode plugin via "./plugins/caveman/plugin.js" not just AGENTS.md fence — actual plugin entry
- Relocate logic for global .agents/skills to agent-specific
- Verify via claude plugin list, opencode plugin files present etc.

Decide approach:
- For TokSave, keep SKILL.md direct write as fallback but add marketplace logic as primary when claude CLI present (more robust)
- For opencode: implement plugin.js entry + skills dir
- For other agents: skills CLI add
- Use unified block for instructions wiring

Also caveman tool currently has version via SKILL file version? In TokSave CAVEMAN_SKILL_VERSION. Keep.

- [ ] **Step 1: Write tests for new caveman logic**

```typescript
describe("caveman opencode register", () => {
  it("adds plugin entry ./plugins/caveman/plugin.js", () => {
    // temp opencode dir with config json
    // call registerCavemanOpencode()
    // verify plugin array contains entry
  });
  it("removes caveman-shrink mcp entry", () => {});
});
describe("skillsAgentID mapping", () => {
  it("copilot maps to github-copilot", () => {
    expect(skillsAgentID("copilot")).toBe("github-copilot");
  });
});
describe("caveman verify", () => {
  // per agent checks
});
```

- [ ] **Step 2: Implement in `src/tools/caveman.ts`**

Add helpers exported:

```typescript
export function skillsAgentID(agent: string): string {
  if (agent==="copilot") return "github-copilot";
  return agent;
}
export function resolveSkillsBin(npxArgs: string[]): { bin: string, args: string[] } { /* check which skills */ }
export function resolveCavemanBin(agent: string, upgrade: boolean): { bin: string, args: string[] } { /* which caveman */ }
export function cavemanSkillsAddArgs(agent: string): string[] { /* -y skills add ... */ }
export function cavemanSkillsRemoveArgs(agent: string): string[] { /* -y skills remove ... */ }

export function registerCavemanOpencode(): void { /* adds ./plugins/caveman/plugin.js */ }
export function unregisterCavemanOpencode(): void { /* removes entry */ }
export function opencodePluginInstalled(): boolean { /* checks config plugin entry + mcp cleanup */ }
export function opencodePluginFilesPresent(): boolean { /* checks .../plugins/caveman/plugin.js */ }

export function claudeCavemanInstalled(): boolean
export function codexCavemanInstalled(): boolean
// etc per tokless
```

Update install() to try npm -g github:JuliusBrussee/caveman if caveman binary not exist.

Existing `getSkillContent()` fetches from GitHub with fallback — keep.

- [ ] **Step 3: Update agent wites for caveman**

Each agent's wire caveman case should:

- If claude: check claude CLI exists, marketplace add + install, else write SKILL.md fallback; writeOwner("claude","caveman")
- opencode: registerOpencodePlugin + writeOwner
- codex: try skills add, writeOwner, maybe relocate skills
- antigravity: similar skills + config/skills
- copilot + droid: similar

For unwire: skills remove + plugin unregister + removeOwner + remove skill dir copies.

Implement verify per agent:

- claude: settings.json contains caveman OR .caveman-active exists OR plugin list contains caveman
- opencode: plugin installed && files present OR hasOwner?
- codex/antigravity/copilot/droid: existence of skill dir OR hasOwner

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/caveman.test.ts src/__tests__/agents.test.ts`

- [ ] **Step 5: Commit**

`feat: caveman marketplace + skills CLI + opencode plugin parity with tokless`

---

### Task 3: CodeGraph tool parity

**Files:**
- Modify: `src/tools/codegraph.ts`
- Modify: `src/agents/*` (codegraph wire/verify per agent includes index hook)
- Modify: `src/commands/build-index.ts` or `index.ts`
- Test: `src/__tests__/agents.test.ts` etc

Tokless codegraph.go detailed:

- `codegraphEnsureInstalled()`: if test mode true, else if ResolveCodegraphBin exists and !upgrade → already installed, else dryRun log would install npm -g, else `NpmGlobalInstall("@colbymchenry/codegraph","latest")` and check bin exists.
- `codegraphRealInstall(opts, agent)`: runs `codegraph install --yes? --target agent` — help check for --yes and --target flag existence via `codegraph install --help` capture, then build args. Timeout 10m.
- `codegraphConfigureMcp(agent)`: dispatches per agent ConfigureXxxMcp("codegraph") — for codex wraps via `PickMcpSpawn("codegraph","serve","--mcp")` + `WrapAutoIndex("codex", spawn)`
- `codegraphVerify(agent)`: per agent checks mcp entry present AND for antigravity/copilot/droid also check HasXxxCodegraphIndexHook
- `codegraphIndexProject(dir, opts)`: if test mode mkdir .codegraph, else if dryRun log, else go codegraphSyncBackground(bin,dir) — if .codegraph exists run `codegraph sync` async else `codegraph init -i` fallback to `init`
- `codegraphSyncBackground(bin,dir)`
- `codegraphWire(agent)`: in test mode configureMcp + WriteOwner + install index hooks (antigravity/copilot/droid), return verify; if dryRun run realInstall; else realInstall try and if fails log debug "writing MCP entry directly"; then configureMcp, writeCodegraphBlock (WriteOwner), unwireAutoIndex(agent), install hooks for antigravity/copilot/droid, syncCopilotIdeInstructions for copilot, return verify
- `unwireAutoIndex(agent)` already in Phase 1 from autoindex.go but for codegraph specifically
- `writeCodegraphBlock(agent)` = WriteOwner(agent,"codegraph")

- `UnwireFor` removes MCP + unwireAutoIndex + RemoveOwner
- `VerifyFor` per agent as above

TokSave current codegraph tool:
- install via npm global (already)
- wire via runmcp proxy (toksave runmcp codegraph serve --mcp) not via direct codegraph binary? tokless uses codegraph binary directly? Actually tokless `PickMcpSpawn` wraps with `WrapAutoIndex` for codex — note: PickMcpSpawn returns command+args for direct binary spawn (node shebang handling? need check) — while TokSave uses toksave runmcp proxy. Need decide parity.

For parity, TokSave can keep runmcp proxy approach (because it handles Node shebang). But need add WrapAutoIndex equivalent: `WrapAutoIndex("codex", spawn)` → spawn that runs auto-index hook before mcp.

Also need Codegraph index hook install for antigravity/copilot/droid: hooks that run on session start to auto-index project? In tokless there are:
- `InstallAntigravityCodegraphIndexHook()` writes hook config for index?
- `InstallCopilotCodegraphIndexHook()` etc.

These need port.

And unwireAutoIndex cleans legacy `tokless index --auto` SessionStart hooks from Claude, Codex, OpenCode, Antigravity.

- [ ] **Step 1: Write tests for codegraph new behaviors**

```typescript
describe("codegraph install", () => {
  it("realInstall checks --yes flag presence", () => {});
});
describe("codegraph verify includes index hook for antigravity", () => {
  // mock mcp present but hook missing -> verify false
});
describe("codegraphIndexProject", () => {
  it("creates .codegraph dir in test mode", () => {});
});
```

- [ ] **Step 2: Implement improved `src/tools/codegraph.ts`**

- Add `codegraphRealInstall`-like: run `codegraph install --yes --target <agent>` if binary present. Capture via Bun.spawn. Check help for flag existence.

- Keep existing install via npm if not present.

- Add `indexProject(dir, opts)` async function: if test mode mkdir .codegraph, else spawn background `codegraph init` or `sync` (use detached? Use Bun.spawn with detached option, or spawn background without wait). Mimic go's goroutine: just fire-and-forget spawn.

- Wire helpers should be exported? Actually agent wiring already handles codegraph mcp. But tool module should export functions for agents to reuse: `configureMcp` is currently in agent modules. In tokless it's in agents. So for TokSave, keep agents owning mcp but add WrapAutoIndex logic.

- Need implement `wrapAutoIndex(agent, spawn)` similar to tokless `WrapAutoIndex("codex", PickMcpSpawn(...))` — this is mcp spawn wrapper that runs auto-index before serving? Let's find tokless mcpspawn.go.

In earlier file list: `internal/util/mcpspawn.go` — likely contains `WrapAutoIndex` and `PickMcpSpawn`.

Need read that file for exact behavior.

For now assume wrap adds env or pre-hook? Let's search tokless codebase reasoning: codegraph MCP needs to ensure index exists before query, so WrapAutoIndex might intercept MCP serve to first ensure project indexed (run `codegraph init/sync` if needed) then delegate to actual server. For TokSave's runmcp proxy, we already have `RunIndex` pre-run: In `runmcp.go`, `RunIndex(InitOptions{Agent: agent}, true)` is called before exec-ing binary. So TokSave already has auto-index pre-run. So we can reuse.

Simplify: For codegraph parity, ensure `src/commands/runmcp.ts` already calls index --auto equivalent before proxy. Verify existing runmcp does `runIndex`. If not, add.

- Implement `unwireAutoIndex(agent)` in `src/util/unified-block.ts` or separate `src/tools/autoindex.ts` — cleanup legacy SessionStart hooks for `tokless index --auto` / `toksave index --auto`.

Port autoindex.go logic:

```typescript
function unwireClaudeAutoIndex() {
  // read ~/.claude/settings.json, remove SessionStart groups containing "toksave index --auto" or "tokless index --auto"
}
function unwireCodexAutoIndex() { /* hooks.json */ }
function unwireOpencodeAutoIndex() { /* remove legacy plugin file tokless-codegraph-init.js */ }
function unwireGeminiAutoIndex() { /* ~/.gemini/settings.json hooks.SessionStart */ }
export function unwireAutoIndex(agent: string) { switch... }
```

This should be called during codegraph wire (after realInstall) to clean old hooks.

- [ ] **Step 3: Update agents for codegraph wiring completeness**

- Claude: wireMcp as before but ensure verify checks hasMcp
- OpenCode: similar
- Codex: configure mcp with WrappedAutoIndex — for TokSave, since we use runmcp proxy, we need runmcp to handle auto-index anyway. So codex mcp entry points to toksave runmcp codegraph ... which already does auto-index. So codegraph mcp verify just checks hasMcp.
- Antigravity: configure mcp + installCodegraphIndexHook + cleanupDeadIdeHooks + verify checks both mcpHas and hasIndexHook
- Copilot: configureCliMcp + IdeMcp + install Index hooks for both + verify includes both
- Droid: similar

Implement index hook installers:

For each agent, install index hook that runs `toksave agy-hook codegraph-index` or `copilot-hook codegraph-index` or generic? In tokless, hook name is `agy-hook codegraph-index` for antigravity, `copilot-hook codegraph-index` for copilot. Command runs codegraph sync/init in background based on stdin workspacePaths.

Port hook installer: write hooks.json entry like:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": ".*",
        "hooks": [
          { "type":"command", "command":"toksave agy-hook codegraph-index" }
        ]
      }
    ]
  }
}
```

But need per-agent variant: check tokless agents/*/codegraph index hook install impl.

- [ ] **Step 4: Tests pass**

`bun test src/__tests__/agents.test.ts`

- [ ] **Step 5: Commit**

`feat: codegraph real install + auto-index hooks + unwireAutoIndex parity`

---

### Task 4: Context-Mode tool parity

**Files:**
- Modify: `src/tools/context-mode.ts`
- Modify: `src/agents/*` (context-mode wire)
- Test: `src/__tests__/agents.test.ts`, `src/__tests__/context-mode-hook.test.ts` (new)

Tokless contextmode.go highlights (from truncated read earlier need full):

- NeedsNode min 22, Node age check via `NodeAgeAlreadyChecked()`, `NodeMajor()`, `Confirm("Upgrade Node.js now?")`, `InstallNodeForTools()` if old.
- `NpmGlobalInstall("context-mode","latest")` cache-skew-resistant (retry strategies)
- `pluginIsContextMode(entry)` check for `context-mode` or `context-mode@`
- `setContextModePluginBare(cfg)` ensures opencode.jsonc plugin array ends with bare `context-mode` entry (not versioned)
- Claude wire: `PickMcpSpawn("context-mode")`, create mcp entry type stdio command spawn, AllowClaudeMcpTool("context-mode"), AllowClaudeMcpToolProjectLocal, WriteOwner, tip slash commands
- OpenCode wire: EnsureDir, loadOrdered, setContextModePlugin (adds "context-mode" bare), write config, WriteOwner, `cleanAllContextModeCache()` (removes old ~/.cache/opencode/packages/context-mode* dirs), `runPostinstallInOpenCodeCache()` (runs bun pm trust context-mode in cache dirs)
- Codex wire: Check codex CLI on PATH else fail, wireCodexManual(): upsert config.toml block `mcp_servers.context_mode` with spawn, enabled true, default_tools_approval_mode approve, cleanup old hooks, cleanupWorkspaceCodexContextModeMcp, writeCodexAgentsMd via WriteOwner, InstallCodexContextModeHook
- `writeCodexAgentsMd()` via WriteOwner("codex","context-mode")
- `removeCodexContextModeHooks(existing OrderedMap)` removes hook entries prefixed "context-mode hook codex ..."
- `cleanupCodexContextModeHooks()` removes context-mode hooks from CodexPathsResolved() Dir + CWD/.codex
- `cleanupWorkspaceCodexContextModeMcp(activeCodexDir)` removes mcp_servers.context-mode from project .codex/config.toml if different than active
- `isOursForEvent(entry,event)` checks command prefix "context-mode hook codex "...
- Unwire per agent removes mcp + owner, plus codex hook cleanup etc
- Additional verification?

Open questions: codex bounded test file `contextmode_codex_bounded_test.go` suggests context-mode for codex has bounded behavior? Maybe codex config.toml size limits? Need check.

For TokSave upgrade:

- Add Node 22 check before install (re-use existing checkNode)
- Implement setContextModePluginBare logic (ensure plugin array ends with bare "context-mode")
- Implement cleanAllContextModeCache and runPostinstallInOpenCodeCache (optional but nice)
- Claude wire: use runmcp proxy like other tools, plus allow MCP tool
- Codex wire: match tokless wireCodexManual: write toml block with enabled true and approval approve, plus InstallCodexContextModeHook
- Need port `InstallCodexContextModeHook` — hook for context-mode for codex? In tokless it's `InstallCodexContextModeHook()` — similar to codegraph index hook but for context-mode
- Also need "context-mode hook codex ..." hook entries removal logic for unwire

Check existing TokSave codex agent wiring for context-mode: need compare.

- [ ] **Step 1: Read contextmode.go full (remaining 10k lines not yet)**

`Read D:\KULIAH\token\TokSave\tokless\internal\tools\contextmode.go` but need complete (earlier truncated at 31758). Use ctx_execute to cat full file.

Also read `internal/agents/codex.go` for ConfigureCodexMcp for context-mode and InstallCodexContextModeHook.

- [ ] **Step 2: Write tests**

```typescript
describe("context-mode opencode plugin bare", () => {
  it("ensures bare context-mode entry at end, removes versioned", () => {
    const cfg = { plugin: ["context-mode@1.0.0", "other"] };
    setContextModePluginBare(cfg);
    expect(cfg.plugin).toEqual(["other","context-mode"]);
  });
});
describe("context-mode cache clean", () => {
  it("removes old cache dirs", () => {});
});
```

- [ ] **Step 3: Implement upgrade in `src/tools/context-mode.ts`**

Add helpers:

```typescript
function isContextModePlugin(entry: string): boolean {
  return entry === "context-mode" || entry.startsWith("context-mode@");
}
export function setPluginBare(cfg: any): void {
  let plugins = cfg.plugin as any[] ?? [];
  const kept = plugins.filter((p:any)=>!(typeof p==="string" && isContextModePlugin(p)));
  kept.push("context-mode");
  cfg.plugin = kept;
  // also clean mcp.context-mode if present (like tokless)
}

export function cleanAllContextModeCache(): void {
  // ~/.cache/opencode/packages/context-mode* remove
}
export function runPostinstallInCache(): void { /* bun pm trust */ }
```

Update install(): add node version check, use npm global install.

Update agent wiring functions to call new helpers.

For codex:

Implement `wireCodexManual()` that writes toml mcp_servers.context_mode block with command = toksave runmcp context-mode, enabled true, approval approve, plus cleanup.

Port `cleanupCodexContextModeHooksInDir` and other hook cleanup.

- [ ] **Step 4: Update agent modules**

- claude: wire already has context-mode case — enhance to allow MCP tool project local + writeOwner + tip message (maybe skip tip)
- opencode: setPluginBare + cleanCache + postinstall + writeOwner
- codex: implement wireCodexManual equivalent
- antigravity, copilot, droid: ensure context-mode mcp wiring

- [ ] **Step 5: Tests pass**

`bun test src/__tests__/agents.test.ts`

- [ ] **Step 6: Commit**

`feat: context-mode npm cache handling + bounded codex wiring + opencode bare plugin parity`

---

### Task 5: Cross-tool verification & integration

**Files:**
- Test: `src/__tests__/agents.test.ts` (full matrix)

Create matrix test like tokless `detect_test.go` + verify for all agents x tools:

For each agent (6) x each tool (6) = 36 combos, verify after wire true, after unwire false (or null).

- [ ] **Step 1: Write matrix test**

```typescript
const agents: AgentId[] = ["claude","opencode","codex","antigravity","copilot","droid"];
const tools: ToolId[] = ["rtk","caveman","codegraph","context-mode","ponytail","principles"];

for (const agent of agents) {
  for (const tool of tools) {
    it(`${agent} x ${tool} wire/verify/unwire`, async () => {
      const home = tempDir();
      overrideEnv(home);
      const wired = await wireTool(agent, tool, {dryRun:false, ...});
      expect(verifyTool(agent,tool)).toBeTruthy();
      await unwireTool(agent, tool, ...);
      expect(verifyTool(agent,tool)).toBeFalsy();
    });
  }
}
```

This mirrors tokless extensive test coverage.

- [ ] **Step 2: Run matrix — fix failures iteratively**

`bun test src/__tests__/agents.test.ts -t "matrix"`

- [ ] **Step 3: Commit**

`test: add full agent x tool wiring matrix`
