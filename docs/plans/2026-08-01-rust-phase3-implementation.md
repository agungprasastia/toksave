# Phase 3: TokSave Rust Rewrite — Tools, Agents & Core Management Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement remaining 5 Tools, 7 Agents, and 4 core management commands (`doctor`, `update`, `uninstall`, `disable`) in `toksave-rs`, completing Phase 3 of the Rust rewrite with 100% test coverage.

**Architecture:** Crate `toksave-rs` in `rust/`. Extend `src/tools/`, `src/agents/`, `src/content/`, and `src/commands/`. All agents and tools register in `src/registry.rs`. Standardized error handling via `ToksaveError`.

**Tech Stack:** Rust 2021 edition, `tokio`, `serde`/`serde_json`, `reqwest`, `colored`, `clap`, `fs2`.

## Global Constraints

- Must compile clean with zero warnings under `cargo clippy --all-targets -- -D warnings`.
- Code must pass `cargo fmt --check`.
- All tests must pass with `cargo test`.
- Trust Boundaries: Unparseable JSON/TOML files must throw error, never overwrite defaults. Multi-file writes must be atomic. Lock file `manifest.json.lock` must be acquired on manifest modifications.

---

### Task 1: Managed Content Modules

**Files:**
- Create: `rust/src/content/mod.rs`
- Create: `rust/src/content/agent_instructions.rs`
- Create: `rust/src/content/caveman_skill.rs`
- Create: `rust/src/content/ctx_rules.rs`
- Modify: `rust/src/lib.rs`

**Interfaces:**
- Consumes: None
- Produces: `content::AGENT_INSTRUCTIONS`, `content::CAVEMAN_SKILL`, `content::CTX_RULES` static string constants / functions.

- [ ] **Step 1: Write failing test in `rust/tests/content.rs`**

```rust
use toksave::content;

#[test]
fn test_managed_content_constants_not_empty() {
    assert!(!content::agent_instructions().is_empty());
    assert!(!content::caveman_skill().is_empty());
    assert!(!content::ctx_rules().is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test content`
Expected: FAIL (modules do not exist)

- [ ] **Step 3: Implement content modules in `rust/src/content/`**

Create `rust/src/content/agent_instructions.rs`:
```rust
pub fn agent_instructions() -> &'static str {
    include_str!("../../../src/content/agent-instructions.ts")
}
```
Or define exact content string constants matching TS source.

Export in `rust/src/content/mod.rs` and `rust/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test content`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/content rust/src/lib.rs rust/tests/content.rs
git commit -m "feat(rust): add managed content modules"
```

---

### Task 2: 5 Remaining Tools (`caveman`, `codegraph`, `context_mode`, `ponytail`, `principles`)

**Files:**
- Create: `rust/src/tools/caveman.rs`
- Create: `rust/src/tools/codegraph.rs`
- Create: `rust/src/tools/context_mode.rs`
- Create: `rust/src/tools/ponytail.rs`
- Create: `rust/src/tools/principles.rs`
- Modify: `rust/src/tools/mod.rs`
- Modify: `rust/src/registry.rs`

**Interfaces:**
- Consumes: `Tool` trait in `src/tools/mod.rs`
- Produces: `CavemanTool`, `CodegraphTool`, `ContextModeTool`, `PonytailTool`, `PrinciplesTool` implementing `Tool`.

- [ ] **Step 1: Write failing tests in `rust/tests/tools_phase3.rs`**

```rust
use toksave::tools::*;
use toksave::util::health::HealthStatus;

