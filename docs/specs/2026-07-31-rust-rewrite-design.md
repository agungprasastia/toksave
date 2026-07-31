# TokSave Rust Rewrite — Design

Date: 2026-07-31
Branch: `rust-rewrite`
Status: Approved

## Goal

Rewrite TokSave (currently Bun + TypeScript CLI) in Rust for feature-parity across all
13 commands (default `init` + 12 explicit), with full integration-test coverage. Primary
motivation: produce a small, statically-linked, zero-runtime-dependency binary (Bun's
embedded runtime makes the current `--compile` binary ~95 MB).

## Decisions (from brainstorming)

- **Scope**: feature-parity penuh — all 13 commands: `init` (default), `doctor`,
  `update`, `uninstall`, `disable`, `self-update`, `codex-perm-hook`, `rtk-hook`,
  `context-mode-hook`, `runmcp`, `index`, `agy-hook`, `copilot-hook`. (`build-index.ts`
  and `codegraph-index-hook.ts` are internal modules of the `index` command, not
  separate commands.)
- **Crates**: popular ecosystem crates allowed — `tokio` (full), `clap` (derive),
  `serde`/`serde_json`, `smol-toml`/`toml`, `reqwest` (rustls, async), `flate2`+`tar`,
  `zip`, `semver`, `fs2`, `dirs`, `colored`, `inquire` (interactive prompts).
- **I/O model**: async (`tokio` + `reqwest` async). Parallel downloads during `update`;
  config writes stay serial for atomicity.
- **Platform targets**: 5 — linux-x64, linux-arm64, darwin-x64, darwin-arm64,
  windows-x64 (parity with current release).
- **Project structure**: single crate (not a workspace).
- **TS coexistence**: TypeScript stays at repo root during the port; Rust lives in a
  separate `rust/` dir to avoid `src/` collisions. TS removed in the final step.
- **Testing**: full integration tests, bun-test-style, against the compiled binary in
  temp dirs with env overrides (`HOME`/`USERPROFILE`/`APPDATA`/`LOCALAPPDATA`).
- **Porting order**: approach A — command-by-command, each command verified by
  ported tests before moving on.

## Architecture

### Binary and layout

Single crate `toksave`:

```
rust/
├── Cargo.toml
├── src/
│   ├── main.rs            # entry: parse args → dispatch → exit code
│   ├── lib.rs             # pub mod tree
│   ├── cli.rs             # clap Command definition (13 commands + flags)
│   ├── commands/
│   │   ├── init.rs
│   │   ├── doctor.rs
│   │   ├── update.rs
│   │   ├── uninstall.rs
│   │   ├── disable.rs
│   │   ├── self_update.rs
│   │   ├── runmcp.rs
│   │   ├── index.rs          # incl. build-index + codegraph-index-hook logic
│   │   └── hooks/          # rtk-hook, context-mode-hook, codex-perm-hook,
│   │                       # agy-hook, copilot-hook
│   ├── agents/
│   │   ├── mod.rs         # Agent trait + registry
│   │   ├── claude.rs, opencode.rs, codex.rs, antigravity.rs,
│   │   ├── copilot.rs, droid.rs, devin.rs, warp.rs
│   ├── tools/
│   │   ├── mod.rs         # Tool trait + registry
│   │   ├── rtk.rs, caveman.rs, codegraph.rs, context_mode.rs,
│   │   ├── ponytail.rs, principles.rs
│   ├── util/
│   │   ├── paths.rs       # cross-platform home/config/cache dirs
│   │   ├── download.rs    # reqwest + retry/backoff, checksum, tar/zip extract
│   │   ├── json.rs        # serde_json + JSONC-preserving writer
│   │   ├── toml.rs
│   │   ├── manifest.rs    # manifest.json tracking (lock + atomic write)
│   │   ├── version.rs     # semver compare
│   │   ├── errors.rs      # error enum + remediation
│   │   └── colors.rs      # terminal output
│   ├── content/           # managed markdown blocks (RTK, caveman, context-mode)
└── tests/
    ├── common/mod.rs      # TestEnv helper
    ├── cli.rs
    ├── agents.rs
    ├── tools.rs
    ├── commands.rs
    ├── hooks.rs
    └── util.rs
```

### Core traits (mirror TS registry)

```rust
trait Agent {
    fn detect(&self) -> Detection;
    fn wire(&self, tool: &ToolId, opts: &RunOpts) -> Result<bool, ToksaveError>;
    fn unwire(&self, tool: &ToolId, opts: &RunOpts) -> Result<bool, ToksaveError>;
    fn verify(&self, tool: &ToolId) -> Option<bool>;
}

trait Tool {
    async fn install(&self) -> Result<(), ToksaveError>;
    fn installed_version(&self) -> Option<String>;
    async fn latest_version(&self) -> Result<String, ToksaveError>;
    fn health_check(&self) -> Result<HealthStatus, ToksaveError>;
    async fn repair(&self) -> Result<RepairResult, ToksaveError>;
}
```

The agent×tool matrix stays a single dispatch point in the registry, matching the TS
design.

### Data model

```rust
struct RunOpts { dry_run: bool, upgrade: bool, verbose: bool, yes: bool }
struct Detection { installed: bool, source: String }
enum ToolId { Rtk, Caveman, Codegraph, ContextMode, Ponytail, Principles }
enum AgentId { Claude, Opencode, Codex, Antigravity, Copilot, Droid, Devin, Warp }
```

## Command flows

### Entry

