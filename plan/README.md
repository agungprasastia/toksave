# Tokless → TokSave Parity Implementation Plan

> **Untuk**: Mengimplementasi semua fitur Tokless (Go) ke TokSave (Bun + TS).
> Source: `D:\KULIAH\token\TokSave\tokless\` (Go, github/HoangP8/tokless)
> Target: `D:\KULIAH\token\TokSave\` (TS + Bun)

## Gap Analysis

### Agents
| Agent | Tokless | TokSave | Gap |
|-------|---------|---------|-----|
| claude | ✅ | ✅ | fitur wiring beda |
| opencode | ✅ | ✅ | fitur wiring beda |
| codex | ✅ | ✅ | fitur wiring beda |
| antigravity | ✅ | ✅ | fitur wiring beda |
| copilot | ✅ | ❌ | **belum ada sama sekali** |
| droid (factory) | ✅ | ❌ | **belum ada sama sekali** |

### Tools
| Tool | Tokless | TokSave | Gap |
|------|---------|---------|-----|
| rtk | ✅ | ✅ | install+hook beda |
| caveman | ✅ | ✅ | wiring beda |
| codegraph | ✅ | ✅ | hook index beda |
| context-mode | ✅ | ✅ | wiring beda |
| ponytail | ✅ | ❌ | **belum ada** |
| principles | ✅ | ❌ | **belum ada (instruction-only)** |

### Commands
| Command | Tokless | TokSave | Gap |
|---------|---------|---------|-----|
| init | ✅ | ✅ (partial) | progress bar, banner, self-update check |
| update | ✅ | ✅ (partial) | version diff, selective upgrade, re-sync wiring |
| doctor | ✅ | ✅ (partial) | version probing, repo footer |
| index | ✅ | ✅ (partial) | auto vs manual, hook variant |
| uninstall | ✅ | ✅ | surgical vs all, purge binaries |
| disable | ✅ alias | ❌ | **belum ada** (surgical uninstall per-tool+agent) |
| self-update | ✅ | ✅ | spinner, re-exec after update |
| rtk-hook | ✅ | ✅ (partial) | antigravity/copilot/droid/claude/codex variants |
| codex-perm-hook | ✅ | ✅ (partial) | |
| context-mode-hook | - | ✅ | tokless gak punya explicit ini, tapi ada codex hook variant |
| runmcp | ✅ | ✅ (partial) | agent-aware index pre-run, mcp_proxy |

### Core Architecture Difference — Instruction Wiring

**Paling penting:**

TokSave saat ini pakai per-tool HTML comment fences:
- `<!-- CAVEMAN_START -->...<!-- CAVEMAN_END -->`
- `<!-- CONTEXT-MODE_START -->...`
- `<!-- RTK_START -->...`

Tokless pakai **unified body** system:
- Satu file `AGENTS.md` / `CLAUDE.md` / etc punya sections dari owners
- `tokless_block.go` adalah unified block manager:
  - `WriteOwner(agent, owner)` → render full body dari owner list
  - `RemoveOwner(agent, owner)` → hapus owner, re-render
  - `HasOwner(agent, owner)` → cek owner exists
  - Legacy fences cleanup: `<!-- caveman-begin -->`, `<!-- CODEGRAPH_START -->`, dll
  - `EnsureInstructionSeparators` → normalize blank lines antar blocks (exactly 2 newlines)
  - Conflict handling: `ConfigureInstructionConflicts(autoAppend)` — kalo file sudah ada content non-tokless, prompt user overwrite vs append
  - `ToklessOwners` order: `principles, caveman, ponytail, codegraph, context-mode`
  - `ToklessAgentBody(owners)` render full markdown dengan index + sections

Ini fundamental — harus diimplementasi dulu.

### Content Difference

Tokless punya `internal/util/agent_instructions.md` — single source template yang di-split per owner:
- Principles, caveman, ponytail, codegraph, context-mode semua dari 1 file
- `instructionIndexSection()` → header overview kalo >=2 owners
- TokSave sekarang punya terpisah: `caveman-skill.ts`, `ctx-rules.ts`, `rtk-rules.ts`

Tokless render body juga include repo footer, progress UI, color system lebih lengkap.

### Other Missing

- **Progress UI**: tokless punya tree progress (section progress, root progress), spinner (`runStatus`), colored tree leaves
- **Logger**: `util.L` structured logger, verbose level, raw vs styled
- **Paths**: lebih lengkap (copilot, droid, antigravity MCP), `WindowsHomeFromWSL`, `selfHealPath`
- **Deps checking**: `EnsureDeps(needNode, needGit, minNode)` centralized
- **Binary resolution**: `ResolveRtkBin`, `ResolveCodegraphBin`, `ResolveNpmBinary`, etc dengan self-heal
- **Npm install**: retry strategies, fallback tarball, path ensure
- **MCP spawn**: `PickMcpSpawn`, `WrapAutoIndex`, `McpSpawn` struct
- **Auto-index unwire**: cleanup `tokless index --auto` SessionStart hooks
- **Codex approval**: `applyCodexApprovalPolicy` in config.toml

## Implementation Order (Dependency)

```
Phase 1: Foundation — unified block system
  └→ content: agent_instructions.md template, ToklessOwners, render body
  └→ util: unified block manager (WriteOwner/RemoveOwner/HasOwner)
  └→ util: separator normalizer
  └→ util: instruction conflict handling

