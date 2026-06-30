# toksave

**Zero-config token-saver for AI coding agents.**

Install and wire [RTK](https://github.com/rtk-ai/rtk), [Caveman](https://github.com/JuliusBrussee/caveman), [CodeGraph](https://github.com/colbymchenry/codegraph), and [Context-Mode](https://github.com/mksglu/context-mode) into your AI agents — without hand-editing configs.

<p align="center">
  <img src="assets/Logo.png" alt="toksave logo" width="300" />
</p>

## Supported Agents

| Agent | MCP | Hooks | Caveman | Context-Mode Rules |
|-------|-----|-------|---------|-------------------|
| Claude Code | ✅ | ✅ RTK bash | ✅ SKILL.md | ✅ AGENTS.md |
| OpenCode | ✅ | — | ✅ AGENTS.md | ✅ AGENTS.md |
| Codex | ✅ (TOML) | ✅ RTK hook | ✅ instructions.md | ✅ instructions.md |
| Antigravity | ✅ (multi-surface) | ✅ RTK + ctx | ✅ SKILL.md | ✅ AGENTS.md |

## Install

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/agungprasastia/toksave/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/agungprasastia/toksave/main/scripts/install.ps1 | iex
```

## Usage

```bash
# Interactive setup — detect agents, install tools, wire everything
toksave

# Target specific agents
toksave --agents claude,antigravity

# Target specific tools
toksave --tools rtk,caveman

# Non-interactive mode (CI/scripts)
toksave --yes

# Dry run — see what would happen
toksave --dry-run

# Verbose output
toksave --verbose
```

## Commands

| Command | Description |
|---------|-------------|
| `toksave` | Install + wire all tools into detected agents |
| `toksave doctor` | Health check — show what's wired and what's broken |
| `toksave update` | Update all tools to latest versions |
| `toksave uninstall` | Remove toksave wiring from agents |
| `toksave self-update` | Update the toksave CLI itself |

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--agents <ids>` | `-a` | Target specific agents (comma-separated) |
| `--tools <ids>` | `-t` | Target specific tools (comma-separated) |
| `--dry-run` | `-n` | Preview changes without modifying anything |
| `--verbose` | `-v` | Show detailed output |
| `--yes` | `-y` | Skip prompts, auto-select detected agents |

## What Gets Installed

| Tool | Method | What It Does |
|------|--------|-------------|
| **RTK** | Prebuilt binary from GitHub | CLI proxy that compresses tool output — 60-90% token savings |
| **Caveman** | Markdown skill file | Communication mode that cuts LLM response tokens ~75% |
| **CodeGraph** | `npm install -g` | Pre-indexed code knowledge graph — fewer tool calls |
| **Context-Mode** | `npm install -g` + rules | MCP server that sandboxes tool output — 98% context reduction |

## Idempotent & Safe

- Run `toksave` multiple times — no duplicate entries
- Tracks what it wired via `~/.cache/toksave/manifest.json`
- `toksave uninstall` only removes what toksave added
- `--dry-run` previews all changes before committing

## Prerequisites

- **Node.js ≥ 22** (for CodeGraph and Context-Mode)
- At least one supported agent installed

## Development

Built with TypeScript + [Bun](https://bun.sh). Compiles to standalone binary via `bun build --compile`.

```bash
# Install dependencies
bun install

# Run in dev
bun run src/index.ts

# Type check
bun run typecheck

# Build binary
bun run build

# Build all platforms
bash scripts/build-release.sh
```

## License

MIT — see [LICENSE](LICENSE).
