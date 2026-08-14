# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

TokSave is a Rust CLI (`src/`) that installs and wires token-saving tools into AI coding agents. It targets 9 agents (Claude Code, OpenCode, Codex, Antigravity, GitHub Copilot, Droid, Devin, Warp, Cursor CLI) and wires RTK, Caveman, CodeGraph, Context-Mode, Ponytail, and Principles.

Node.js >= 22 is needed for full install flows involving npm-based tools (`context-mode`, `codegraph`, `ponytail`).

## Commands

```bash
cargo run -- --help
cargo run -- doctor --offline
cargo run -- doctor --fix
cargo check
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test agents
cargo test --lib
cargo build --release
```

CI runs `cargo check`, `cargo fmt --check`, `cargo clippy`, `cargo test`, and cross-platform release builds.

## Architecture

- [src/main.rs](src/main.rs) is the executable entrypoint. It handles early hook dispatch, parses CLI args, dispatches command modules, and exits with returned status code.
- [src/cli.rs](src/cli.rs) owns `clap` CLI setup and argument parsing. Default command is `init`.
- [src/registry.rs](src/registry.rs) is the central agent/tool matrix. Contains `ALL_AGENTS`, `ALL_TOOLS`, `AgentId`, `ToolId`, and `RunOpts`.
- [src/commands/](src/commands/) contains user-facing command flows:
  - `init` detects agents, checks tool dependencies, prompts/auto-selects targets, installs tools, and wires configs.
  - `doctor` checks agent wiring, tool health, and tool versions; `--offline` skips remote latest-version checks, and `--fix` runs tool repair for unhealthy tools.
  - `update` reinstalls tools with upgrade semantics.
  - `uninstall` unwires selected agents/tools and cleans empty configs.
  - `disable`, `self_update`, `runmcp`, `hooks`, `index` are specialized helper commands.
- [src/agents/](src/agents/) contains per-agent config writers. These modules know each agent's file layout and implement all tool-specific wiring/unwiring for that agent.
- [src/tools/](src/tools/) contains tool installation, health check, and repair logic (`rtk`, `caveman`, `codegraph`, `context_mode`, `ponytail`, `principles`).
- [src/util/paths.rs](src/util/paths.rs) centralizes cross-platform home/config/cache paths (`dirs` crate). Prefer it over inline path construction for agent config locations.
- [src/util/json.rs](src/util/json.rs) and [src/util/toml.rs](src/util/toml.rs) provide JSON/TOML config handling and pruning helpers (`write_json_pruned`, `write_toml_pruned`).
- [src/util/manifest.rs](src/util/manifest.rs) tracks wires in `TOKSAVE_CACHE_DIR` or `~/.cache/toksave/manifest.json`.

## Error Handling & Health Checks

- [src/util/errors.rs](src/util/errors.rs) defines `ToksaveError` and `ToksaveErrorKind` (`Tool`, `Install`, `Download`, `Network`, `HealthCheck`, `Integrity`, `Platform`, `Config`, `Io`).
- [src/util/health.rs](src/util/health.rs) defines health check types: `HealthStatus`, `HealthIssue`, `RepairResult`.
- Tool modules implement the `Tool` trait: `install()`, `installed_version()`, `latest_version()`, `health_check()`, `repair()`.

## Important details

- Rust edition is `2024`.
- Version lives in [Cargo.toml](Cargo.toml) (`version = "0.8.5"`).
- Tests modifying process environment variables (`HOME`, `PATH`) use `env_test_lock()` (`static ENV_LOCK`) to prevent race conditions during parallel test execution.
- Config unwiring prunes empty containers and deletes empty files to leave user configs clean.
