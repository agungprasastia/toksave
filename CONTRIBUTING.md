# Contributing to toksave

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
# Prerequisites: Rust toolchain (cargo, rustc >= 1.85), Node.js >= 22

# Clone the repo
git clone https://github.com/agungprasastia/toksave.git
cd toksave

# Run CLI in dev mode
cargo run -- --help

# Check compilation
cargo check

# Run tests
cargo test

# Lint + format check
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Build release binary
cargo build --release
```

## Project Structure

```
src/
├── main.rs           # Executable entry point
├── cli.rs            # CLI argument parser (clap)
├── registry.rs       # Agent/Tool enums, structs, & dispatch
├── agents/           # Per-agent wiring logic (claude, opencode, codex, etc.)
├── tools/            # Per-tool install logic (rtk, caveman, codegraph, etc.)
├── commands/         # CLI commands (init, doctor, update, uninstall, etc.)
└── util/             # Shared utilities (paths, json, toml, exec, etc.)
```

## Adding a New Agent

1. Create `src/agents/<name>.rs` implementing `detect()`, `wire()`, `unwire()`, `verify()`.
2. Add the agent to `ALL_AGENTS` in `src/registry.rs` and the `AgentId` enum.
3. Add dispatch branches in `src/agents/mod.rs` and `src/registry.rs`.
4. Add path helpers in `src/util/paths.rs`.
5. Add agent wiring tests in `tests/` using `env_test_lock()` and temporary directories.

## Adding a New Tool

1. Create `src/tools/<name>.rs` implementing the `Tool` trait (`install()`, `installed_version()`, `latest_version()`, `health_check()`, `repair()`).
2. Add the tool to `ALL_TOOLS` in `src/registry.rs` and the `ToolId` enum.
3. Add dispatch branches in `src/tools/mod.rs` and `src/registry.rs`.
4. Add `wire()` and `unwire()` cases in each agent module.
5. Add tool tests under `tests/` covering health check and repair behavior.

## Pull Request Process

1. Fork the repo and create a branch from `main`.
2. Make your changes.
3. Run `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
4. Update `CHANGELOG.md` for user-visible fixes/features and keep `Cargo.toml` version updated.
5. Open a PR with a clear description.

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for Cursor agent
fix: RTK asset name for linux-arm64
docs: update README install instructions
chore: bump dependencies
```

## Code Style

- Rust 2024 edition.
- Run `cargo fmt` before committing.
- Run `cargo clippy --all-targets -- -D warnings`.
- Explicit error types using `ToksaveError` and `Result<T>`.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
