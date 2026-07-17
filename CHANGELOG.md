# Changelog

All notable changes to TokSave will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.2] - 2026-07-17

### Added

- **Verify-after-wire regression test**: 6 new tests confirm dry-run wire does not pass verify for any agent×tool combo. Exposes missing dryRun guards in wire functions.

### Fixed

- **WireCaveman dryRun writes owner marker**: Claude Code and Antigravity `wireCaveman()` called `writeOwner()` even with `dryRun: true`, causing verify to falsely report caveman as wired. `writeOwner` now correctly skipped on dryRun.

## [0.8.1] - 2026-07-17

### Changed

- **Verify after wire**: Init command now calls `verifyTool()` after `wireTool()` succeeds. If verification fails, the tool is reported as not wired — matching tokless behavior. Prevents silent wiring failures.
- **Pre-flight dependency check**: Init command now uses unified `ensureDeps()` instead of ad-hoc `checkNode()`. Ready for git dep checks.

### Fixed

- **Agent detect false positives**: Config-dir fallback in `detect()` now only activates under `NODE_ENV=test` for all agents (claude, droid, antigravity, copilot). Prevents `~/.factory`, `~/.gemini`, or project-level `.vscode/mcp.json` from triggering false detection.

## [0.8.0] - 2026-07-17

### Added

- **Auto-index on agent startup**: CodeGraph index builds automatically when agents start, not just on install.
  - **Claude Code**: SessionStart hook in `~/.claude/settings.json` runs `toksave index --auto` on each session start.
  - **OpenCode**: `plugins/toksave-autoindex.js` fires on first `tool.execute.before` call.
  - Both agents install/remove/verify the auto-index through `wire("codegraph")` / `unwire("codegraph")` / `verify("codegraph")`.

- **New agents: Copilot and Droid**: Full agent manifest, detection, wire/unwire/verify for all 6 tools (RTK, Caveman, CodeGraph, Context-Mode, Ponytail, Principles). Added to registry, agent index, and matrix test.
  - [src/agents/copilot.ts](src/agents/copilot.ts) — GitHub Copilot agent with MCP + IDE MCP wiring, RTK hook, CodeGraph index hook, Context-Mode hook, copilot-instructions.md sync.
  - [src/agents/droid.ts](src/agents/droid.ts) — Droid agent with MCP + AGENTS.md wiring, PreToolUse hook removal for context-mode.

- **New tools: Ponytail and Principles**:
  - [src/tools/ponytail.ts](src/tools/ponytail.ts) — Ponytail npm package, version detection, install/uninstall.
  - [src/tools/principles.ts](src/tools/principles.ts) — Principles tool with owner-based verify.

- **Unified-block content system**: Replaced per-agent ctx-rules/rtk-rules with a shared `unified-block.ts` content system using marker-based blocks. All agents now share a single `agent-instructions.ts` for standard content blocks.
  - [src/util/unified-block.ts](src/util/unified-block.ts) — `writeOwner()`, `removeOwner()`, `hasOwner()`, `writeBlock()`, `removeBlock()` helpers.
  - [src/util/separators.ts](src/util/separators.ts) — Standard marker separators.
  - [src/content/agent-instructions.ts](src/content/agent-instructions.ts) — Unified content templates.
  - Migrated Claude Code, OpenCode, Codex, Antigravity from ctx-rules/rtk-rules blocks.

- **Caveman marketplace & skills CLI**: Caveman now supports installing skills from marketplace URLs, has a `skills` subcommand, and an OpenCode plugin for skill discovery.
  - [src/tools/caveman.ts](src/tools/caveman.ts) — `installCavemanSkill()`, `resolveCavemanBin()`, `registerCavemanOpencode()`, `installedSkills()`, `opencodePluginInstalled()`, marketplace URL support.

- **Progress tree & runStatus**: New [src/util/progress.ts](src/util/progress.ts) with `SectionProgress`, `RootSectionProgress`, `runStatus()` spinner with dynamic sections.

- **Version utils & health**: New [src/util/version.ts](src/util/version.ts) with `gatherVersions()`, `semverCompare()`, `semverGte()`, `countOutdated()`. New [src/util/deps.ts](src/util/deps.ts) with `ensureDeps()`. New [src/util/mcpspawn.ts](src/util/mcpspawn.ts) with `pickMcpSpawn()`, `wrapAutoIndex()`.

