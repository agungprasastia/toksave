# Contributing to toksave

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
# Prerequisites: Bun >= 1.1, Node.js >= 22

# Clone the repo
git clone https://github.com/agungprasastia/toksave.git
cd toksave

# Install dependencies
bun install

# Run in dev mode
bun run src/index.ts

# Type check
bun run typecheck

# Run tests
bun test

# Lint + format
bun run lint
bun run format

# Build standalone binary
bun run build
```

## Project Structure

```
src/
├── index.ts          # Entry point
├── cli.ts            # CLI argument parser (commander)
├── registry.ts       # Agent/Tool enums + dispatch
├── agents/           # Per-agent wiring logic
├── tools/            # Per-tool install logic
├── commands/         # CLI commands (init, doctor, update, uninstall, self-update)
├── config/           # JSON/JSONC and TOML config handlers
├── content/          # Embedded content (Caveman SKILL.md, Context-Mode rules)
└── util/             # Shared utilities (paths, download, exec, manifest, etc.)
```

## Adding a New Agent

1. Create `src/agents/<name>.ts` implementing `detect()`, `wire()`, `unwire()`, `verify()`.
2. Add the agent to `ALL_AGENTS` in `src/registry.ts`.
3. Add dispatch entries in `agentModules` in `src/registry.ts`.
4. Add path helpers in `src/util/paths.ts`.
5. Add tests in `src/agents/__tests__/<name>.test.ts`.

## Adding a New Tool

1. Create `src/tools/<name>.ts` implementing `install()`, `installedVersion()`, `latestVersion()`.
2. Add the tool to `ALL_TOOLS` in `src/registry.ts`.
3. Add dispatch entries in `toolModules` in `src/registry.ts`.
4. Add `wire()` cases in each agent module.
5. Add tests in `src/tools/__tests__/<name>.test.ts`.

## Pull Request Process

1. Fork the repo and create a branch from `main`.
2. Make your changes.
3. Run `bun run typecheck && bun test && bun run lint`.
4. Update `CHANGELOG.md` under `[Unreleased]`.
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

- TypeScript strict mode.
- Use `async/await` for all I/O.
- No `any` except config parsing boundaries.
- Explicit return types on exported functions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
