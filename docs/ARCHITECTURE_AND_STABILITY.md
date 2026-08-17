# TokSave Architecture & Stability Guide

## Overview

**TokSave** is a zero-config, token-efficient toolchain written in **Rust (2024 edition)** that installs and wires token-saving tools into AI coding agents. It automates tool installation, agent detection, configuration wiring, runtime health probes, and clean uninstallation.

---

## Core Capabilities

### Supported Agents (8)
- **Claude Code** (`.claude.json`, `PreToolUse` hooks, skill directories)
- **OpenCode** (`config.json`, plugin registrations, `AGENTS.md`)
- **Codex** (`config.toml`, `hooks.json`, instructions)
- **Antigravity** (`mcp.json`, `hooks.json`, skill directories)
- **GitHub Copilot** (`mcp.json`, `hooks.json`, instructions)
- **Factory Droid** (`mcp.json`, `hooks.json`, instructions)
- **Devin / Cascade** (`mcp.json`, `hooks.json`, instructions)
- **Warp / Oz** (desktop `mcp.json` + `.mcp.json`, Agent CLI `.mcp.json`, `hooks.json`, instructions)

### Supported Tools (6)
- **RTK**: CLI proxy for compressing tool output (60-90% token savings).
- **Caveman**: Terse response mode (~75% output token reduction).
- **CodeGraph**: Pre-indexed code knowledge graph for minimal MCP round-trips.
- **Context-Mode**: MCP sandbox with session memory & context compression.
- **Ponytail**: Lazy-coding discipline (YAGNI, stdlib first, minimal diffs).
- **Principles**: Karpathy-style coding standards (surgical edits, explicit reasoning).

---

## Architecture & Design Highlights

### 1. Owner-Consolidated Instruction Blocks
All tool instructions written into `AGENTS.md` or `INSTRUCTIONS.md` are wrapped inside a single managed block:
```markdown
<!-- TOKSAVE:START -->
... [Consolidated tool instructions] ...
<!-- TOKSAVE:END -->
```
Running `toksave init` replaces this block cleanly without duplicating instructions across multiple runs.

### 2. Smart Config Pruning (`Clean Unwire`)
When running `toksave uninstall` or unwiring specific agents:
- **JSON Pruning** (`write_json_pruned`): Removes TokSave keys. If top-level containers become empty (`{}`), the file is automatically deleted. User-owned keys (e.g., `$schema` in OpenCode `config.json`) are preserved.
- **TOML Pruning** (`write_toml_pruned`): Recursively prunes empty `[mcp_servers]` tables from Codex/Antigravity `config.toml`.

### 3. Windows Native Reliability
- **PATHEXT Resolution**: Spawns PATHEXT-qualified binaries (`.cmd` / `.exe`) to prevent bare npm shim execution failures (`os error 193`).
- **Git Bash Path Validation**: `toksave doctor` checks Windows backslash paths inside hooks that would fail in Git Bash / POSIX environments.

### 4. Smart Init Sequence
During `toksave init`:
1. Tool flags & agent detection run **first**.
2. If no target agent is installed or selected, `init` short-circuits with `"Nothing selected."` without installing global npm/GitHub tools unnecessarily.
3. Preflight dependency checks warn if minimum Node.js (≥22) or Git requirements are missing before attempting tool downloads.

### 5. Self-Healing Diagnostics (`toksave doctor --fix`)
`toksave doctor` probes installed binaries, hook execution paths, and config health. Running `toksave doctor --fix` automatically repairs broken npm or release-download tool installations.

---

## Verification & Test Suite Status

TokSave maintains strict safety and reliability guarantees:

- **Compiler & Linter**: `cargo check`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (Clean / 0 warnings).
- **Unit & Integration Tests**: 35+ lib unit tests covering JSON/TOML pruning, PATH resolution, error reporting, and manifest tracking.
- **Thread Safety**: Multithreaded tests modifying `HOME` or `PATH` are serialized using `env_test_lock()`.
- **E2E Round-Trip Verified**:
  - `init` (Installs tools & wires configs) -> **Equipped 8/8**
  - `doctor` -> **100% Wired**
  - `uninstall` -> **100% Clean** (Only user-owned `$schema` preserved).
