<div align="center">
  <img src="assets/Logo.png" alt="toksave logo" width="300" />
</div>

# toksave

**Zero-config token-saver for AI coding agents.**

[![version](https://img.shields.io/github/v/release/agungprasastia/toksave?label=version)](https://github.com/agungprasastia/toksave/releases)
[![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)](https://github.com/agungprasastia/toksave)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/agungprasastia/toksave/actions/workflows/ci.yml/badge.svg)](https://github.com/agungprasastia/toksave/actions)

Install and wire [RTK](https://github.com/rtk-ai/rtk), [Caveman](https://github.com/JuliusBrussee/caveman), [CodeGraph](https://github.com/colbymchenry/codegraph), [Context-Mode](https://github.com/mksglu/context-mode), [Ponytail](https://github.com/dietrichgebert/ponytail), and [Principles](https://github.com/agungprasastia/toksave) into your AI agents — without hand-editing configs.

<table>
<tr><td>✔️</td><td><b>Best tools, unified</b> — picks the best token-saving tools and wires them without conflicts</td></tr>
<tr><td>✔️</td><td><b>One command, done</b> — pick your agent, restart, go</td></tr>
<tr><td>✔️</td><td><b>All platforms</b> — macOS, Linux, Windows</td></tr>
<tr><td>✔️</td><td><b>Zero config</b> — everything wired automatically</td></tr>
<tr><td>✔️</td><td><b>Simple updates</b> — <code>toksave update</code> upgrades everything in one shot</td></tr>
<tr><td>✔️</td><td><b>Clean uninstall</b> — reverts every change it made</td></tr>
</table>

## 🤖 Supported Agents

<div align="center">
  <table>
    <tr>
      <td align="center" width="140">
        <img src="assets/agents/claude.jpg" width="56" alt="Claude Code" /><br/>
        <b>Claude Code</b><br/>
        <sub><b style="color:#3fb950">✓ Done</b></sub>
      </td>
      <td align="center" width="140">
        <img src="assets/agents/opencode.png" width="56" alt="OpenCode" /><br/>
        <b>OpenCode</b><br/>
        <sub><b style="color:#3fb950">✓ Done</b></sub>
      </td>
      <td align="center" width="140">
        <img src="assets/agents/codex.jpg" width="56" alt="Codex" /><br/>
        <b>Codex</b><br/>
        <sub><b style="color:#3fb950">✓ Done</b></sub>
      </td>
      <td align="center" width="140">
        <img src="assets/agents/antigravity.png" width="56" alt="Antigravity" /><br/>
        <b>Antigravity</b><br/>
        <sub><b style="color:#3fb950">✓ Done</b></sub>
      </td>
    </tr>
    <tr>
      <td align="center" width="140">
        <img src="assets/agents/copilot.jpg" width="56" alt="GitHub Copilot" /><br/>
        <b>GitHub Copilot</b><br/>
        <sub><b style="color:#3fb950">✓ Done</b></sub>
      </td>
      <td align="center" width="140">
        <img src="assets/agents/droid.png" width="56" alt="Droid" /><br/>
        <b>Droid</b><br/>
        <sub><b style="color:#3fb950">✓ Done</b></sub>
      </td>
    </tr>
  </table>
</div>

Pick one, some, or all:

```bash
toksave                              # interactive: pick agents
toksave --agents claude,opencode     # wire just these
toksave --agents claude,opencode,codex,antigravity,copilot,droid  # all
```

## 📦 What Gets Installed

| Tool | ⭐ | What It Does |
| :--- | :---: | :--- |
| **RTK** | ![](https://img.shields.io/github/stars/rtk-ai/rtk?style=flat-square&label=) | CLI proxy that compresses tool output — **60-90% token savings** |
| **Caveman** | ![](https://img.shields.io/github/stars/JuliusBrussee/caveman?style=flat-square&label=) | Communication mode that cuts LLM response tokens **~75%** |
| **CodeGraph** | ![](https://img.shields.io/github/stars/colbymchenry/codegraph?style=flat-square&label=) | Pre-indexed code knowledge graph — **fewer tool calls** |
| **Context-Mode**| ![](https://img.shields.io/github/stars/mksglu/context-mode?style=flat-square&label=) | MCP server that sandboxes tool output — **98% context reduction** |
| **Ponytail** | ![](https://img.shields.io/github/stars/DietrichGebert/ponytail?style=flat-square&label=) | Forces laziest working solution — YAGNI, stdlib first, delete over add |
| **Principles** | ![](https://img.shields.io/github/stars/agungprasastia/toksave?style=flat-square&label=) | Agent coding standards — think before code, simplicity, surgical edits |

### Wiring per Agent

| Tool | Claude | OpenCode | Codex | Antigravity | Copilot | Droid |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **RTK** | Hook + Allow | Plugin | Hook | Hook + Allow | Hook + Allow | Hook |
| **Caveman** | Plugin + Instruction | Plugin + Instruction | Skill + Instruction | Skill + Instruction | Skill + Instruction | Skill + Instruction |
| **Ponytail** | Plugin + Instruction | Plugin + Instruction | Plugin + Instruction | Plugin + Instruction | Skill + Instruction | Skill + Instruction |
| **CodeGraph** | MCP + Allow + Instruction | MCP + Instruction + Auto-index | MCP + Instruction | MCP + Instruction + Hook | MCP + Instruction + Hook | MCP + Instruction + Hook |
| **Context-Mode** | MCP + Allow + Instruction | Plugin + Instruction | MCP + Instruction + Hook | MCP + Instruction | MCP + Hook + Instruction | MCP + Instruction |

## 🚀 Getting Started

### Prerequisites

- **Node.js ≥ 22** (required for CodeGraph and Context-Mode)
- At least one supported AI agent installed

### Install

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/agungprasastia/toksave/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/agungprasastia/toksave/main/scripts/install.ps1 | iex
```

### Command Reference

| Command | Description |
|---------|-------------|
| `toksave` | Interactive setup — detects agents, installs tools, wires everything |
| `toksave doctor` | Health check with diagnostics and repair suggestions |
| `toksave doctor --fix` | Repair unhealthy tool installations |
| `toksave update` | Update all tools to latest versions |
| `toksave uninstall` | Remove toksave wiring from all agents |
| `toksave disable` | Remove all wire/unwire + owner entries for all agents |
| `toksave index` | Pre-build CodeGraph index in current directory |
| `toksave self-update` | Update the toksave CLI itself |

### Flags

| Flag | Description |
|------|-------------|
| `-a, --agents <ids>` | Target specific agents (e.g., `claude,antigravity`) |
| `-t, --tools <ids>` | Target specific tools (e.g., `rtk,caveman`) |
| `-n, --dry-run` | Preview changes without modifying anything |
| `-y, --yes` | Skip prompts, auto-select detected agents (CI-friendly) |
| `-v, --verbose` | Show detailed output |

Restart agents after install so they pick up new config.

## 🛠️ Development

Built with TypeScript + [Bun](https://bun.sh). Compiles to a standalone binary via `bun build --compile`.

```bash
# Clone and install dependencies
git clone https://github.com/agungprasastia/toksave.git
cd toksave
bun install

# Development tasks
bun run src/index.ts      # Run CLI in dev mode
bun run typecheck         # Run TypeScript checks
bun test                  # Run unit tests (151 tests) 
bun run lint              # Lint with Biome
bun run build             # Build local binary

# Build all cross-platform releases
bash scripts/build-release.sh
```

See [CHANGELOG.md](CHANGELOG.md) for release history and detailed changes.

## 📜 License

Licensed under the MIT License — [see LICENSE](LICENSE).
