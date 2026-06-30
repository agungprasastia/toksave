# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