- **Enhanced colors**: [src/util/colors.ts](src/util/colors.ts) now exports `C` (Cyan, Bold, etc), `L` logger with `.sub()` / `.ok()`, `StdoutIsTTY`.

- **CodeGraph real install**: [src/tools/codegraph.ts](src/tools/codegraph.ts) now has `realInstall()` with progress bars, background auto-index via `codegraphIndexBg()`, and `indexProject()` that discovers and indexes directories.

- **Context-Mode for Copilot + Droid**: Added context-mode wiring for the two new agents in [src/tools/caveman.ts](src/tools/caveman.ts) (codex handler cleanup) and [src/agents/copilot.ts](src/agents/copilot.ts) / [src/agents/droid.ts](src/agents/droid.ts).

- **RTK hook override + stripRtkRef**: [src/agents/claude.ts](src/agents/claude.ts) now includes `overrideClaudeRtkHook()` to replace existing RTK hooks, and `stripRtkRefFromMd()` to clean up old references.

- **Matrix test (36 combos)**: New parametrized test in [src/__tests__/agents.test.ts](src/__tests__/agents.test.ts) covering all 6 agents × 6 tools wire→verify→unwire→verify cycles.

### Changed

- **Agent wiring migration**: Claude Code, OpenCode, Codex, Antigravity wiring moved from ctx-rules/rtk-rules content system to unified-block content system. Antigravity GEMINI.md rules cleaned up. Codex context-mode hook cleanup.

- **Paths extended**: [src/util/paths.ts](src/util/paths.ts) added `toksaveAbs()`, `copilotPaths()`, `droidPaths()`, `opencodeKnownBinDirs()`, `opencodeDesktopPaths()`.

- **Registry expanded**: [src/registry.ts](src/registry.ts) now includes Copilot, Droid, Ponytail, Principles agents and tools. Added `ALL_AGENTS` export with full agent manifest data.

- **CLI updated**: [src/cli.ts](src/cli.ts) and [src/index.ts](src/index.ts) register new commands, dry-run flag support, colorized output.

- **Commands enhanced**: `doctor` now shows `--offline` status cleaner. `update` handles Copilot/Droid. `uninstall` handles new agents/tools. `runmcp` now supports `--agent` flag, `disable` command added. `build-index` updated for codegraph changes.

### Fixed

- **Opencode context-mode verify**: `verify("context-mode")` was checking `mcp.context-mode` which gets deleted during wire (plugin entry used instead). Changed to check plugin array. Also fixed `unwire` to remove the plugin entry instead of the missing MCP entry.

- **Caveman SKILL.md version fallback**: `cavemanInstalled()` now checks for the version field in SKILL.md, falling back to the constant only when absent.

## [0.7.1] - 2026-07-06

### Fixed

