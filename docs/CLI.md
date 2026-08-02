# TokSave CLI & Command Reference

Complete guide to the `toksave` command-line interface, subcommands, flags, and environment variables.

---

## Command Overview

```bash
toksave [FLAGS] [COMMAND]
```

When run without a subcommand, `toksave` defaults to `init`.

---

## Subcommands

| Command | Purpose | Primary Flags |
| :--- | :--- | :--- |
| **`init`** *(default)* | Installs missing tools and wires configurations into detected AI agents. | `-a`, `-t`, `-n`, `-y`, `-v` |
| **`doctor`** | Checks wiring health, missing binaries, and tool version status. | `--fix`, `--offline`, `-a`, `-t` |
| **`update`** | Reinstalls token-saving tools with upgrade semantics. | `-a`, `-t`, `-n`, `-y`, `-v` |
| **`uninstall`** | Removes TokSave wiring from all or targeted agents/tools. | `-a`, `-t`, `-n`, `-y`, `-v` |
| **`disable`** | Surgical uninstall of specific tool/agent combinations. | `-a`, `-t` |
| **`self-update`** | Updates the `toksave` executable binary itself. | None |
| **`index`** | Builds per-project CodeGraph indexes in the current directory. | `--auto` |

---

## Detailed Command Specifications

### `toksave init`

Installs missing tools, detects installed AI agents, and wires integration hooks/plugins.

```bash
toksave init                       # Interactive: detect agents and prompt for targets
toksave init --yes                 # Non-interactive: auto-wire all detected agents
toksave -a claude,opencode         # Wire only specified agents
toksave -t rtk,context-mode        # Install & wire only specified tools
toksave init --dry-run             # Preview changes without modifying files
```

**Order of Operations:**
1. Filter requested tools (default: all 6).
2. Detect installed agents on the host system.
3. Prompt user for target agents (or auto-select in `--yes` / non-interactive mode). If no agents are detected or selected, **short-circuits with `"Nothing selected."`** before installing tools.
4. Run preflight dependency checks (Node.js ≥ 22 for npm-channel tools, `git` for GitHub release assets).
5. Download & install tools.
6. Wire agent configs and record entries in `manifest.json`.

---

### `toksave doctor`

Performs health checks across all configured agents and tools.

```bash
toksave doctor                     # Full health check with version updates
toksave doctor --offline           # Fast health check (skip remote version probes)
toksave doctor --fix               # Auto-repair broken tool installations
toksave doctor -a claude           # Check only specific agents
```

**What Doctor Probes:**
- **Agent Wiring**: Verifies JSON/TOML configuration blocks and hook paths.
- **Hook Paths**: Checks Windows backslash paths (`\`) that break Git Bash POSIX shells.
- **Binary Resolution**: Validates tool executables exist on system `PATH`.
- **Version Checks**: Probes local version vs. remote npm/GitHub release endpoints.

---

### `toksave update`

Re-downloads and updates tools to their latest release versions.

```bash
toksave update                     # Re-install tools with upgrade flag enabled
toksave update --dry-run           # Preview available tool upgrades
```

---

### `toksave uninstall`

Unwires TokSave configurations from target agents.

```bash
toksave uninstall                  # Remove wiring from all agents
toksave uninstall -a opencode      # Unwire only OpenCode
toksave uninstall -t rtk           # Unwire only RTK across all agents
```

**Clean Unwire Guarantee:**
- Removes instruction blocks (`<!-- TOKSAVE:START --> ... <!-- TOKSAVE:END -->`).
- Prunes empty JSON objects (`{}`) and TOML tables (`[mcp_servers]`).
- Deletes config files if they contain no other user configuration (preserving `$schema` keys).

---

### `toksave index`

Triggers local CodeGraph index generation for the current working directory.

```bash
toksave index                      # Manual index generation
toksave index --auto               # Silent background mode (triggered by agent hooks)
```

---

## Global Flags

| Flag | Long Flag | Description |
| :--- | :--- | :--- |
| `-a` | `--agents <list>` | Target specific agents (comma-separated: `claude,opencode,codex,antigravity,copilot,droid,devin,warp`). |
| `-t` | `--tools <list>` | Target specific tools (comma-separated: `rtk,caveman,codegraph,context-mode,ponytail,principles`). |
| `-n` | `--dry-run` | Preview actions without modifying filesystem. |
| `-y` | `--yes` | Non-interactive mode (auto-accept prompts). |
| `-v` | `--verbose` | Print verbose execution logs. |
| `-h` | `--help` | Display command help message. |
| `-V` | `--version` | Print TokSave version. |

---

## Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `HOME` / `USERPROFILE` | User home directory for resolving configuration paths. | Platform home |
| `LOCALAPPDATA` | Windows local app data directory for tool binaries. | `%USERPROFILE%\AppData\Local` |
| `TOKSAVE_CACHE_DIR` | Directory storing manifest data and temporary downloads. | `~/.cache/toksave` |
| `TOKSAVE_TEST` | Test mode override (bypasses interactive prompts & downloads). | Unset |

---

## Exit Codes

- `0`: Success / all targets wired without error.
- `1`: Partial failure or command error (e.g., tool installation failure).