Phase 2: New Agents (copilot + droid)
  └→ paths: Copilot + Droid paths
  └→ agents/copilot.ts — full MCP + hooks + skills + IDE sync
  └→ agents/droid.ts — hooks + MCP + skills

Phase 3: New Tools (ponytail + principles)
  └→ tools/ponytail.ts — npm + plugin install
  └→ content: principles template
  └→ tools/principles.ts — instruction-only
  └→ registry: add ponytail + principles + update agent wiring

Phase 4: Upgrade existing tools to parity
  └→ caveman: marketplace.add + plugin install + skills relocate
  └→ rtk: platform binary (prebuilt + cargo fallback), hook override
  └→ codegraph: WrapAutoIndex, VerifyFor, IndexProject background sync
  └→ context-mode: full wiring parity (codex bounded, cache clean, etc)

Phase 5: Commands parity
  └→ disable command (surgical uninstall)
  └→ index: auto vs manual, hook variant agy/copilot
  └→ doctor: version probing, repo footer, tree display
  └→ update: version diff table, selective update, resyncWiring
  └→ uninstall: purge binaries, surgical per-tool/agent select
  └→ self-update: spinner + reexec

Phase 6: Util infrastructure parity
  └→ progress tree, colors extended, deps check, paths completeness
  └→ mcp spawn system
  └→ runmcp proxy with pre-index
  └→ rtk-hook copilot/droid variants
```

---

## Plan files

- `plan/001-foundation-unified-block.md` — unified body rendering, WriteOwner/RemoveOwner, separator
- `plan/002-agents-copilot-droid.md` — new agents GitHub Copilot + Factory Droid
- `plan/003-tools-ponytail-principles.md` — ponytail + principles tools
- `plan/004-tools-upgrade-parity.md` — upgrade caveman/rtk/codegraph/context-mode
- `plan/005-commands-parity.md` — disable, index, doctor, update, uninstall, self-update
- `plan/006-util-infrastructure.md` — progress, colors, deps, paths, npm, mcp spawn

## Key Decisions

- Keep TS/Bun stack (don't switch to Go) — port logic, not language
- Unified block system = single source of truth template (import agent_instructions.md concept)
- Backward compat: support removing old HTML fence blocks on migration (stripLegacy)
- Agent detection: keep existing + add copilot/droid
- Content: agent_instructions.md equivalent in TS (one file split per owner)
