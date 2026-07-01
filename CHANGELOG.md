# Changelog

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