#[tokio::test]
async fn test_caveman_tool_health() {
    let tool = CavemanTool::new();
    let health = tool.health_check().unwrap();
    assert!(matches!(health, HealthStatus::Healthy { .. }));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tools_phase3`
Expected: FAIL

- [ ] **Step 3: Implement remaining tools in `rust/src/tools/` and register in `src/registry.rs`**

Implement each struct implementing `Tool` trait, matching TS behavior:
- `caveman`: static prompt tool.
- `codegraph`: binary/node runner tool.
- `context_mode`: rule injector tool.
- `ponytail`: skill injector tool.
- `principles`: prompt injector tool.

Register all in `Registry::all_tools()`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test tools_phase3`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/tools rust/src/registry.rs rust/tests/tools_phase3.rs
git commit -m "feat(rust): implement remaining 5 tools"
```

---

### Task 3: 7 Remaining Agents (`opencode`, `codex`, `antigravity`, `copilot`, `droid`, `devin`, `warp`)

**Files:**
- Create: `rust/src/agents/opencode.rs`
- Create: `rust/src/agents/codex.rs`
- Create: `rust/src/agents/antigravity.rs`
- Create: `rust/src/agents/copilot.rs`
- Create: `rust/src/agents/droid.rs`
- Create: `rust/src/agents/devin.rs`
- Create: `rust/src/agents/warp.rs`
- Modify: `rust/src/agents/mod.rs`
- Modify: `rust/src/registry.rs`

**Interfaces:**
- Consumes: `Agent` trait in `src/agents/mod.rs`
- Produces: `OpencodeAgent`, `CodexAgent`, `AntigravityAgent`, `CopilotAgent`, `DroidAgent`, `DevinAgent`, `WarpAgent` implementing `Agent`.

- [ ] **Step 1: Write failing tests for agent detection and wiring safety in `rust/tests/agents_phase3.rs`**

```rust
use toksave::agents::*;

#[test]
fn test_warp_corrupted_config_fails() {
    let agent = WarpAgent::new();
    // Test corrupt json error
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test agents_phase3`
Expected: FAIL

- [ ] **Step 3: Implement remaining agents in `rust/src/agents/`**

- Support config atomic writes & strict JSON/TOML parsing.
- Register all 7 agents in `Registry::all_agents()`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test agents_phase3`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/agents rust/src/registry.rs rust/tests/agents_phase3.rs
git commit -m "feat(rust): implement remaining 7 agents with strict config parsing"
```

---

### Task 4: Full Agent x Tool Matrix & Integration Suite

**Files:**
- Modify: `rust/tests/matrix.rs`

**Interfaces:**
- Consumes: All 8 Agents and 6 Tools from `Registry`.
- Produces: Verified 48 Agent × Tool matrix test execution.

- [ ] **Step 1: Write full 8x6 matrix integration test in `rust/tests/matrix.rs`**

```rust
use toksave::registry::Registry;
use toksave::util::paths::TestEnv;

#[tokio::test]
async fn test_full_agent_tool_matrix() {
    let env = TestEnv::new();
    let registry = Registry::new();
    for agent in registry.agents() {
        for tool in registry.tools() {
            // wire, verify, unwire, verify
        }
    }
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --test matrix`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add rust/tests/matrix.rs
git commit -m "test(rust): add full 8x6 agent-tool matrix integration test"
```

---

### Task 5: `doctor` Command Implementation

**Files:**
- Create: `rust/src/commands/doctor.rs`
- Modify: `rust/src/commands/mod.rs`
- Modify: `rust/src/cli.rs`
- Modify: `rust/src/main.rs`

**Interfaces:**
- Consumes: `Registry`, `Agent::verify()`, `Tool::health_check()`, `Tool::repair()`
- Produces: `commands::doctor::run(opts)` returning `Result<i32, ToksaveError>`.

- [ ] **Step 1: Write failing test in `rust/tests/cmd_doctor.rs`**

```rust
use toksave::util::paths::TestEnv;

#[tokio::test]
async fn test_doctor_command_offline() {
    let env = TestEnv::new();
    // execute doctor --offline
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_doctor`
Expected: FAIL

- [ ] **Step 3: Implement `doctor` command in `rust/src/commands/doctor.rs`**

Implement health checks table, `--offline` flag, `--fix` flag. Wire up in `main.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_doctor`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/commands/doctor.rs rust/src/commands/mod.rs rust/src/cli.rs rust/src/main.rs rust/tests/cmd_doctor.rs
git commit -m "feat(rust): implement doctor command with fix and offline support"
```

---

### Task 6: `update` Command Implementation

**Files:**
- Create: `rust/src/commands/update.rs`
- Modify: `rust/src/commands/mod.rs`

**Interfaces:**
- Consumes: `Registry`, `Tool::latest_version()`, `Tool::install()`, `Manifest`
- Produces: `commands::update::run(opts)` returning `Result<i32, ToksaveError>`.

- [ ] **Step 1: Write failing test in `rust/tests/cmd_update.rs`**

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_update`
Expected: FAIL

- [ ] **Step 3: Implement `update` command with parallel tool downloads**

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_update`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/commands/update.rs rust/src/commands/mod.rs rust/tests/cmd_update.rs
git commit -m "feat(rust): implement update command with parallel tool updates"
```

---

### Task 7: `uninstall` Command Implementation

**Files:**
- Create: `rust/src/commands/uninstall.rs`
- Modify: `rust/src/commands/mod.rs`

**Interfaces:**
- Consumes: `Registry`, `Agent::unwire()`, `Manifest`
- Produces: `commands::uninstall::run(opts)` returning `Result<i32, ToksaveError>`.

- [ ] **Step 1: Write failing test in `rust/tests/cmd_uninstall.rs`**

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_uninstall`
Expected: FAIL

- [ ] **Step 3: Implement `uninstall` command**

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_uninstall`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/commands/uninstall.rs rust/src/commands/mod.rs rust/tests/cmd_uninstall.rs
git commit -m "feat(rust): implement uninstall command"
```

---

### Task 8: `disable` Command Implementation

**Files:**
- Create: `rust/src/commands/disable.rs`
- Modify: `rust/src/commands/mod.rs`

**Interfaces:**
- Consumes: `Registry`, `Agent::unwire()`, `Manifest`
- Produces: `commands::disable::run(opts)` returning `Result<i32, ToksaveError>`.

- [ ] **Step 1: Write failing test in `rust/tests/cmd_disable.rs`**

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_disable`
Expected: FAIL

- [ ] **Step 3: Implement `disable` command**

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_disable`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/src/commands/disable.rs rust/src/commands/mod.rs rust/tests/cmd_disable.rs
git commit -m "feat(rust): implement disable command"
```

---

### Task 9: Phase 3 Verification & Formatting Check

- [ ] **Step 1: Run whole workspace tests, fmt, and clippy**

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

- [ ] **Step 2: Verify zero warnings and 100% passing tests**

- [ ] **Step 3: Commit final Phase 3 status update**
