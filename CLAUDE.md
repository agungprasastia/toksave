# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

TokSave is a Bun + TypeScript CLI that installs and wires token-saving tools into AI coding agents. It targets Claude Code, OpenCode, Codex, and Antigravity, and wires RTK, Caveman, CodeGraph, and Context-Mode.

Use Bun for local development. Node.js >= 22 is needed for full install flows involving npm-based tools, especially Context-Mode.

## Commands

```bash
rtk bun install --frozen-lockfile
rtk bun run src/index.ts -- --help
rtk bun run src/index.ts -- doctor --offline
rtk bun run typecheck
rtk bun run lint
rtk bun run lint:fix
rtk bun test
rtk bun test src/__tests__/cli.test.ts
rtk bun test src/__tests__/cli.test.ts -t "doctor command"
rtk bun run build
rtk bun run build:all
rtk bash scripts/build-release.sh
```

CI runs `bun run typecheck`, `bun run lint`, `bun test`, and a `bun build --compile` smoke build on Ubuntu, macOS, and Windows.

## Architecture

- [src/index.ts](src/index.ts) is the executable entrypoint. It parses CLI args, dispatches command modules, then exits with returned status code.
- [src/cli.ts](src/cli.ts) owns Commander setup and maps global flags into `RunOpts`. Default command is `init`.
- [src/registry.ts](src/registry.ts) is the central agent/tool matrix. Add new agents/tools there and provide matching modules with `detect`, `wire`, `unwire`, `verify`, `install`, and version functions.
- [src/commands/](src/commands/) contains user-facing command flows:
  - `init` installs selected tools, detects agents, prompts/auto-selects targets, wires tools, then records manifest entries.
  - `doctor` checks agent wiring and tool versions; `--offline` skips remote latest-version checks.
  - `update` reinstalls tools with upgrade semantics.
  - `uninstall` unwires selected agents/tools and cleans TokSave cache on full removal.
  - `self-update`, `runmcp`, `codex-perm-hook`, and `index` are specialized helper commands.
- [src/agents/](src/agents/) contains per-agent config writers. These modules know each agent's file layout and implement all tool-specific wiring for that agent.
- [src/tools/](src/tools/) contains tool installation/version logic. RTK downloads platform releases or falls back to installer/cargo; CodeGraph and Context-Mode use global npm installs; Caveman is markdown-only.
- [src/util/paths.ts](src/util/paths.ts) centralizes cross-platform home/config/cache paths. Prefer it over inline path construction for agent config locations.
- [src/config/json.ts](src/config/json.ts) and [src/config/toml.ts](src/config/toml.ts) preserve JSONC/TOML config structure while updating agent files.
- [src/content/](src/content/) stores managed markdown blocks for Caveman and Context-Mode. Keep marker comments stable because tests and uninstall use them.
- [src/util/manifest.ts](src/util/manifest.ts) tracks wires in `TOKSAVE_CACHE_DIR` or `~/.cache/toksave/manifest.json`. Tests override `TOKSAVE_CACHE_DIR`.

## Important details

- This repo has [.codegraph/](.codegraph/). Use `rtk codegraph explore "<symbol or question>"` before broad grep/read sweeps.
- TypeScript is strict, ESM, `moduleResolution: "bundler"`, and `noEmit`; Bun runs TypeScript sources directly.
- Biome formats with 2 spaces, double quotes, semicolons, trailing commas, line width 100.
- Version currently lives in both [package.json](package.json) and [src/util/version.ts](src/util/version.ts); keep them in sync for releases.
- `runmcp` proxies MCP server execution and handles Node shebang scripts; keep stdout piping behavior intact.
- Agent wiring edits files under user config directories, not only repo files. Use `--dry-run` flows or temp env/path overrides in tests when possible.
