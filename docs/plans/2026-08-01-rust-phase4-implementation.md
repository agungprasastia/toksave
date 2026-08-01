# Phase 4: TokSave Rust Rewrite — Hooks, Advanced Commands & Cleanup Implementation Plan

**Goal:** Port remaining 5 hooks (`rtk-hook`, `context-mode-hook`, `codex-perm-hook`, `agy-hook`, `copilot-hook`), 3 advanced commands (`runmcp`, `index`, `self-update`), and perform full repo transition to pure Rust crate.

**Architecture:** Rust 2021 edition, single crate `toksave-rs` (renamed `toksave` at the end). Implement subcommands in `src/commands/hooks/` and `src/commands/`.

**Tech Stack:** `clap`, `tokio`, `serde_json`, `toml_edit`, `colored`.

---

### Task 1: Implement Hooks Interceptors

**Files:**
- Create: `rust/src/commands/hooks/mod.rs`
- Create: `rust/src/commands/hooks/rtk.rs`
- Create: `rust/src/commands/hooks/context_mode.rs`
- Create: `rust/src/commands/hooks/codex_perm.rs`
- Create: `rust/src/commands/hooks/agy.rs`
- Create: `rust/src/commands/hooks/copilot.rs`
- Modify: `rust/src/commands/mod.rs`
- Modify: `rust/src/main.rs`

- [ ] **1. Write tests for hooks in `rust/tests/hooks.rs`**
- [ ] **2. Run tests to verify failure**
- [ ] **3. Implement hook handlers in `rust/src/commands/hooks/`**
- [ ] **4. Connect hook subcommands to `clap` and dispatcher**
- [ ] **5. Run test to verify it passes**
- [ ] **6. Commit**

### Task 2: Implement Special Commands (`runmcp`, `index`, `self-update`)

**Files:**
- Create: `rust/src/commands/runmcp.rs`
- Create: `rust/src/commands/index.rs`
- Create: `rust/src/commands/self_update.rs`
- Modify: `rust/src/commands/mod.rs`

- [ ] **1. Write tests for special commands in `rust/tests/cmd_special.rs`**
- [ ] **2. Run tests to verify failure**
- [ ] **3. Implement `runmcp`, `index`, and `self-update`**
- [ ] **4. Connect to dispatcher**
- [ ] **5. Run tests to verify success**
- [ ] **6. Commit**

### Task 3: TypeScript Cleanup & Crate Rename

**Files:**
- Delete: `src/` (TypeScript directory)
- Modify: `rust/Cargo.toml` (rename `toksave-rs` to `toksave` and version to 1.0.0)
- Move all files from `rust/` to root workspace
- Delete: `rust/` directory

- [ ] **1. Delete TypeScript codebase directory**
- [ ] **2. Move Rust workspace files to root**
- [ ] **3. Update `Cargo.toml` references (rename, paths)**
- [ ] **4. Update CI workflow path references and binary name**
- [ ] **5. Verify `cargo clippy`, `cargo test`, `cargo fmt` pass at repo root**
- [ ] **6. Commit**

### Task 4: Whole-Branch Final Verification & Review

- [ ] **1. Run full test suite and verification**