`main()` → `Cli::parse()` → `match command` → `run_command()` → `i32` exit code.
All commands return an exit code, 0 on success, non-zero on failure (same as TS).

### Init (primary command — exercises nearly every subsystem)

```
init(args)
 ├─ detect_agents()        → scan PATH/desktop/config dirs per agent → Detection
 ├─ resolve_targets()      → --agents/--tools flags, or interactive prompt (inquire)
 │                          → / --yes auto-select detected agents
 ├─ for each tool:  install_tool(tool) → download/extract/install → health_check
 ├─ for each agent × tool:
 │    agent.wire(tool, opts) → write config (JSON/TOML/AGENTS.md/MCP) atomically
 │    agent.verify(tool)     → confirm actually wired
 │    manifest.record(tool, agent)
 └─ summary output
```

### Doctor

```
doctor(opts)
 ├─ detect agents
 ├─ for each wired agent×tool: verify() → status
 ├─ for each tool: health_check() → installed? version? outdated? (--offline skips remote)
 ├─ --fix: tool.repair() for unhealthy tools
 └─ table output (cli-table3 equivalent)
```

### Update / Uninstall / Disable

Same pattern: resolve targets → loop agent×tool → wire/unwire/verify → manifest.

### Concurrency

- Parallel: tool downloads during `update`.
- Serial: config writes (atomicity across a batch).

### Config write atomicity

Every config write: temp file → rename. Multi-file batches (e.g. Antigravity
`wireMcp`) write all temp files, then commit; roll back all already-written files if
any commit fails.

## Error handling

```rust
enum ToksaveError {
    Tool,        // tool install/run failure
    Install,     // install-specific, with remediation
    Download,    // HTTP failure, includes status code
    Network,     // connectivity lost
    HealthCheck, // health check failure
    Integrity,   // checksum mismatch
    Platform,    // unsupported OS/arch
    Config,      // JSON/TOML parse failure
    Io,          // filesystem
}
```

Each variant carries: message, `context: String`,
`remediation: Option<String>`, `source: Option<Box<dyn Error>>`. `Display` renders a
user-friendly message; `impl From<io::Error>` etc. without excessive boilerplate.

### Trust boundaries (never skipped)

1. **Config parse safety**: unparseable `settings.json`/`hooks.json`/`mcp.json` →
   **error, not `{}` fallback**. Never overwrite a user's config (Warp bug fix from PR
   #22 must be preserved).
2. **Tar/zip path traversal**: validate every entry path — reject absolute paths,
   `../`, escapes from destDir.
3. **Checksum verification**: RTK download verified SHA256 after download.
4. **Retry with backoff**: 3 retries (1s, 2s, 4s), 10s timeout per fetch.
5. **Manifest atomicity**: `manifest.json.lock` + temp-write-rename.
6. **Multi-file rollback**: batch agent writes — all temp → commit → rollback on failure.
7. **runmcp**: spawn child with expanded PATH (tool bin dirs + node), pipe stdio,
   resolve shebang (node script vs direct binary), validate tool name.

## Testing

- Framework: `cargo test`, integration tests against the compiled binary.
- `tests/common/mod.rs` provides `TestEnv` — temp dirs + env overrides
  (`HOME`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`); `paths.rs` reads these env vars
  on all platforms so tests can override.
- Coverage:
  1. `cli`: parsing all 13 commands + flags.
  2. `agents` matrix (8 agents × 6 tools): wire→verify→unwire→verify in temp env,
     plus Warp hooks.json safety regression (2 tests from PR #22).
  3. `tools`: install/installed_version/latest_version/health_check — network via
     local mock server (tiny_http/httptest), or offline checks. No real internet in tests.
  4. `commands`: init end-to-end, doctor (--offline/--fix), update/uninstall/disable,
     index, runmcp.
  5. `util`: paths, semver, manifest atomicity, JSONC-preserving config parse,
     download retry.
  6. `hooks`: rtk-hook stdin→stdout piping, context-mode-hook proxy.
- **Parity gate**: a command is "ported" only when its relevant TS behavior tests are
  re-ported to Rust and pass. Compiling is not enough.

## Coexistence, build, release, transition

### Coexist TS + Rust

- TS stays at repo root (`src/`); Rust crate lives in `rust/`.
- Binary name during coexistence: `toksave-rs` (avoids npm bin collision). Renamed to
  `toksave` when TS is removed.

### Build & release

- `cargo build --release` → single static binary.
- Cross-compile 5 targets in GitHub Actions:
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`,
  `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, packaged as `.tar.gz`/`.zip`.
- `release.yml` gains a Rust job producing artifacts; release notes still from CHANGELOG.
- Install scripts (curl|bash, irm|iex) download the Rust binary instead of npm.

### Transition steps (commit order on `rust-rewrite`)

1. Scaffold crate `rust/` (Cargo.toml, clap CLI with 13 command stubs).
2. Port `init` + registry + 1 agent (Claude) + 1 tool (RTK) → integration test PASS.
3. Extend to all agents & tools.
4. Port `doctor`, `update`, `uninstall`, `disable`.
5. Port hooks + `runmcp` + `index` + `self-update`.
6. Full parity check (all relevant TS tests ported and passing).
7. Remove TS, rename binary `toksave-rs` → `toksave`, move `rust/` → repo root,
   update CI/release/install scripts.
8. Tag release v1.0.0 (rewrite = major bump).

### CI

`ci.yml` gains a `rust-check` job: `cargo fmt --check`, `cargo clippy -D warnings`,
`cargo test`, on 3 OS. TS job stays during coexistence.
