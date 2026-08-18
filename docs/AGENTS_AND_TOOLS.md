# Agent & Tool Integration Architecture

This document provides a detailed technical reference on how **TokSave** wires every tool into each supported AI coding agent.

---

## Tool Matrix Summary

| Tool | Type / Channel | Core Function | Primary Targets |
| :--- | :--- | :--- | :--- |
| **RTK** | GitHub Binary Asset | Intercepts tool execution and compresses output (60-90% token savings). | Native hooks (`PreToolUse`, `tool.execute.before`) & Instructions |
| **Caveman** | SKILL.md / GitHub | Enforces ultra-terse response mode (~75% output token savings). | Skill markdown files & Agent instruction blocks |
| **CodeGraph** | npm global | Background indexer & MCP server for code symbol exploration. | MCP config (`mcp.json` / `mcpServers`) & Auto-index hooks |
| **Context-Mode** | npm global | MCP server providing session memory & dynamic context reduction. | MCP config & OpenCode plugin registration |
| **Ponytail** | npm global | Enforces lazy-coding discipline (YAGNI, stdlib first, surgical diffs). | OpenCode plugin registration & Skill/Instructions |
| **Principles** | Instruction-only | Coding standards based on Karpathy & Karpathy-style guidelines. | Agent instruction files (`AGENTS.md` / `INSTRUCTIONS.md`) |

---

## Detailed Agent Wiring Specifications

### 1. Claude Code
- **Configuration Path**: `~/.claude/` & `~/.claude.json`
- **Hooks (`PreToolUse`)**: Wires RTK command prefixer into `~/.claude/hooks.json` or `~/.claude.json`.
- **MCP Servers**: Configures `codegraph` and `context-mode` under `mcpServers`.
- **Skills**: Installs `caveman`, `ponytail`, and `principles` skill files into `~/.claude/skills/`.

---

### 2. OpenCode
- **Configuration Path**: `~/.config/opencode/`
- **Plugins**: Registers `context-mode` and `@dietrichgebert/ponytail` in `config.json["plugin"]`.
- **Instructions**: Writes consolidated `<!-- TOKSAVE:START -->` instruction block to `AGENTS.md`.
- **Auto-Index**: Configures `tool.execute.before` plugin for automatic CodeGraph indexing.

---

### 3. Codex
- **Configuration Path**: `~/.codex/`
- **Configuration File**: `~/.codex/config.toml` (`[mcp_servers]`) and `~/.codex/hooks.json` (`PreToolUse`).
- **Instruction Block**: Appends/replaces consolidated instruction block in `~/.codex/instructions.md`.

---

### 4. Antigravity
- **Configuration Path**: `~/.antigravity/`
- **MCP & Hooks**: Modifies `~/.antigravity/mcp.json` and `~/.antigravity/hooks.json`.
- **Skills**: Writes skill rules under `~/.antigravity/config/skills/`.

---

### 5. GitHub Copilot CLI
- **Configuration Path**: Platform-specific VS Code / Copilot globalStorage.
- **Wiring**: Wires MCP servers into `mcp.json` and RTK execution guards into `hooks.json`.

---

### 6. Factory Droid
- **Configuration Path**: `~/.factory/`
- **Wiring**: Configures `mcp.json` for MCP tools and updates `INSTRUCTIONS.md`.

---

### 7. Devin / Cascade
- **Configuration Path**: `~/.devin/`
- **Wiring**: Sets up `mcp.json` and updates instruction context.

---

### 8. Warp / Oz
- **Desktop**: `~/.warp/mcp.json` (legacy TokSave path) and official file-based MCP at `~/.warp/.mcp.json`.
- **Warp Agent CLI**: platform MCP file (`~/.warp_cli/.mcp.json` on macOS; `${XDG_CONFIG_HOME:-~/.config}/warp-terminal/cli/.mcp.json` on Linux; `%LOCALAPPDATA%\warp\Warp\config\cli\.mcp.json` on Windows), plus the documented `~/.warp_cli/.mcp.json` path on every platform.
- **Oz cloud**: same `warp` / `oz` agent id. Cloud MCP is per-run (`oz agent run --mcp` / agent YAML) and is not written by TokSave. Shared rules/skills still apply.
- **Wiring**: MCP for CodeGraph and Context-Mode on every local Warp MCP file; RTK via `~/.warp/hooks.json`; instructions via `~/.warp/instructions.md`.

---

### 9. Cursor CLI
- **Configuration Path**: `~/.cursor/` (or `CURSOR_CONFIG_DIR`). Cursor reads `~/.cursor/hooks.json`, not `$XDG_CONFIG_HOME/cursor`.
- **Hooks**: Native `hooks.json` `{ version, hooks.preToolUse }` with `matcher: "Shell"` calling `toksave rtk-hook cursor`.
- **Permissions**: Adds `Shell(rtk *)` to `cli-config.json` `permissions.allow`.
- **MCP Servers**: Configures `codegraph` and `context-mode` in `mcp.json`.
- **Instructions**: Writes the consolidated TokSave block to `~/.cursor/AGENTS.md`.

---

## Unified Instruction Block Format

TokSave manages instructions across agents using a single, idempotent block:

```markdown
<!-- TOKSAVE:START -->
# TokSave Agent Instructions

## RTK
Compress terminal tool output. Run commands via RTK proxy.

## Caveman
Respond in concise, ultra-terse language. Omit fluff and pleasantries.

## Ponytail
Follow lazy senior developer discipline:
1. YAGNI — do not write speculative code.
2. Stdlib first — do not add dependencies for trivial tasks.
3. Surgical edits — minimal diffs win.

## Principles
1. Think before editing.
2. Simplify relentlessly.
<!-- TOKSAVE:END -->
```

Unwiring an agent surgically strips this block while leaving user-written instructions untouched.