- **Caveman outdated detection was a no-op**: `healthCheck()` in [src/tools/caveman.ts](src/tools/caveman.ts) compared `installedVersion()` against the bundled `CAVEMAN_SKILL_VERSION` constant, but `installedVersion()` itself falls back to that same constant when SKILL.md has no version field (which the official one doesn't). Removed the dead comparison — outdated detection is already handled correctly by `doctor.ts` via the async `latestVersion()` GitHub fetch.

### Added

- **Caveman test coverage**: New [src/__tests__/caveman.test.ts](src/__tests__/caveman.test.ts) covering `healthCheck()` (installed vs not-installed) and `latestVersion()` resilience on network failure.

## [0.7.0] - 2026-07-06

### Added

- **RTK native hook enforcement for Claude Code and OpenCode**: RTK enforcement upgraded from text-only instructions (AGENTS.md) to native agent mechanisms — Claude Code now uses a `PreToolUse` hook with `updatedInput` to auto-prefix Bash commands, OpenCode now uses a `tool.execute.before` plugin in `~/.config/opencode/plugins/`. This matches the hook-based enforcement Codex and Antigravity already had. AGENTS.md instructions retained as fallback/documentation.
  - Modified [src/agents/claude.ts](src/agents/claude.ts) to wire RTK via `PreToolUse` hook
  - Modified [src/agents/opencode.ts](src/agents/opencode.ts) to wire RTK via OpenCode plugin system
- **RTK unreachable binary detection**: New `isInstalledButUnreachable()` in [src/tools/rtk.ts](src/tools/rtk.ts) detects when RTK binary exists in `localBin()` but is missing from system PATH. `doctor` now shows actionable per-platform instructions (Unix shell rc vs Windows `setx`).

### Changed

- **Atomic multi-file writes in Antigravity wiring**: `wireMcp()` and `allowEntry()` in [src/agents/antigravity.ts](src/agents/antigravity.ts) now roll back all successfully-written files if any write in the batch fails, preventing half-wired states.

### Fixed

- **Null-safety in Codex hook removal**: `removeRtkHook()` in [src/agents/codex.ts](src/agents/codex.ts) now handles `readJsonFile()` returning null with `?? {}` fallback. Audited and applied same pattern across all `src/agents/*.ts` modules.
- **Test isolation: replaced global mock.module with scoped spyOn**: Removed `mock.module("../util/exec.js")` from [src/__tests__/rtk.test.ts](src/__tests__/rtk.test.ts) and replaced with `spyOn` + `mockRestore()` per-test, matching the pattern already used for `isOnPath` in [src/__tests__/agents.test.ts](src/__tests__/agents.test.ts). Eliminates cross-file mock contamination that caused inconsistent CI failures across OS.

## [0.6.1] - 2026-07-05

### Added

- **Agent wiring regression tests**: Added temp-home tests for Claude Code, OpenCode, Codex, and Antigravity `wire()`, `unwire()`, and `verify()` flows without touching real user config.

### Changed

- **Doctor auto-repair**: Added `toksave doctor --fix` to run existing tool health checks and repair unhealthy tool installations, while keeping default `doctor` read-only.

### Fixed

- **Null-safe config reads**: Fixed Claude Code, OpenCode, and Antigravity MCP removal/verification when config files do not exist yet.

## [0.6.0] - 2026-07-03

### Added

- **RTK auto-on for all agents**: Added RTK instruction blocks to AGENTS.md/instructions.md for all 4 AI agents, completing the "run once, everything works" philosophy
  - Created [src/content/rtk-rules.ts](src/content/rtk-rules.ts) with RTK usage instructions and helper functions
  - Modified [src/agents/claude.ts](src/agents/claude.ts) to inject RTK rules into Claude Code AGENTS.md
  - Modified [src/agents/opencode.ts](src/agents/opencode.ts) to inject RTK rules into OpenCode AGENTS.md
  - Modified [src/agents/codex.ts](src/agents/codex.ts) to inject RTK rules into Codex instructions.md (alongside existing PreToolUse hook)
  - Modified [src/agents/antigravity.ts](src/agents/antigravity.ts) to inject RTK rules into Antigravity AGENTS.md (alongside existing PreToolUse hook)
  - Instructions guide when to use `rtk` prefix for commands with large output, complement existing hooks where available
  - Claude Code and OpenCode now have RTK guidance via instructions (no hook support)
  - Codex and Antigravity have both hooks (auto-prefix) and instructions (guidance for edge cases)

### Changed

- **Caveman auto-fetch from GitHub**: Updated Caveman to fetch real SKILL.md content and versions from official source instead of hardcoded value
  - Modified [src/tools/caveman.ts](src/tools/caveman.ts) `latestVersion()` to fetch from GitHub releases API (https://api.github.com/repos/JuliusBrussee/caveman/releases/latest)
  - Added [src/tools/caveman.ts](src/tools/caveman.ts) `getSkillContent()` to fetch official SKILL.md from https://raw.githubusercontent.com/JuliusBrussee/caveman/main/skills/caveman/SKILL.md
  - Modified [src/agents/claude.ts](src/agents/claude.ts), [src/agents/antigravity.ts](src/agents/antigravity.ts) to use fetched content when wiring Caveman
  - Updated [src/content/caveman-skill.ts](src/content/caveman-skill.ts) `CAVEMAN_SKILL_VERSION` from "1.0.0" to "1.9.1" to match current official version
  - Users now get the latest Caveman skill content automatically on install/update
  - Local template remains as fallback when GitHub fetch fails
  - Philosophy fulfilled: all tools (RTK, CodeGraph, Context-Mode, Caveman) now fetch latest from official sources

### Fixed

- **Manifest race conditions & parse safety**: Implemented file-based locking (`manifest.json.lock`) and atomic temp-write-rename pattern to prevent manifest corruption. Added parse error warnings instead of silent config resets.
- **Safe configuration writes**: Updated config writers to throw on parse failures instead of returning empty objects/null, preventing existing configs from being overwritten on syntax errors. Implemented atomic file writes for all configuration files.
- **Codex hook preservation**: Refactored RTK hook wiring to merge hooks into `PreToolUse` and `PermissionRequest` instead of completely overwriting them. Removed global `approval_policy = "never"` mutation to avoid modifying user settings outside TokSave scope.
- **Antigravity multi-surface MCP safety**: Modified MCP wiring and settings updates to aggregate partial file write failures and throw explicit errors instead of failing silently. Corrected Context-Mode hook command path proxy target.
- **Tar archive path traversal safety**: Added path validation checks for `.tar.gz` extractions to prevent directory traversal exploits.
- **RTK installation fallback & cleanup**: Ensured downloaded RTK binary is removed if initial integration setup (`rtk init -g`) fails. Added script/cargo install fallbacks on GitHub release asset failures.
- **RTK local path resolution**: Updated version detection to search the local bin directory when `PATH` is not yet updated.
- **Indeterminate progress reporting**: Fixed download progress callback to trigger even when `Content-Length` is not provided (e.g. GitHub chunked transfers).
- **Rule upgrade refresh & validation**: Modified rules wiring to overwrite rule blocks on upgrade instead of skipping early. Improved verification checks to inspect rule presence in `AGENTS.md` and `instructions.md`.
- **Network timeout in fetches**: Added a 10-second timeout to all remote fetches (rtk, caveman, npm) to prevent indefinite hangs on poor connections.
- **Regex rule markers**: Improved rules removal regex to be resilient to carriage returns (`\r?\n`) and flexible whitespace formatting.
- **MCP idempotency**: Replaced fragile `JSON.stringify` comparisons with field-by-field validation to prevent false negatives caused by key ordering.
- **Caveman version detection**: Fixed `installedVersion()` returning "0.0.0" when official SKILL.md has no version field, and added support for detecting version from OpenCode and Codex instructions.
- **Caveman content extraction robustness**: Increased buffer from 12 to 20 lines in `getCavemanInstructionBlock()` for better handling of content structure changes.
- **CodeGraph CI hang**: Removed `-i` interactive flag from `codegraph init` that caused indefinite hangs in non-interactive environments (CI, background processes).
- **Remaining dynamic `require()` calls**: Replaced leftover `require("../content/ctx-rules.js")` in [src/agents/claude.ts](src/agents/claude.ts) and `require("node:path")` calls in [src/agents/antigravity.ts](src/agents/antigravity.ts) with static ES imports.

## [0.5.0] - 2026-07-03

### Added
- **Missing hook commands**: Added `rtk-hook` and `context-mode-hook` CLI commands that were referenced in Codex and Antigravity agent wiring but didn't exist, causing RTK and Context-Mode hooks to fail silently
  - [src/commands/rtk-hook.ts](src/commands/rtk-hook.ts) - PreToolUse hook that prefixes Bash commands with `rtk`
  - [src/commands/context-mode-hook.ts](src/commands/context-mode-hook.ts) - PreInvocation hook that proxies to context-mode CLI
  - Updated [src/cli.ts](src/cli.ts) and [src/index.ts](src/index.ts) to register both commands
- **User-Agent with version**: Created `userAgent()` helper in [src/util/version.ts](src/util/version.ts) that returns `toksave/<version>` for better debugging of API issues
  - Updated all HTTP requests in [src/util/download.ts](src/util/download.ts), [src/tools/rtk.ts](src/tools/rtk.ts), and [src/util/npm.ts](src/util/npm.ts)
- **Test step in release workflow**: Added `bun test` step in [.github/workflows/release.yml](.github/workflows/release.yml) before building release artifacts to prevent shipping broken binaries
- **Stricter TypeScript checks**: Added `noUncheckedIndexedAccess` and `noFallthroughCasesInSwitch` to [tsconfig.json](tsconfig.json) and fixed all 27 resulting type errors across the codebase

### Fixed
- **Security: Zip slip vulnerability**: Added path validation in [src/util/download.ts](src/util/download.ts) `downloadZip()` to prevent malicious archives from writing files outside the destination directory
  - Rejects absolute paths, path traversal sequences (`../`), and entries that escape destDir
- **Security: Config parse errors silently swallowed**: Added warnings when [src/config/json.ts](src/config/json.ts) and [src/config/toml.ts](src/config/toml.ts) fail to parse config files, preventing silent data loss when users make typos
- **semverCmp invalid input bug**: Fixed [src/util/version.ts](src/util/version.ts) to return `-1` instead of `0` when version parsing fails, preventing updates from being silently skipped
- **makeExecutable silent error**: Fixed [src/util/download.ts](src/util/download.ts) to throw error with remediation message instead of silently ignoring chmod failures
- **Dead code in RTK health check**: Removed unreachable `!isOnPath("rtk")` check in [src/tools/rtk.ts](src/tools/rtk.ts) that could never be true if `installedVersion()` succeeded
- **Dynamic require() usage**: Replaced dynamic `require()` calls with static ES imports in [src/util/paths.ts](src/util/paths.ts), [src/agents/claude.ts](src/agents/claude.ts), and [src/agents/antigravity.ts](src/agents/antigravity.ts)
- **Windows platform support**:
  - Added Windows-specific Node.js directories in [src/util/detect.ts](src/util/detect.ts) `resolveNode()`
  - Added proper Windows fallback for `localBin()` in [src/util/paths.ts](src/util/paths.ts) when LOCALAPPDATA is unset
- **Deleted misleading barrel file**: Removed [src/tools/index.ts](src/tools/index.ts) that only re-exported RTK while registry imports all tools directly

### Changed
- **Checksum verification**: Implemented SHA256 verification using existing `DownloadOptions.checksum` field in [src/util/download.ts](src/util/download.ts) for `downloadFile()`, `downloadTarGz()`, and `downloadZip()`
- **fetchJson retry logic**: Updated [src/util/download.ts](src/util/download.ts) `fetchJson()` to use `fetchWithRetry()` for consistent network resilience across all download operations
- **Pinned @types/bun version**: Changed from `"latest"` to `"1.3.14"` in [package.json](package.json) to prevent CI/local divergence
- **Updated actions/checkout**: Changed from `@v4` to `@v7` in [.github/workflows/release.yml](.github/workflows/release.yml) for consistency with build job
- **Removed dead tsconfig options**: Removed `outDir` and `rootDir` from [tsconfig.json](tsconfig.json) as they're meaningless with `noEmit: true`
- **Extracted fetchBuffer helper**: Deduplicated 15-line chunk-reading logic between `downloadTarGz()` and `downloadZip()` in [src/util/download.ts](src/util/download.ts) by extracting shared `fetchBuffer()` helper

## [0.4.1] - 2026-07-02

### Fixed
- **Caveman version tracking**: Fixed bug where Caveman showed version "0.0.0" even after running `toksave update`
  - Root cause: `wireCaveman()` functions only wrote skill files if they didn't exist, so update operations never rewrote old files with the new template containing `version: 1.0.0`
  - Modified agent wire functions in [claude.ts](src/agents/claude.ts) and [antigravity.ts](src/agents/antigravity.ts) to rewrite skill files when `opts.upgrade` is true
  - Enhanced `installedVersion()` in [caveman.ts](src/tools/caveman.ts) to check both Claude Code and Antigravity skill paths (previously only checked Claude Code)

## [0.4.0] - 2026-07-01

### Added

#### Enhanced Error Handling (Sprint 1.1)
- **Structured Error Classes**: Introduced comprehensive error hierarchy with context and remediation guidance
  - `ToolError`: Base error class with error codes and structured context
  - `InstallError`: Installation failures with actionable remediation steps
  - `DownloadError`: Network download failures with HTTP status codes
  - `NetworkError`: Network connectivity issues with retry suggestions
  - `HealthCheckError`: Health check failures with diagnostic information
  - `IntegrityError`: Checksum/integrity verification failures
  - `PlatformError`: Unsupported platform errors with fallback suggestions
  - `IndexError`: CodeGraph indexing failures with permission/binary checks
- **Actionable Error Messages**: All errors now include root cause explanation, specific remediation steps, relevant context (URLs, file paths, status codes), and caused-by stack traces for debugging

#### Health Check & Repair System (Sprint 1.2)
- **Tool Health Checks**: Added `healthCheck()` method to all tools
  - RTK: Verifies binary installation, PATH configuration, and version detection
  - CodeGraph: Checks npm package installation and binary availability
  - Context-Mode: Validates npm global installation
  - Caveman: Verifies skill file presence and version compatibility
- **Automated Repair**: Added `repair()` method to all tools that attempts automatic fixes for common installation issues
- **Health Status Types**: New structured health status interface with `HealthStatus`, `HealthIssue`, and `RepairResult`

#### Download Resilience (Sprint 1.3)
- **Retry Logic**: Implemented exponential backoff for network requests (default 3 retries with 1s, 2s, 4s delays)
- **Progress Reporting**: Added `onProgress(downloaded, total)` callback support for real-time progress on large downloads
- **Download Options**: New `DownloadOptions` interface with configurable `retries`, `timeout`, `onProgress`, `checksum`, and `fallbackUrls`

#### Caveman Version Tracking (Sprint 4)
- **Version Management**: Caveman skill now tracks semantic versions with `CAVEMAN_SKILL_VERSION` constant
- **Version Comparison**: Health check detects outdated skills and warns when installed version doesn't match latest
- **Backward Compatibility**: Old skills without version field default to "0.0.0"

### Changed

#### Tool Improvements
- **RTK**: Enhanced installation error handling, post-install verification, better platform detection errors
- **CodeGraph**: Replaced dynamic `require()` calls with static ES imports for type safety, enhanced `indexProject()` with `IndexResult` and `IndexOptions` interfaces
- **Context-Mode**: Enhanced error handling for npm installation failures, added health check for PATH verification
- **Download Utilities**: Refactored all download functions to use `fetchWithRetry()`, added streaming progress support, better HTTP error status handling
- **npm Utilities**: Enhanced `installGlobal()` error handling with npm configuration troubleshooting hints

### Fixed
- CodeGraph: Fixed silent failures in `indexProject()` (now throws `IndexError`)
- Download: Fixed missing error context in download failures
- RTK: Fixed installation failures not reporting specific error causes
- npm: Fixed generic error messages that didn't help troubleshooting

### Technical Improvements
- All tool modules now export `healthCheck()` and `repair()` methods
- Registry exports `toolHealthCheck()` and `toolRepair()` dispatchers
- New [src/util/errors.ts](src/util/errors.ts) module with error class hierarchy
- New [src/util/health.ts](src/util/health.ts) module with health status types
- Enhanced type safety throughout download and installation flows
- All tests pass (56 tests), TypeScript compilation clean

## [0.3.1] — 2026-07-01

### Changed
- **UX Progress Delay**: Added a slight artificial delay during `init`, `update`, and `uninstall` commands. This ensures the progress spinner is always visible, preventing the terminal from appearing to flash instantly to completion.

### Fixed
- **MCP Proxy Path Resolution Bug**: Replaced `process.argv[0]` with `"toksave"` in `toksaveAbs()` to correctly resolve the executable path when installed globally via npm. This prevents catastrophic crashes where AI agents attempt to execute `node runmcp` instead of the TokSave binary.
- **Windows File Descriptor Leak**: Refactored MCP binary shebang detection (`isNodeShebangScript`) to use a robust `try...finally` block. This guarantees file handles are closed even during read errors, preventing `toksave` from permanently locking executable files on Windows.

## [0.3.0] — 2026-07-01

### Added
- **JSONC Config Preservation**: Replaced standard JSON parser with `comment-json`. TokSave now flawlessly preserves all user comments (`//`, `/* */`) and key order when modifying agent configurations like `settings.json`.

### Fixed
- **Windows Command Execution Bug**: Added `shell: process.platform === "win32"` to subprocess execution. Fixes catastrophic installation failures for global npm packages (like CodeGraph and Context-Mode) on Windows.
- **Node.js MCP Proxy Shebang Resolution Bug**: Fixed an issue where `toksave runmcp` attempted to spawn `toksave.exe` instead of `node` for node-based MCP servers. Tools now correctly resolve and execute using the system's `node` binary.
- **Async Fetch Refactor**: Removed anti-pattern synchronous Node subprocessing for network requests (`node -e 'fetch'`) in favor of native asynchronous API calls (`await fetch`), improving performance and environment portability.

## [0.2.0] — 2026-07-01

### Added
- **Codex Permission Auto-Allow (`toksave codex-perm-hook`)**: Intercepts Codex permission requests to automatically allow harmless commands and MCP tool calls. Drastically reduces manual prompts.
- **MCP Proxy (`toksave runmcp`)**: A secure wrapper to resolve Node.js binaries and proxy stdio. Ensures MCP servers run flawlessly across platforms (especially Windows).
- **Proactive Indexing (`toksave index`)**: New command to pre-build CodeGraph indexes in the current directory to speed up the agent's first query.

### Fixed
- **RTK MCP Compliance**: Fixed an issue where the RTK hook intercepted MCP tools in Antigravity. The matcher is now specifically restricted to terminal commands (`Bash`, `cmd`, etc.), restoring 100% MCP compliance.

## [0.1.1] — 2026-06-30

### Fixed

- **CLI Output:** Fixed an issue where the Caveman skill was incorrectly reported as `not installed` in the `toksave` and `toksave doctor` CLI summaries.

## [0.1.0] — 2026-06-30

### 🚀 Initial Release

Zero-config CLI that installs and wires token-saving tools into AI coding agents.
Built with TypeScript + Bun. Compiles to standalone binary — zero runtime dependencies.

### Added

- **Interactive wizard** — run `toksave` with no args, pick agents, everything gets wired.
- **4 agents supported** — Claude Code, OpenCode, Codex, Antigravity.
- **4 tools installed** — RTK, Caveman, CodeGraph, Context-Mode.
- **5 commands** — `toksave` (install), `doctor`, `update`, `uninstall`, `self-update`.
- **`--agents` / `--tools` flags** — target specific agents or tools.
- **`--dry-run` flag** — preview changes without modifying anything.
- **`--verbose` flag** — detailed logs (which files written, which configs modified).
- **`--yes` flag** — skip interactive prompts, auto-select detected agents. CI-friendly.
- **Manifest tracking** — `~/.cache/toksave/manifest.json` records what toksave wired, so `uninstall` only removes what toksave added.
- **Full Caveman SKILL.md** — intensity levels (`lite`, `full`, `ultra`), persistence rules, auto-clarity, and boundaries.
- **Context-Mode routing rules** — injected into each agent's rules file (AGENTS.md or instructions.md). Redirects heavy operations through context-mode sandbox.
- **RTK auto-init** — `rtk init -g` called automatically after binary download to activate global shell integration.
- **OpenCode Caveman wiring** — writes caveman rules to `~/.config/opencode/AGENTS.md`.
- **Codex Caveman wiring** — writes caveman rules to `~/.codex/instructions.md`.
- **Idempotent** — safe to run multiple times, no duplicate entries.
- **Cross-platform** — macOS (Intel + Apple Silicon), Linux (x64 + arm64), Windows (x64).
- **Install scripts** — `curl | bash` for Mac/Linux, `irm | iex` for Windows PowerShell.
- **CI/CD** — GitHub Actions for CI (3 OS) and Release (5 targets).

### Agent × Tool Matrix

| Agent | MCP | Hooks | Caveman | Context-Mode Rules |
|-------|-----|-------|---------|-------------------|
| Claude Code | ✅ JSON | ✅ RTK bash allow | ✅ SKILL.md | ✅ AGENTS.md |
| OpenCode | ✅ JSON | — | ✅ AGENTS.md | ✅ AGENTS.md |
| Codex | ✅ TOML | ✅ RTK hook | ✅ instructions.md | ✅ instructions.md |
| Antigravity | ✅ JSON (multi-surface) | ✅ RTK + ctx hooks | ✅ SKILL.md | ✅ AGENTS.md |

### Build Targets

| Target | Format |
|--------|--------|
| `linux-x64` | `.tar.gz` |
| `linux-arm64` | `.tar.gz` |
| `darwin-x64` (macOS Intel) | `.tar.gz` |
| `darwin-arm64` (macOS Apple Silicon) | `.tar.gz` |
| `windows-x64` | `.zip` |
