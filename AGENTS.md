# TokSave Agent Instructions

## Project
TokSave is a Rust CLI that installs and wires token-saving tools into AI coding agents. Targets Claude Code, OpenCode, Codex, Antigravity, Copilot, Droid, Devin, Warp, and wires RTK, Caveman, CodeGraph, Context-Mode, Ponytail, Principles.

## Commands
- `cargo fmt --check` (format check)
- `cargo clippy --all-targets -- -D warnings` (no warnings)
- `cargo test` (all test suites)

## Workflow
- Root crate is `toksave` (crate name in `Cargo.toml`).
- Entrypoint is `src/main.rs` which dispatches commands via `clap`.
- `src/registry.rs` contains the unified agent & tool matrix.
- **Support & Tools**: Modules in `src/agents/` and `src/tools/`.

## Architecture & Safety Rules
1. **Trust boundaries**:
   - `src/util/json.rs` parses config safely. Corrupted config files (e.g., Warp `hooks.json`) must NOT be overwritten. They throw `ToksaveError::Config`.
2. **Tests**: `tests/common/mod.rs` provides `common::setup()` which sets up temporary HOME and overrides cache paths.
3. **Manifest**: `src/util/manifest.rs` tracks wiring state using `manifest.json` requiring an atomic lock (`manifest.json.lock`) before modifying.
4. **Runmcp**: `src/commands/runmcp.rs` handles direct child process spawns, resolves Node shebangs, and manages PATH resolution.

## Style Guide
- 2 spaces, 100 line width (standard Rust format). Strict `clippy` (`-D warnings`).
- No TS codebase. `AGENTS.md` is the source of truth for contributors.