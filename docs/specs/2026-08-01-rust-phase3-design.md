# TokSave Rust Rewrite — Phase 3 Design

Date: 2026-08-01
Branch: `rust-rewrite`
Status: Approved

## Goal

Implement Phase 3 of the TokSave Rust rewrite:
1. Complete remaining 5 Tools (`caveman`, `codegraph`, `context_mode`, `ponytail`, `principles`) and managed content blocks.
2. Complete remaining 7 Agents (`opencode`, `codex`, `antigravity`, `copilot`, `droid`, `devin`, `warp`) with full registration in `Registry`.
3. Port 4 core management commands: `doctor`, `update`, `uninstall`, `disable`.
4. Validate full 8 Agents × 6 Tools matrix and command execution through integration unit tests.

## Subsystems and Components

### 1. Managed Content Blocks (`src/content/`)
- `agent_instructions.rs`: embedded instruction content for system prompts.
- `caveman_skill.rs`: caveman skill definition content.
- `ctx_rules.rs`: context-mode rule blocks.

### 2. Tools Module (`src/tools/`)
- `caveman.rs`: Content-based tool (system prompt injection). `health_check`, `install` (noop/version cache).
- `codegraph.rs`: Node/npm dependency binary runner. Health check & version detection via binary spawn.
- `context_mode.rs`: Managed rule block injector (`ctx-rules`).
- `ponytail.rs`: Managed skill/prompt block injector.
- `principles.rs`: Managed instruction block injector.

### 3. Agents Module (`src/agents/`)
- `opencode.rs`: config `opencode.json` / `AGENTS.md` wiring.
- `codex.rs`: config `codex.json` / `AGENTS.md` / permission hooks wiring.
- `antigravity.rs`: MCP config wiring (`mcp.json` + batch rollback on write failure).
- `copilot.rs`: VS Code Copilot instructions wiring (`.github/copilot-instructions.md`).
- `droid.rs`: Factory Droid config wiring.
- `devin.rs`: Devin config wiring.
- `warp.rs`: `warp/hooks.json` wiring with safe parsing (error on unparseable JSON, never overwrite valid JSON with `{}`).

### 4. Core Management Commands (`src/commands/`)
- `doctor.rs`:
  - Detect installed agents and status of wired tools.
  - Health check installed tools.
  - Support `--offline` (skips remote version checks) and `--fix` (runs `tool.repair()`).
  - Terminal output formatting with status summary table.
- `update.rs`:
  - Resolve target tools.
  - Download & install updates in parallel for outdated tools.
  - Rewire updated tools across wired agents.
  - Update `manifest.json` atomically with lock.
- `uninstall.rs`:
  - Unwire target tools across detected agents.
  - Remove installed tool binaries from `~/.toksave/bin/`.
  - Update `manifest.json` atomically.
- `disable.rs`:
  - Unwire target tools from agents without removing binary files from `~/.toksave/bin/`.
  - Update `manifest.json` status to disabled.

## Safety and Trust Boundaries
1. **Config Safety**: Unparseable config files in Warp, Claude, Antigravity, OpenCode, Codex fail immediately with `ToksaveError::Config` and remediation instructions. Existing configs are never overwritten with defaults.
2. **Atomic Config Batches**: Multi-file agent wiring (e.g. Antigravity MCP) uses temp-write-commit pattern with automatic rollback on any error.
3. **Manifest Atomicity**: Lock-file guard (`manifest.json.lock`) during batch updates, uninstalls, and disables.

## Verification Strategy
- Cargo test suite passing (`cargo test`).
- Clippy clean (`cargo clippy --all-targets -- -D warnings`).
- Formatting clean (`cargo fmt --check`).
- Matrix tests in `tests/agents.rs` and `tests/tools.rs` covering all 48 Agent × Tool combinations.
- Command integration tests in `tests/commands.rs` for `doctor`, `update`, `uninstall`, `disable`.
