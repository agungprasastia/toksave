# TokSave Rust Rewrite — Phase 4 Design

Date: 2026-08-01
Branch: `rust-rewrite`
Status: In Progress

## Goal

Complete TokSave Rust rewrite by porting remaining hooks, advanced commands, and finalizing transition:
1. Port remaining 5 hooks: `rtk-hook`, `context-mode-hook`, `codex-perm-hook`, `agy-hook`, `copilot-hook`.
2. Port advanced commands: `runmcp`, `index`, `self-update`.
3. Remove TypeScript files, rename binary from `toksave-rs` to `toksave`, and move `rust/` contents to repo root.

## Components

### 1. Hooks (`src/commands/hooks/`)
- `rtk_hook.rs`: handle `rtk-hook <agent>` with JSON piping and modification output.
- `context_mode_hook.rs`: handle `context-mode-hook <agent> <event>` with stdio passthrough.
- `codex_perm_hook.rs`: intercept codex exec requests and modify permission payload.
- `agy_hook.rs`: handle Auto-index reroute on Antigravity session start (`agy-hook codegraph-index`).
- `copilot_hook.rs`: handle Copilot VS Code CLI hook events.

### 2. Advanced Commands
- `runmcp.rs`: resolve Node binary, spawn child processes (MCP proxies), expand PATH, handle shebang resolution.
- `index.rs`: spawn Codegraph auto-index logic (`index` command + `build-index` module).
- `self_update.rs`: download and swap TokSave binary itself (handles Windows rename-lock dance).

### 3. Transition & Cleanup
- Delete `src/` (TS codebase) at repo root.
- Flatten `rust/` directory contents to repo root (`Cargo.toml`, `src/`, `tests/`).
- Rename crate and binary from `toksave-rs` to `toksave`.
- Update CI/Release build pipelines to compile the `toksave` static binary across 5 targets.

## Verification

- All 13 commands successfully executable in the final static binary.
- 100% passing test suites (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`).
- CI release pipeline builds and packages correctly.
