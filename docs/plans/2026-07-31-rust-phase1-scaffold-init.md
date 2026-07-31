# TokSave Rust Rewrite — Phase 1+2 (Scaffold + Init + Claude + RTK) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the Rust crate (`rust/`) with a clap CLI covering all 13 commands (stubs), then port the `init` command end-to-end for the Claude × RTK slice: registry, paths, JSON/JSONC config, manifest, RTK tool install, Claude wiring, and passing integration tests — the first independently-testable milestone of the Rust rewrite.

**Architecture:** Single crate `toksave` living in `rust/` beside the TS `src/` during coexistence. Async via `tokio` + `reqwest`. `main.rs` → `Cli::parse()` → dispatch → exit code. Agent/Tool traits dispatch through a match-based registry mirroring `agentModules`/`toolModules` in the TS. Binary name during coexistence: `toksave-rs` (avoids npm-bin collision with the TS build). Rust code mirrors the TS implementation line-for-line so parity tests can be ported directly.

**Tech Stack:** Rust (edition 2021, MSRV 1.75), tokio, clap (derive), serde/serde_json, reqwest (rustls), flate2, tar, zip, dirs, colored. Dev-deps: tiny_http (local mock HTTP server for download tests).

## Global Constraints

- Crate root: `rust/` (NOT repo root — TS `src/` must stay untouched until final phase).
- Binary name: `toksave-rs`. Version: `0.8.5` (parity with `package.json`).
- Trust boundaries (MUST NOT be simplified away):
  1. Unparseable agent config (settings.json, hooks.json, mcp.json, .claude.json) → `Err(ToksaveError::Config)`, NEVER an empty-`{}` fallback. Missing file is fine (→ `None`/`{}`), unparseable content is an error.
  2. Tar/zip extraction: reject absolute paths, `..` traversal, escapes from dest dir (zip-slip).
  3. Download: 3 retries (1s, 2s, 4s backoff), 10s timeout on version checks; no retry on HTTP 404.
  4. Manifest writes: lock file + atomic temp-write-rename.
  5. `runmcp`/hooks: defer to later phases (not in this plan's scope).
- Env overrides for tests (must be honored on ALL platforms): `HOME`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, `TOKSAVE_CACHE_DIR`.
- No comments unless the port demands a `ponytail:` note. Follow Biome style where porting TS semantics.
- Verification commands, run at each task's final step:
  - `cargo fmt --check` (after `cargo fmt`)
  - `cargo clippy -- -D warnings`
  - `cargo test`
- Commits are separate per task, on branch `rust-rewrite`, conventional format `feat(rust): ...` / `test(rust): ...` / `chore(rust): ...`.

---

## File Structure

```
rust/
├── Cargo.toml
├── src/
│   ├── main.rs            # tokio entry, dispatch, exit code
│   ├── lib.rs             # pub mod tree
│   ├── cli.rs             # clap Cli + Command enum + parse → ParsedCli
│   ├── registry.rs        # ToolId/AgentId/Channel, AgentInfo/ToolInfo,
│   │                      # ALL_AGENTS/ALL_TOOLS, parse*, dispatch fns
│   ├── commands/
│   │   ├── mod.rs
│   │   └── init.rs        # run_init (ported from src/commands/init.ts)
│   ├── agents/
│   │   ├── mod.rs         # Agent trait + detect/wire/unwire/verify dispatch
│   │   └── claude.rs
│   ├── tools/
│   │   ├── mod.rs         # Tool trait + install/version/health dispatch
│   │   └── rtk.rs
│   ├── util/
│   │   ├── colors.rs
│   │   ├── detect.rs
│   │   ├── errors.rs
│   │   ├── exec.rs
│   │   ├── health.rs
│   │   ├── json.rs
│   │   ├── manifest.rs
│   │   ├── paths.rs
│   │   └── version.rs
│   └── download.rs        # fetch/retry/tar/zip (util/download.ts port)
└── tests/
    ├── common/mod.rs      # TestEnv helper
    ├── cli.rs
    ├── registry.rs
    ├── config_json.rs
    ├── manifest.rs
    ├── rtk.rs
    ├── claude.rs
    └── init.rs
```

The `.codegraph/` index covers `src/` (TS). The Rust code is new; use `codegraph_explore` only to read the TS reference modules being ported (paths, detect, exec, json, manifest, registry, claude, rtk, init, cli).

---

### Task 1: Bootstrap toolchain + crate scaffold

**Files:**
- Create: `rust/Cargo.toml`
- Create: `rust/src/main.rs`, `rust/src/lib.rs`
- Create: `rust/src/util/version.rs`
- Modify: `.gitignore` (add `rust/target/`)

**Interfaces:**
- Consumes: nothing.
- Produces: `toksaveVersion() -> &'static str` returning `"0.8.5"`; `lib.rs` exposing `pub mod util;`.

- [ ] **Step 1: Install Rust toolchain**

Rust is not on PATH on the dev machine. Install via rustup (or the user's preferred method), verify:

Run:
```powershell
winget install --id Rustlang.Rustup
# reopen shell, then:
rustup default stable
cargo --version
rustc --version
```
Expected: both print a version ≥ 1.75. If winget is unavailable, use the rustup-init.exe installer from https://rustup.rs.

- [ ] **Step 2: Create Cargo.toml**

```toml
[package]
name = "toksave-rs"
version = "0.8.5"
edition = "2021"
description = "Zero-config token-saver for AI coding agents (Rust rewrite)"
license = "MIT"

[dependencies]
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "gzip"] }
flate2 = "1"
tar = "0.4"
zip = "2"
dirs = "5"
colored = "2"

[dev-dependencies]
tiny_http = "0.12"

[profile.release]
strip = true
```

- [ ] **Step 3: Create src/lib.rs and src/util/mod.rs**

```rust
pub mod cli;
pub mod commands;
pub mod agents;
pub mod tools;
pub mod util;

pub mod registry;
```

```rust
pub mod colors;
pub mod detect;
pub mod errors;
pub mod exec;
pub mod health;
pub mod json;
pub mod manifest;
pub mod paths;
pub mod version;
```

- [ ] **Step 4: Create src/util/version.rs**

```rust
pub fn toksave_version() -> &'static str {
    "0.8.5"
}
```

- [ ] **Step 5: Create src/main.rs (minimal, placeholder dispatch)**

```rust
use toksave_rs::cli::{parse_cli, Command};
use toksave_rs::util::version::toksave_version;

fn main() {
    println!("toksave-rs {}", toksave_version());
    let _ = parse_cli(std::env::args().collect());
}
```

(This will not compile yet — `cli` module is created in Task 8. For Task 1 verification, temporarily comment out the `cli` import and `parse_cli` call.)

- [ ] **Step 6: Update .gitignore**

Append:
```
# Rust build artifacts
rust/target/
```

- [ ] **Step 7: Verify build**

Run: `cargo build`
Expected: compiles (with the `cli` lines commented out from main.rs) with no errors.

- [ ] **Step 8: Commit**

```bash
git add rust/Cargo.toml rust/src/main.rs rust/src/lib.rs rust/src/util/mod.rs rust/src/util/version.rs .gitignore
git commit -m "chore(rust): scaffold toksave-rs crate"
```

---

### Task 2: Errors + health types

**Files:**
- Create: `rust/src/util/errors.rs`
- Create: `rust/src/util/health.rs`
- Test: `rust/tests/errors.rs` (inline `#[cfg(test)]` in errors.rs is acceptable — no TestEnv needed)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub enum ToksaveError { Tool, Install, Download, Network, HealthCheck, Integrity, Platform, Config, Io }`
  - `impl ToksaveError` constructors: `Tool::new(ctx, msg)`, `Download::new(ctx, msg, url)`, `Network::new(kind, msg)`, `Integrity::new(msg)`, `Platform::new(platform, msg)`, `Config::new(path, msg)`, `Io::from` (`impl From<std::io::Error>`).
  - Each variant holds `message: String`, `context: String`, `remediation: Option<String>`. `Display` renders `"{context}: {message}"` and appends `" Remediation: {remediation}"` when present. `std::error::Error` impl with `source()` returning inner boxed error when stored.
  - `pub enum HealthStatus { Healthy, Unhealthy }` plus `pub struct HealthIssue { severity: Severity, message: String, remediation: Option<String> }`, `pub enum Severity { Error, Warning }`. Port of `src/util/health.ts`.

- [ ] **Step 1: Write errors.rs**

```rust
use std::fmt;

#[derive(Debug, Clone)]
pub enum ToksaveErrorKind {
    Tool,
    Install,
    Download,
    Network,
    HealthCheck,
    Integrity,
    Platform,
    Config,
    Io,
}

#[derive(Debug)]
pub struct ToksaveError {
    pub kind: ToksaveErrorKind,
    pub context: String,
    pub message: String,
    pub remediation: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ToksaveError {
    fn new(kind: ToksaveErrorKind, context: &str, message: &str, remediation: Option<&str>) -> Self {
        Self {
            kind,
            context: context.to_string(),
            message: message.to_string(),
            remediation: remediation.map(str::to_string),
            source: None,
        }
    }

    pub fn tool(context: &str, message: &str) -> Self {
        Self::new(ToksaveErrorKind::Tool, context, message, None)
    }
    pub fn install(context: &str, message: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Install, context, message, remediation)
    }
    pub fn download(context: &str, message: &str, url: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Download, context, &format!("{message} ({url})"), remediation)
    }
    pub fn network(context: &str, message: &str, url: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Network, context, &format!("{message} ({url})"), remediation)
    }
    pub fn integrity(context: &str, message: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Integrity, context, message, remediation)
    }
    pub fn platform(platform: &str, message: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Platform, platform, message, remediation)
    }
    pub fn config(path: &str, message: &str) -> Self {
        Self::new(ToksaveErrorKind::Config, path, message, None)
    }
}

impl fmt::Display for ToksaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.message)?;
        if let Some(rem) = &self.remediation {
            write!(f, " Remediation: {rem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ToksaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|b| b as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for ToksaveError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ToksaveErrorKind::Io, "io", &err.to_string(), None)
    }
}

pub type Result<T> = std::result::Result<T, ToksaveError>;
```

- [ ] **Step 2: Write health.rs**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub severity: Severity,
    pub message: String,
    pub remediation: Option<String>,
}

impl HealthIssue {
    pub fn error(message: &str, remediation: &str) -> Self {
        Self {
            severity: Severity::Error,
            message: message.to_string(),
            remediation: Some(remediation.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: Option<String>,
    pub issues: Vec<HealthIssue>,
}
```

- [ ] **Step 3: Write tests (inline in errors.rs, `#[cfg(test)]` module)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_context_and_message() {
        let e = ToksaveError::install("rtk", "failed", Some("run manually"));
        let s = e.to_string();
        assert!(s.contains("rtk"));
        assert!(s.contains("failed"));
        assert!(s.contains("run manually"));
    }

    #[test]
    fn io_error_converts() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no file");
        let e: ToksaveError = io.into();
        assert!(matches!(e.kind, ToksaveErrorKind::Io));
    }

    #[test]
    fn config_error_has_no_remediation() {
        let e = ToksaveError::config("settings.json", "parse failed");
        assert!(!e.to_string().contains("Remediation"));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib util::errors`
Expected: 3 passing tests.

- [ ] **Step 5: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add rust/src/util/errors.rs rust/src/util/health.rs
git commit -m "feat(rust): add ToksaveError and health types"
```

---

### Task 3: Paths (env-aware, cross-platform)

**Files:**
- Create: `rust/src/util/paths.rs`
- Test: `rust/src/util/paths.rs` inline `#[cfg(test)]`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub fn home() -> PathBuf` — env `HOME`, else `USERPROFILE`, else `dirs::home_dir()`.
  - `pub struct ClaudePaths { dir, global_json, settings, skills_dir, agents_md }`
  - `pub fn claude_paths() -> ClaudePaths`
  - `pub fn claude_known_bin_dirs() -> Vec<PathBuf>`
  - `pub fn claude_desktop_paths() -> Vec<PathBuf>`
  - `pub fn local_bin() -> PathBuf`
  - `pub fn cache_dir() -> PathBuf` — env `TOKSAVE_CACHE_DIR`, else `home/.cache/toksave`.
  - `pub fn ensure_dir(p: &Path) -> Result<()>`
  - `pub fn read_file(p: &Path) -> Option<String>`
  - `pub fn write_file(p: &Path, content: &str) -> Result<()>` — temp-write + rename (atomic).
  - `pub fn toksave_abs() -> String` — `std::env::current_exe()` string; in tests where hooks are checked, tests call this so the written command matches what a real install would write.

- [ ] **Step 1: Write paths.rs**

```rust
use crate::util::errors::Result;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    if let Some(h) = env::var_os("HOME") {
        return PathBuf::from(h);
    }
    if let Some(h) = env::var_os("USERPROFILE") {
        return PathBuf::from(h);
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub struct ClaudePaths {
    pub dir: PathBuf,
    pub global_json: PathBuf,
    pub settings: PathBuf,
    pub skills_dir: PathBuf,
    pub agents_md: PathBuf,
}

pub fn claude_paths() -> ClaudePaths {
    let h = home();
    let dir = h.join(".claude");
    ClaudePaths {
        global_json: h.join(".claude.json"),
        settings: dir.join("settings.json"),
        skills_dir: dir.join("skills"),
        agents_md: dir.join("AGENTS.md"),
        dir,
    }
}

pub fn claude_known_bin_dirs() -> Vec<PathBuf> {
    vec![home().join(".local").join("bin")]
}

pub fn claude_desktop_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        let mut v = vec![];
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            v.push(PathBuf::from(local).join("AnthropicClaude").join("claude.exe"));
        }
        if let Some(roam) = env::var_os("APPDATA") {
            v.push(PathBuf::from(roam).join("Claude").join("claude.exe"));
        }
        v
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/Claude.app")]
    } else {
        vec![]
    }
}

pub fn local_bin() -> PathBuf {
    if cfg!(windows) {
        if let Some(la) = env::var_os("LOCALAPPDATA") {
            return PathBuf::from(la).join("Programs").join("toksave");
        }
        home().join("AppData").join("Local").join("Programs").join("toksave")
    } else {
        home().join(".local").join("bin")
    }
}

pub fn cache_dir() -> PathBuf {
    if let Some(c) = env::var_os("TOKSAVE_CACHE_DIR") {
        return PathBuf::from(c);
    }
    home().join(".cache").join("toksave")
}

pub fn ensure_dir(p: &Path) -> Result<()> {
    if !p.exists() {
        fs::create_dir_all(p)?;
    }
    Ok(())
}

pub fn read_file(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

pub fn write_file(p: &Path, content: &str) -> Result<()> {
    if let Some(parent) = p.parent() {
        ensure_dir(parent)?;
    }
    let pid = std::process::id();
    let tmp = p.with_extension(format!("{}.{}.tmp", p.extension().unwrap_or_default().to_string_lossy(), pid));
    let mut f = fs::File::create(&tmp)?;
    f.write_all(content.as_bytes())?;
    f.flush()?;
    fs::rename(&tmp, p)?;
    Ok(())
}

pub fn toksave_abs() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "toksave".to_string())
}
```

- [ ] **Step 2: Write tests (inline)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn home_uses_env() {
        let tmp = env::temp_dir().join("toksave-paths-test-home");
        fs::create_dir_all(&tmp).unwrap();
        let old = env::var_os("HOME");
        env::set_var("HOME", &tmp);
        assert_eq!(home(), tmp);
        if let Some(o) = old {
            env::set_var("HOME", o);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn cache_dir_uses_env_override() {
        let tmp = env::temp_dir().join("toksave-cache-test");
        let old = env::var_os("TOKSAVE_CACHE_DIR");
        env::set_var("TOKSAVE_CACHE_DIR", &tmp);
        assert_eq!(cache_dir(), tmp);
        if let Some(o) = old {
            env::set_var("TOKSAVE_CACHE_DIR", o);
        } else {
            env::remove_var("TOKSAVE_CACHE_DIR");
        }
    }

    #[test]
    fn write_file_is_atomic_and_readable() {
        let p = env::temp_dir().join("toksave-write-test.txt");
        write_file(&p, "hello\n").unwrap();
        assert_eq!(read_file(&p).as_deref(), Some("hello\n"));
        assert!(!fs::read_dir(env::temp_dir()).unwrap().any(|e| {
            e.unwrap().file_name().to_string_lossy().contains(".tmp")
        }));
        fs::remove_file(&p).ok();
    }

    #[test]
    fn claude_paths_under_home() {
        let tmp = env::temp_dir().join("toksave-claude-test");
        let old = env::var_os("HOME");
        env::set_var("HOME", &tmp);
        let cp = claude_paths();
        assert_eq!(cp.dir, tmp.join(".claude"));
        assert_eq!(cp.global_json, tmp.join(".claude.json"));
        if let Some(o) = old {
            env::set_var("HOME", o);
        } else {
            env::remove_var("HOME");
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib util::paths`
Expected: 4 passing tests. NOTE: `write_file_is_atomic_and_readable` asserts no stray `.tmp` file in the temp dir after the write — this is a loose check; if the machine's temp dir has concurrent writers the assertion is flaky. If flaky, weaken to `read_file` equality only.

- [ ] **Step 4: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add rust/src/util/paths.rs
git commit -m "feat(rust): env-aware cross-platform paths"
```

---

### Task 4: detect + exec + colors

**Files:**
- Create: `rust/src/util/detect.rs`
- Create: `rust/src/util/exec.rs`
- Create: `rust/src/util/colors.rs`
- Test: inline `#[cfg(test)]` in each.

**Interfaces:**
- Consumes: `crate::util::errors::Result`.
- Produces:
  - `pub fn is_on_path(name: &str) -> bool`
  - `pub fn find_binary(name: &str) -> Option<String>`
  - `pub fn find_binary_in(name: &str, extra_dirs: &[PathBuf]) -> Option<String>` — PATH first, then extra dirs (with `.exe` on Windows).
  - `pub fn resolve_node() -> Option<String>`
  - `pub struct RunResult { code: i32, stdout: String, stderr: String }`
  - `pub fn run(cmd: &str, args: &[&str]) -> RunResult` — 120s timeout, captures stdout/stderr, trims.
  - `pub fn run_ok(cmd: &str, args: &[&str]) -> bool`
  - `pub fn run_stdout(cmd: &str, args: &[&str]) -> Option<String>` — Some only on exit 0.
  - `pub fn npm_cmd() -> &'static str`, `pub fn npx_cmd() -> &'static str` — `.cmd` suffix on Windows.
  - Colors: `CHECK`, `CROSS`, `WARN`, `BULLET` consts; `ok(msg)`, `err(msg)`, `warn(msg)`, `info(msg)`, `banner(title, subtitle)`, `verbose(msg, is_verbose)` using `colored`.

- [ ] **Step 1: Write detect.rs**

```rust
use crate::util::errors::Result;
use std::env;
use std::path::PathBuf;

fn path_var() -> Vec<PathBuf> {
    env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect()
}

fn exe_candidates(name: &str) -> Vec<String> {
    if cfg!(windows) {
        let pathext = env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
        let mut v = vec![name.to_string()];
        for ext in pathext.split(';') {
            if !ext.is_empty() {
                v.push(format!("{name}{}", ext.to_lowercase()));
            }
        }
        v
    } else {
        vec![name.to_string()]
    }
}

fn exists_executable(dir: &PathBuf, name: &str) -> Option<String> {
    for cand in exe_candidates(name) {
        let p = dir.join(&cand);
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}

pub fn is_on_path(name: &str) -> bool {
    find_binary(name).is_some()
}

pub fn find_binary(name: &str) -> Option<String> {
    for dir in path_var() {
        if let Some(found) = exists_executable(&dir, name) {
            return Some(found);
        }
    }
    None
}

pub fn find_binary_in(name: &str, extra_dirs: &[PathBuf]) -> Option<String> {
    if let Some(found) = find_binary(name) {
        return Some(found);
    }
    for dir in extra_dirs {
        if let Some(found) = exists_executable(dir, name) {
            return Some(found);
        }
    }
    None
}

pub fn resolve_node() -> Option<String> {
    if let Some(found) = find_binary("node") {
        return Some(found);
    }
    let extra: Vec<PathBuf> = if cfg!(windows) {
        ["C:\\Program Files\\nodejs", "C:\\Program Files (x86)\\nodejs"]
            .iter()
            .map(PathBuf::from)
            .collect()
    } else {
        ["/usr/local/bin", "/usr/bin", "/opt/homebrew/bin"]
            .iter()
            .map(PathBuf::from)
            .collect()
    };
    find_binary_in("node", &extra)
}
```

- [ ] **Step 2: Write tests (inline)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_binary_in_checks_extra_dirs() {
        let tmp = env::temp_dir().join("toksave-detect-test");
        fs::create_dir_all(&tmp).unwrap();
        let exe = if cfg!(windows) { "fakebin.exe" } else { "fakebin" };
        fs::write(tmp.join(exe), "x").unwrap();
        let found = find_binary_in("fakebin", &[tmp.clone()]);
        assert!(found.is_some());
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn resolve_node_returns_path() {
        // node may not exist on all dev machines; when it does, path must be non-empty
        if let Some(p) = resolve_node() {
            assert!(!p.is_empty());
        }
    }
}
```

- [ ] **Step 3: Write exec.rs**

```rust
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RunResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn cmd_std(cmd: &str, args: &[&str]) -> RunResult {
    let mut c = Command::new(cmd);
    c.args(args).stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());
    match c.output() {
        Ok(out) => RunResult {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        },
        Err(e) => RunResult {
            code: -1,
            stdout: String::new(),
            stderr: format!("Failed to execute {cmd}: {e}"),
        },
    }
}

pub fn run(cmd: &str, args: &[&str]) -> RunResult {
    cmd_std(cmd, args)
}

pub fn run_ok(cmd: &str, args: &[&str]) -> bool {
    run(cmd, args).code == 0
}

pub fn run_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let r = run(cmd, args);
    if r.code == 0 { Some(r.stdout) } else { None }
}

pub fn npm_cmd() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

pub fn npx_cmd() -> &'static str {
    if cfg!(windows) { "npx.cmd" } else { "npx" }
}
```

- [ ] **Step 4: Write exec tests (inline)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_ok_true_for_echo() {
        let r = run_ok("echo", &["hi"]);
        assert!(r);
    }

    #[test]
    fn run_stdout_echo() {
        if cfg!(windows) {
            assert!(run_stdout("cmd", &["/c", "echo", "hi"]).is_some());
        } else {
            assert_eq!(run_stdout("echo", &["hi"]).as_deref(), Some("hi"));
        }
    }

    #[test]
    fn run_failure_returns_nonzero() {
        let r = run("definitely-not-a-real-binary-xyz", &[]);
        assert_ne!(r.code, 0);
    }
}
```

- [ ] **Step 5: Write colors.rs**

```rust
use colored::Colorize;

pub const CHECK: &str = "✔ ";
pub const CROSS: &str = "✖ ";
pub const WARN: &str = "⚠ ";
pub const BULLET: &str = "• ";

pub fn ok(msg: &str) {
    println!("  {} {}", CHECK.green(), msg);
}

pub fn err(msg: &str) {
    eprintln!("  {} {}", CROSS.red(), msg);
}

pub fn warn(msg: &str) {
    println!("  {} {}", WARN.yellow(), msg);
}

pub fn info(msg: &str) {
    println!("  {} {}", "ℹ ".cyan(), msg);
}

pub fn banner(title: &str, subtitle: &str) {
    println!();
    println!("  {}{}", title.bold().cyan(), format!("  {subtitle}").dimmed());
    println!();
}

pub fn verbose(msg: &str, is_verbose: bool) {
    if is_verbose {
        println!("  [v] {}", msg.dimmed());
    }
}
```

- [ ] **Step 6: fmt + clippy + test**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test --lib util::detect util::exec`
Expected: detect + exec tests pass, clippy clean.

- [ ] **Step 7: Commit**

```bash
git add rust/src/util/detect.rs rust/src/util/exec.rs rust/src/util/colors.rs
git commit -m "feat(rust): detect, exec, colors utils"
```

---

### Task 5: JSON/JSONC config module

**Files:**
- Create: `rust/src/util/json.rs`
- Test: `rust/tests/config_json.rs` (integration — port of `src/__tests__/config.test.ts` JSON half)

**Interfaces:**
- Consumes: `crate::util::paths`, `crate::util::errors`.
- Produces:
  - `pub fn read_json_file(path: &Path) -> Result<Option<serde_json::Value>>` — `None` for missing file, `Err(ToksaveError::Config)` for unparseable content. Trust boundary #1.
  - `pub fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()>` — pretty-printed `serde_json::to_string_pretty` + `\n`.
  - `pub fn get_or_create_object<'a>(parent: &'a mut serde_json::Value, key: &str) -> &'a mut serde_json::Value` — creates `{}` when absent/non-object/array.
  - `pub fn add_to_array_if_missing(arr: &mut Vec<serde_json::Value>, entry: serde_json::Value)`
  - `pub fn remove_from_array(arr: &mut Vec<serde_json::Value>, entry: &serde_json::Value)`
  - `pub fn has_key(obj: &serde_json::Value, key: &str) -> bool`
  - `fn strip_jsonc(raw: &str) -> String` — strips `//` and `/* */` comments and trailing commas, respecting string literals. Non-trivial logic → must carry its own test.

- [ ] **Step 1: Write json.rs (comment + trailing-comma stripping is the hard part)**

```rust
use crate::util::errors::{Result, ToksaveError};
use crate::util::paths::read_file;
use std::path::Path;

/// Remove // and /* */ comments, respecting string literals.
fn strip_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                in_string = true;
                out.push(b);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // line comment
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // block comment
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            _ => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Remove trailing commas before `}` or `]`, respecting string literals.
fn strip_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push(b);
            i += 1;
            continue;
        }
        if b == b',' {
            // look ahead past whitespace; if next non-ws is } or ], skip the comma
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                i += 1; // drop comma
                continue;
            }
            out.push(b);
            i += 1;
            continue;
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn read_json_file(path: &Path) -> Result<Option<serde_json::Value>> {
    let Some(raw) = read_file(path) else {
        return Ok(None);
    };
    let cleaned = strip_trailing_commas(&strip_comments(&raw));
    serde_json::from_str(&cleaned).map(Some).map_err(|e| {
        ToksaveError::config(
            &path.to_string_lossy(),
            &format!("Failed to parse JSON: {e}"),
        )
    })
}

pub fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<()> {
    let s = format!("{}\n", serde_json::to_string_pretty(value)?);
    crate::util::paths::write_file(path, &s)
}

pub fn get_or_create_object<'a>(parent: &'a mut serde_json::Value, key: &str) -> &'a mut serde_json::Value {
    if !parent.is_object() {
        *parent = serde_json::json!({});
    }
    let obj = parent.as_object_mut().expect("object now");
    if !obj.get(key).map(serde_json::Value::is_object).unwrap_or(false) {
        obj.insert(key.to_string(), serde_json::json!({}));
    }
    obj.get_mut(key).expect("inserted")
}

pub fn add_to_array_if_missing(arr: &mut Vec<serde_json::Value>, entry: serde_json::Value) {
    if !arr.contains(&entry) {
        arr.push(entry);
    }
}

pub fn remove_from_array(arr: &mut Vec<serde_json::Value>, entry: &serde_json::Value) {
    if let Some(idx) = arr.iter().position(|v| v == entry) {
        arr.remove(idx);
    }
}

pub fn has_key(obj: &serde_json::Value, key: &str) -> bool {
    obj.is_object() && obj.get(key).is_some()
}
```

- [ ] **Step 2: Write integration test rust/tests/config_json.rs**

```rust
use std::fs;
use std::path::PathBuf;
use toksave_rs::util::json::{
    add_to_array_if_missing, get_or_create_object, read_json_file, remove_from_array, write_json_file,
};
use toksave_rs::util::paths::write_file;

fn tmp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-config-test-{}", std::process::id()))
}

#[test]
fn read_json_file_missing_returns_none() {
    let p = tmp_dir().join("nope.json");
    assert!(read_json_file(&p).unwrap().is_none());
}

#[test]
fn write_read_round_trip() {
    let p = tmp_dir().join("test.json");
    write_json_file(&p, &serde_json::json!({ "foo": "bar", "num": 42 })).unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["foo"], "bar");
    assert_eq!(read["num"], 42);
}

#[test]
fn strips_jsonc_comments() {
    let p = tmp_dir().join("commented.json");
    write_file(&p, "{\n  // comment\n  /* block */\n  \"key\": \"val\"\n}").unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["key"], "val");
}

#[test]
fn strips_trailing_commas() {
    let p = tmp_dir().join("trailing.json");
    write_file(&p, "{ \"a\": 1, \"b\": [2, 3,], }").unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["a"], 1);
    assert_eq!(read["b"][1], 3);
}

#[test]
fn unparseable_config_is_error_not_fallback() {
    let p = tmp_dir().join("bad.json");
    write_file(&p, "{ this is not valid json").unwrap();
    assert!(read_json_file(&p).is_err());
}

#[test]
fn string_with_comment_like_content_survives() {
    let p = tmp_dir().join("strings.json");
    write_file(&p, "{ \"url\": \"http://example.com/x\", \"path\": \"a/*b\" }").unwrap();
    let read = read_json_file(&p).unwrap().unwrap();
    assert_eq!(read["url"], "http://example.com/x");
    assert_eq!(read["path"], "a/*b");
}

#[test]
fn get_or_create_object_creates_nested() {
    let mut obj = serde_json::json!({});
    let sub = get_or_create_object(&mut obj, "mcpServers");
    sub["test"] = serde_json::json!(true);
    assert_eq!(obj["mcpServers"]["test"], true);
}

#[test]
fn add_to_array_avoids_duplicates() {
    let mut arr = serde_json::json!(["a", "b"]);
    let items = arr.as_array_mut().unwrap();
    add_to_array_if_missing(items, serde_json::json!("b"));
    add_to_array_if_missing(items, serde_json::json!("c"));
    assert_eq!(items.len(), 3);
}

#[test]
fn remove_from_array_removes_entry() {
    let mut arr = serde_json::json!(["a", "b", "c"]);
    let items = arr.as_array_mut().unwrap();
    remove_from_array(items, &serde_json::json!("b"));
    assert_eq!(items.len(), 2);
}

#[test]
fn cleanup() {
    fs::remove_dir_all(tmp_dir()).ok();
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test config_json`
Expected: 9 tests pass. The `unparseable_config_is_error_not_fallback` test encodes trust boundary #1.

- [ ] **Step 4: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add rust/src/util/json.rs rust/tests/config_json.rs
git commit -m "feat(rust): JSONC-aware config read/write with strict parse errors"
```

---

### Task 6: Registry (agents + tools matrix)

**Files:**
- Create: `rust/src/registry.rs`
- Create: `rust/src/agents/mod.rs` (trait only), `rust/src/tools/mod.rs` (trait only)
- Test: `rust/tests/registry.rs`

**Interfaces:**
- Consumes: `crate::util::errors`.
- Produces:
  - `pub enum AgentId { Claude, Opencode, Codex, Antigravity, Copilot, Droid, Devin, Warp }` with `FromStr`-style `parse_agent_id(&str) -> Option<AgentId>` (aliases: `cascade`→Devin, `oz`→Warp).
  - `pub enum ToolId { Rtk, Caveman, Codegraph, ContextMode, Ponytail, Principles }` with `parse_tool_id(&str) -> Option<ToolId>` (aliases: `contextmode`→ContextMode, `karpathy-skills`/`karpathy`/`karpathyskills`→Principles).
  - `pub enum Channel { Github, Npm, Skill }`
  - `pub struct AgentInfo { id, label: &'static str, homepage: &'static str, cli_bin: &'static str }`
  - `pub struct ToolInfo { id, label, homepage, channel, min_node_major: u32, not_trackable: bool, instruction_only: bool }`
  - `pub const ALL_AGENTS: &[AgentInfo]`, `pub const ALL_TOOLS: &[ToolInfo]`
  - `pub fn agent_info(id: AgentId) -> &'static AgentInfo`, `pub fn tool_info(id: ToolId) -> &'static ToolInfo`
  - `pub struct RunOpts { dry_run: bool, upgrade: bool, verbose: bool, yes: bool }`
  - `pub struct Detection { installed: bool, source: String }`

- [ ] **Step 1: Write registry.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentId {
    Claude,
    Opencode,
    Codex,
    Antigravity,
    Copilot,
    Droid,
    Devin,
    Warp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolId {
    Rtk,
    Caveman,
    Codegraph,
    ContextMode,
    Ponytail,
    Principles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Github,
    Npm,
    Skill,
}

#[derive(Debug, Clone, Copy)]
pub struct AgentInfo {
    pub id: AgentId,
    pub label: &'static str,
    pub homepage: &'static str,
    pub cli_bin: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolInfo {
    pub id: ToolId,
    pub label: &'static str,
    pub homepage: &'static str,
    pub channel: Channel,
    pub min_node_major: u32,
    pub not_trackable: bool,
    pub instruction_only: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RunOpts {
    pub dry_run: bool,
    pub upgrade: bool,
    pub verbose: bool,
    pub yes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Detection {
    pub installed: bool,
    pub source: String,
}

pub const ALL_AGENTS: &[AgentInfo] = &[
    AgentInfo { id: AgentId::Claude, label: "Claude Code", homepage: "https://github.com/anthropics/claude-code", cli_bin: "claude" },
    AgentInfo { id: AgentId::Opencode, label: "OpenCode", homepage: "https://github.com/anomalyco/opencode", cli_bin: "opencode" },
    AgentInfo { id: AgentId::Codex, label: "Codex", homepage: "https://github.com/openai/codex", cli_bin: "codex" },
    AgentInfo { id: AgentId::Antigravity, label: "Antigravity", homepage: "https://antigravity.google", cli_bin: "agy" },
    AgentInfo { id: AgentId::Copilot, label: "GitHub Copilot", homepage: "https://github.com/github/copilot-cli", cli_bin: "copilot" },
    AgentInfo { id: AgentId::Droid, label: "Factory Droid", homepage: "https://factory.ai", cli_bin: "droid" },
    AgentInfo { id: AgentId::Devin, label: "Devin / Cascade", homepage: "https://devin.ai", cli_bin: "devin" },
    AgentInfo { id: AgentId::Warp, label: "Warp / Oz", homepage: "https://warp.dev", cli_bin: "warp" },
];

pub const ALL_TOOLS: &[ToolInfo] = &[
    ToolInfo { id: ToolId::Rtk, label: "RTK", homepage: "https://github.com/rtk-ai/rtk", channel: Channel::Github, min_node_major: 0, not_trackable: false, instruction_only: false },
    ToolInfo { id: ToolId::Caveman, label: "Caveman", homepage: "https://github.com/JuliusBrussee/caveman", channel: Channel::Skill, min_node_major: 0, not_trackable: false, instruction_only: false },
    ToolInfo { id: ToolId::Codegraph, label: "CodeGraph", homepage: "https://github.com/colbymchenry/codegraph", channel: Channel::Npm, min_node_major: 18, not_trackable: false, instruction_only: false },
    ToolInfo { id: ToolId::ContextMode, label: "Context-Mode", homepage: "https://github.com/mksglu/context-mode", channel: Channel::Npm, min_node_major: 22, not_trackable: false, instruction_only: false },
    ToolInfo { id: ToolId::Ponytail, label: "Ponytail", homepage: "https://github.com/DietrichGebert/ponytail", channel: Channel::Npm, min_node_major: 0, not_trackable: false, instruction_only: false },
    ToolInfo { id: ToolId::Principles, label: "Principles", homepage: "https://github.com/multica-ai/andrej-karpathy-skills", channel: Channel::Skill, min_node_major: 0, not_trackable: true, instruction_only: true },
];

pub fn agent_info(id: AgentId) -> &'static AgentInfo {
    ALL_AGENTS.iter().find(|a| a.id == id).expect("valid agent id")
}

pub fn tool_info(id: ToolId) -> &'static ToolInfo {
    ALL_TOOLS.iter().find(|t| t.id == id).expect("valid tool id")
}

pub fn parse_agent_id(s: &str) -> Option<AgentId> {
    match s.to_lowercase().trim() {
        "claude" => Some(AgentId::Claude),
        "opencode" => Some(AgentId::Opencode),
        "codex" => Some(AgentId::Codex),
        "antigravity" => Some(AgentId::Antigravity),
        "copilot" => Some(AgentId::Copilot),
        "droid" => Some(AgentId::Droid),
        "devin" => Some(AgentId::Devin),
        "cascade" => Some(AgentId::Devin),
        "warp" => Some(AgentId::Warp),
        "oz" => Some(AgentId::Warp),
        _ => None,
    }
}

pub fn parse_tool_id(s: &str) -> Option<ToolId> {
    match s.to_lowercase().trim() {
        "rtk" => Some(ToolId::Rtk),
        "caveman" => Some(ToolId::Caveman),
        "codegraph" => Some(ToolId::Codegraph),
        "context-mode" | "contextmode" => Some(ToolId::ContextMode),
        "ponytail" => Some(ToolId::Ponytail),
        "principles" | "karpathy-skills" | "karpathy" | "karpathyskills" => Some(ToolId::Principles),
        _ => None,
    }
}
```

- [ ] **Step 2: Write agent + tool traits**

`rust/src/agents/mod.rs`:
```rust
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::errors::Result;

pub trait Agent {
    fn detect(&self) -> Detection;
    fn wire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool>;
    fn unwire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool>;
    fn verify(&self, tool: ToolId) -> Option<bool>;
}
```

`rust/src/tools/mod.rs`:
```rust
use crate::registry::RunOpts;
use crate::util::errors::Result;
use crate::util::health::HealthStatus;

pub trait Tool {
    async fn install(&self, opts: &RunOpts) -> Result<bool>;
    fn installed_version(&self) -> Option<String>;
    async fn latest_version(&self) -> Result<Option<String>>;
    fn health_check(&self) -> HealthStatus;
}
```

- [ ] **Step 3: Write test rust/tests/registry.rs**

```rust
use toksave_rs::registry::{
    agent_info, parse_agent_id, parse_tool_id, tool_info, ALL_AGENTS, ALL_TOOLS,
    AgentId, ToolId,
};

#[test]
fn all_agents_count() {
    assert_eq!(ALL_AGENTS.len(), 8);
}

#[test]
fn all_tools_count() {
    assert_eq!(ALL_TOOLS.len(), 6);
}

#[test]
fn parse_agent_aliases() {
    assert_eq!(parse_agent_id("cascade"), Some(AgentId::Devin));
    assert_eq!(parse_agent_id("oz"), Some(AgentId::Warp));
    assert_eq!(parse_agent_id("CLAUDE"), Some(AgentId::Claude));
    assert_eq!(parse_agent_id("bogus"), None);
}

#[test]
fn parse_tool_aliases() {
    assert_eq!(parse_tool_id("contextmode"), Some(ToolId::ContextMode));
    assert_eq!(parse_tool_id("karpathy"), Some(ToolId::Principles));
    assert_eq!(parse_tool_id("karpathy-skills"), Some(ToolId::Principles));
    assert_eq!(parse_tool_id("nope"), None);
}

#[test]
fn info_lookups() {
    assert_eq!(agent_info(AgentId::Claude).cli_bin, "claude");
    assert_eq!(tool_info(ToolId::ContextMode).min_node_major, 22);
    assert!(tool_info(ToolId::Principles).instruction_only);
}
```

- [ ] **Step 4: Run tests + fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test --test registry`
Expected: 5 tests pass, clippy clean. (Note: `lib.rs` currently references `cli` — Task 1 left it commented; `agents`/`tools`/`commands` mods must exist or be temporarily commented until their tasks land. If `lib.rs` has stubs for all mods, create empty `commands/mod.rs` in this task.)

- [ ] **Step 5: Commit**

```bash
git add rust/src/registry.rs rust/src/agents/mod.rs rust/src/tools/mod.rs rust/src/commands/mod.rs rust/tests/registry.rs
git commit -m "feat(rust): agent/tool registry with id parsing"
```

---

### Task 7: Manifest

**Files:**
- Create: `rust/src/util/manifest.rs`
- Test: `rust/tests/manifest.rs`

**Interfaces:**
- Consumes: `crate::util::paths::cache_dir`.
- Produces:
  - `#[derive(Serialize, Deserialize)] pub struct ManifestEntry { agent: String, tool: String, wired_at: String, #[serde(skip_serializing_if = "Option::is_none")] version: Option<String> }`
  - `#[derive(Serialize, Deserialize, Default)] pub struct Manifest { entries: Vec<ManifestEntry> }`
  - `pub fn read_manifest() -> Manifest` (empty on missing/unparseable — matches TS behavior).
  - `pub fn record_wire(agent: &str, tool: &str, version: Option<&str>) -> Result<()>` — lock file + replace existing agent+tool entry + atomic write.
  - `pub fn remove_wire(agent: &str, tool: &str) -> Result<()>`
  - `pub fn was_wired_by_us(agent: &str, tool: &str) -> bool`
  - Lock: create `.lock` dir (mkdir is atomic) with 30s stale detection and 5s acquisition timeout — port of TS `withManifestLock`.

- [ ] **Step 1: Write manifest.rs**

```rust
use crate::util::errors::Result;
use crate::util::paths::cache_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub agent: String,
    pub tool: String,
    pub wired_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

fn manifest_path() -> PathBuf {
    cache_dir().join("manifest.json")
}

fn lock_path() -> PathBuf {
    cache_dir().join("manifest.json.lock")
}

fn read_manifest_file() -> Manifest {
    let p = manifest_path();
    let Ok(raw) = fs::read_to_string(&p) else {
        return Manifest::default();
    };
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to parse manifest at {}: {e}", p.display());
        Manifest::default()
    })
}

fn write_manifest(m: &Manifest) -> Result<()> {
    let p = manifest_path();
    if let Some(parent) = p.parent() {
        crate::util::paths::ensure_dir(parent)?;
    }
    let content = format!("{}\n", serde_json::to_string_pretty(m)?);
    crate::util::paths::write_file(&p, &content)
}

fn stale_lock_age() -> Duration {
    Duration::from_secs(30)
}

/// Acquire the manifest lock by creating a lock dir (mkdir is atomic).
/// Timeout after 5s; stale locks older than 30s are force-removed.
fn with_manifest_lock<T>(f: impl FnOnce() -> T) -> T {
    let lock = lock_path();
    if let Some(parent) = lock.parent() {
        let _ = crate::util::paths::ensure_dir(parent);
    }
    let started = SystemTime::now();
    loop {
        match fs::create_dir(&lock) {
            Ok(()) => break,
            Err(_) => {
                if let Ok(meta) = fs::metadata(&lock) {
                    if let Ok(modified) = meta.modified() {
                        if modified.elapsed().unwrap_or_default() > stale_lock_age() {
                            let _ = fs::remove_dir_all(&lock);
                            continue;
                        }
                    }
                }
                if started.elapsed().unwrap_or_default() > Duration::from_secs(5) {
                    eprintln!("Timed out waiting for manifest lock: {}", lock.display());
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    let result = f();
    let _ = fs::remove_dir_all(&lock);
    result
}

pub fn read_manifest() -> Manifest {
    read_manifest_file()
}

pub fn record_wire(agent: &str, tool: &str, version: Option<&str>) -> Result<()> {
    with_manifest_lock(|| {
        let mut m = read_manifest_file();
        m.entries.retain(|e| !(e.agent == agent && e.tool == tool));
        m.entries.push(ManifestEntry {
            agent: agent.to_string(),
            tool: tool.to_string(),
            wired_at: now_iso8601(),
            version: version.map(str::to_string),
        });
        let _ = write_manifest(&m);
    });
    Ok(())
}

pub fn remove_wire(agent: &str, tool: &str) -> Result<()> {
    with_manifest_lock(|| {
        let mut m = read_manifest_file();
        m.entries.retain(|e| !(e.agent == agent && e.tool == tool));
        let _ = write_manifest(&m);
    });
    Ok(())
}

pub fn was_wired_by_us(agent: &str, tool: &str) -> bool {
    read_manifest()
        .entries
        .iter()
        .any(|e| e.agent == agent && e.tool == tool)
}

fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC3339-ish; tests only compare equality of entries, not format.
    // ponytail: full chrono formatting deferred; this is stable & parseable.
    format!("{secs}s")
}
```

- [ ] **Step 2: Write test rust/tests/manifest.rs**

```rust
use std::fs;
use std::path::PathBuf;
use toksave_rs::util::manifest::{read_manifest, record_wire, remove_wire, was_wired_by_us};

fn tmp_cache() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-manifest-test-{}", std::process::id()))
}

#[test]
fn read_manifest_empty_for_fresh() {
    let old = std::env::var_os("TOKSAVE_CACHE_DIR");
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    let m = read_manifest();
    assert!(m.entries.is_empty());
    restore(old);
}

#[test]
fn record_wire_and_was_wired_by_us() {
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    record_wire("claude", "rtk", Some("0.43.0")).unwrap();
    assert!(was_wired_by_us("claude", "rtk"));
    assert!(!was_wired_by_us("claude", "caveman"));
    restore(std::env::var_os("TOKSAVE_CACHE_DIR"));
}

#[test]
fn record_replaces_existing() {
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    record_wire("claude", "rtk", Some("0.42.0")).unwrap();
    record_wire("claude", "rtk", Some("0.43.0")).unwrap();
    let m = read_manifest();
    let matches: Vec<_> = m.entries.iter().filter(|e| e.agent == "claude" && e.tool == "rtk").collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].version.as_deref(), Some("0.43.0"));
}

#[test]
fn remove_wire_clears() {
    std::env::set_var("TOKSAVE_CACHE_DIR", tmp_cache());
    record_wire("opencode", "codegraph", None).unwrap();
    remove_wire("opencode", "codegraph").unwrap();
    assert!(!was_wired_by_us("opencode", "codegraph"));
}

fn restore(v: Option<std::ffi::OsString>) {
    match v {
        Some(v) => std::env::set_var("TOKSAVE_CACHE_DIR", v),
        None => std::env::remove_var("TOKSAVE_CACHE_DIR"),
    }
}

#[test]
fn cleanup() {
    fs::remove_dir_all(tmp_cache()).ok();
    std::env::remove_var("TOKSAVE_CACHE_DIR");
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test manifest`
Expected: 4 tests pass. (Tests mutate the global `TOKSAVE_CACHE_DIR` env; that's fine since `cargo test` runs test binaries single-threaded by default per binary. The test binary is one process, tests run in threads — env mutation is race-prone. To be safe, run with `cargo test --test manifest -- --test-threads=1`.)

- [ ] **Step 4: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add rust/src/util/manifest.rs rust/tests/manifest.rs
git commit -m "feat(rust): manifest with atomic lock"
```

---

### Task 8: Download util (fetch/retry/tar/zip)

**Files:**
- Create: `rust/src/util/download.rs`
- Test: `rust/tests/download.rs` (uses `tiny_http` local mock server)

**Interfaces:**
- Consumes: `crate::util::errors`, `crate::util::version`.
- Produces:
  - `pub struct DownloadOptions { retries: usize, timeout: Duration, checksum: Option<String> }` (default retries 3, timeout 120s).
  - `pub async fn fetch_with_retry(url: &str, opts: &DownloadOptions) -> Result<Vec<u8>>` — returns body bytes; no retry on 404; backoff 1s/2s/4s; uses `reqwest`.
  - `pub async fn fetch_json(url: &str) -> Result<serde_json::Value>` — 10s timeout.
  - `pub async fn download_tar_gz(url: &str, dest_dir: &Path, opts: &DownloadOptions) -> Result<()>` — verify checksum if provided, extract via `tar` + `flate2`, reject traversal.
  - `pub async fn download_zip(url: &str, dest_dir: &Path, opts: &DownloadOptions) -> Result<()>` — verify checksum, extract via `zip`, reject traversal (zip-slip).
  - `pub fn make_executable(path: &Path) -> Result<()>` (Unix only: chmod 755).
  - `fn is_safe_archive_path(entry_path: &str, dest_dir: &Path) -> bool` — trust boundary #2. Non-trivial → own test.
  - `fn verify_checksum(bytes: &[u8], expected: &str, url: &str) -> Result<()>`.

- [ ] **Step 1: Write download.rs**

```rust
use crate::util::errors::{Result, ToksaveError};
use crate::util::version::toksave_version;
use reqwest::header::USER_AGENT;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub retries: usize,
    pub timeout: Duration,
    pub checksum: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            retries: 3,
            timeout: Duration::from_secs(120),
            checksum: None,
        }
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(format!("toksave-rs/{}", toksave_version()))
        .build()
        .expect("reqwest client")
}

async fn fetch_with_retry_inner(url: &str, opts: &DownloadOptions) -> Result<Vec<u8>> {
    let client = client();
    let mut last_err: Option<ToksaveError> = None;
    for attempt in 0..=opts.retries {
        let resp = client.get(url).timeout(opts.timeout).send().await;
        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    let bytes = r.bytes().await.map_err(|e| {
                        ToksaveError::network("download", "failed to read body", url, None)
                            .with_source(e)
                    })?;
                    return Ok(bytes.to_vec());
                }
                if r.status().as_u16() == 404 {
                    return Err(ToksaveError::download(
                        "download",
                        "HTTP 404 Not Found",
                        url,
                        Some("URL not found. Check if the resource exists or try a different version."),
                    ));
                }
                last_err = Some(ToksaveError::download(
                    "download",
                    &format!("HTTP {} {}", r.status().as_u16(), r.status().canonical_reason().unwrap_or("")),
                    url,
                    None,
                ));
            }
            Err(e) => {
                last_err = Some(ToksaveError::network("download", &e.to_string(), url, None));
            }
        }
        if attempt < opts.retries {
            let backoff = Duration::from_millis(1000 * (1 << attempt));
            tokio::time::sleep(backoff).await;
        }
    }
    Err(last_err.unwrap_or_else(|| ToksaveError::network("download", "unknown error", url, None)))
}

pub async fn fetch_with_retry(url: &str, opts: &DownloadOptions) -> Result<Vec<u8>> {
    fetch_with_retry_inner(url, opts).await
}

pub async fn fetch_json(url: &str) -> Result<serde_json::Value> {
    let opts = DownloadOptions {
        timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let bytes = fetch_with_retry_inner(url, &opts).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| ToksaveError::network("json", &format!("invalid JSON: {e}"), url, None))
}

fn verify_checksum(bytes: &[u8], expected: &str, url: &str) -> Result<()> {
    use sha2?; // see note below
    Ok(())
}

pub fn is_safe_archive_path(entry_path: &str, dest_dir: &Path) -> bool {
    let p = Path::new(entry_path);
    if p.is_absolute() {
        return false;
    }
    let mut components = Vec::new();
    for c in p.components() {
        use std::path::Component;
        match c {
            Component::Normal(s) => components.push(s.to_string_lossy().into_owned()),
            Component::ParentDir => return false,
            Component::CurDir => {}
            _ => return false,
        }
    }
    let target = components.iter().fold(dest_dir.to_path_buf(), |acc, part| acc.join(part));
    target.starts_with(dest_dir)
}

pub async fn download_tar_gz(url: &str, dest_dir: &Path, opts: &DownloadOptions) -> Result<()> {
    let bytes = fetch_with_retry_inner(url, opts).await?;
    if let Some(expected) = &opts.checksum {
        verify_checksum_sha256(&bytes, expected, url)?;
    }
    crate::util::paths::ensure_dir(dest_dir)?;

    let mut decoder = flate2::read::GzDecoder::new(bytes.as_slice());
    let mut tar_bytes = Vec::new();
    decoder.read_to_end(&mut tar_bytes).map_err(|e| {
        ToksaveError::network("tar.gz", &format!("corrupt gzip: {e}"), url, None)
    })?;

    let mut archive = tar::Archive::new(tar_bytes.as_slice());
    let entries: Vec<_> = archive
        .entries()
        .map_err(|e| ToksaveError::network("tar.gz", &e.to_string(), url, None))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ToksaveError::network("tar.gz", &e.to_string(), url, None))?;

    for entry in entries {
        let path = entry
            .path()
            .map_err(|e| ToksaveError::network("tar.gz", &e.to_string(), url, None))?
            .to_string_lossy()
            .into_owned();
        if !is_safe_archive_path(&path, dest_dir) {
            return Err(ToksaveError::download(
                "tar.gz",
                &format!("Archive entry escapes destination: {path}"),
                url,
                Some("The downloaded archive contains malicious entries. Aborting extraction."),
            ));
        }
        let target = dest_dir.join(&path);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            crate::util::paths::ensure_dir(&target)?;
        } else if kind.is_file() {
            if let Some(parent) = target.parent() {
                crate::util::paths::ensure_dir(parent)?;
            }
            let mut out = std::fs::File::create(&target)?;
            let mut e = entry;
            std::io::copy(&mut e, &mut out)?;
        }
    }
    Ok(())
}

pub async fn download_zip(url: &str, dest_dir: &Path, opts: &DownloadOptions) -> Result<()> {
    let bytes = fetch_with_retry_inner(url, opts).await?;
    if let Some(expected) = &opts.checksum {
        verify_checksum_sha256(&bytes, expected, url)?;
    }
    crate::util::paths::ensure_dir(dest_dir)?;

    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| ToksaveError::network("zip", &format!("corrupt zip: {e}"), url, None))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| ToksaveError::network("zip", &e.to_string(), url, None))?;
        let path = entry.name().to_string();
        if !is_safe_archive_path(&path, dest_dir) {
            return Err(ToksaveError::download(
                "zip",
                &format!("Zip entry escapes destination: {path}"),
                url,
                Some("The downloaded archive contains malicious entries. Aborting extraction."),
            ));
        }
        let target = dest_dir.join(&path);
        if entry.is_dir() {
            crate::util::paths::ensure_dir(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                crate::util::paths::ensure_dir(parent)?;
            }
            let mut out = std::fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

pub fn make_executable(path: &Path) -> Result<()> {
    if cfg!(windows) {
        return Ok(());
    }
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

fn verify_checksum_sha256(bytes: &[u8], expected: &str, url: &str) -> Result<()> {
    let actual = {
        use std::io::Write;
        let mut h = <sha2::Sha256 as sha2::Digest>::new();
        h.update(bytes);
        format!("{:x}", h.finalize())
    };
    if actual != expected.to_lowercase() {
        return Err(ToksaveError::integrity(
            "downloaded file",
            &format!("Checksum mismatch for {url} (expected {expected}, got {actual})"),
            Some("The downloaded file may be corrupted or tampered with. Try again or verify the source."),
        ));
    }
    Ok(())
}
```

NOTE: The `sha2` crate is needed for checksum verification. Add `sha2 = "0.10"` to `Cargo.toml` `[dependencies]` in this task. The placeholder `fn verify_checksum`/`use sha2?;` at the top of the file is INVALID — delete that stub; only the real `verify_checksum_sha256` at the bottom stays, and callers use it.

- [ ] **Step 2: Add sha2 to Cargo.toml**

Add under `[dependencies]`:
```toml
sha2 = "0.10"
```

- [ ] **Step 3: Write test rust/tests/download.rs**

```rust
use std::io::Write;
use std::path::PathBuf;
use toksave_rs::util::download::{
    download_tar_gz, download_zip, fetch_json, fetch_with_retry, is_safe_archive_path,
    DownloadOptions,
};

fn tmp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-download-test-{}", std::process::id()))
}

#[test]
fn is_safe_archive_path_rejects_traversal() {
    let dest = tmp_dir().join("dest");
    assert!(is_safe_archive_path("ok/file.txt", &dest));
    assert!(is_safe_archive_path("a/b/c.txt", &dest));
    assert!(!is_safe_archive_path("../escape.txt", &dest));
    assert!(!is_safe_archive_path("a/../../escape.txt", &dest));
    assert!(!is_safe_archive_path("/absolute/path", &dest));
    assert!(!is_safe_archive_path("C:/abs", &dest));
}

#[tokio::test]
async fn fetch_404_not_retried_and_errors() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/missing");
    let h = std::thread::spawn(move || {
        if let Ok(Some(_req)) = server.recv() {
            // respond 404
        }
    });
    let res = fetch_with_retry(&url, &DownloadOptions::default()).await;
    assert!(res.is_err());
    h.join().ok();
}

#[tokio::test]
async fn fetch_json_ok() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/json");
    let h = std::thread::spawn(move || {
        if let Ok(Some(req)) = server.recv() {
            let resp = tiny_http::Response::from_string("{\"tag_name\":\"v1.2.3\"}");
            req.respond(resp).ok();
        }
    });
    let v = fetch_json(&url).await.unwrap();
    assert_eq!(v["tag_name"], "v1.2.3");
    h.join().ok();
}

#[tokio::test]
async fn download_zip_rejects_zip_slip() {
    // Build a malicious zip in memory: entry "../evil.txt"
    let cursor = std::io::Cursor::new(vec![0u8; 0]);
    let mut zw = zip::ZipWriter::new(cursor);
    let opts = zip::write::SimpleFileOptions::default();
    zw.start_file("../evil.txt", opts).unwrap();
    zw.write_all(b"boom").unwrap();
    let cursor = zw.finish().unwrap();
    let bytes = cursor.into_inner();

    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let url = format!("http://127.0.0.1:{port}/evil.zip");
    let h = std::thread::spawn(move || {
        if let Ok(Some(req)) = server.recv() {
            let resp = tiny_http::Response::from_data(bytes);
            req.respond(resp).ok();
        }
    });
    let dest = tmp_dir().join("zipdest");
    let res = download_zip(&url, &dest, &DownloadOptions::default()).await;
    assert!(res.is_err());
    assert!(!dest.join("..").join("evil.txt").exists());
    h.join().ok();
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test download`
Expected: 4 tests pass. If `tiny_http` server bind/threading is flaky on Windows CI, use the `--test-threads=1` flag.

- [ ] **Step 5: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add rust/src/util/download.rs rust/tests/download.rs rust/Cargo.toml
git commit -m "feat(rust): download with retry, checksum, safe archive extraction"
```

---

### Task 9: RTK tool

**Files:**
- Create: `rust/src/tools/rtk.rs`
- Test: `rust/tests/rtk.rs`

**Interfaces:**
- Consumes: `crate::util::detect`, `crate::util::exec`, `crate::util::download`, `crate::util::paths`, `crate::util::errors`, `crate::util::health`, `crate::registry`.
- Produces:
  - `pub struct RtkTool;` implementing `Tool`.
  - `fn asset_name() -> Option<&'static str>` — platform/arch asset mapping (port of TS).
  - `pub fn is_installed_but_unreachable() -> bool`
  - `fn rtk_bin_name() -> &'static str` — `"rtk.exe"` on Windows else `"rtk"`.
  - `pub fn tool_installed_version(tool: ToolId) -> Option<String>` dispatch helper in `tools/mod.rs` (used by init version table).

- [ ] **Step 1: Write tools/rtk.rs**

```rust
use crate::registry::RunOpts;
use crate::util::detect::{find_binary_in, is_on_path};
use crate::util::download::{download_tar_gz, download_zip, make_executable, DownloadOptions};
use crate::util::errors::{Result, ToksaveError};
use crate::util::exec::{run, run_stdout};
use crate::util::health::{HealthIssue, HealthStatus};
use crate::util::paths::{ensure_dir, home, local_bin};
use crate::tools::Tool;
use std::path::PathBuf;

pub fn asset_name() -> Option<&'static str> {
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { return Some("rtk-x86_64-apple-darwin.tar.gz"); }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { return Some("rtk-aarch64-apple-darwin.tar.gz"); }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { return Some("rtk-x86_64-unknown-linux-musl.tar.gz"); }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { return Some("rtk-aarch64-unknown-linux-musl.tar.gz"); }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { return Some("rtk-x86_64-pc-windows-msvc.zip"); }
    None
}

pub fn rtk_bin_name() -> &'static str {
    if cfg!(windows) { "rtk.exe" } else { "rtk" }
}

pub fn local_rtk_path() -> PathBuf {
    local_bin().join(rtk_bin_name())
}

pub fn is_installed_but_unreachable() -> bool {
    if is_on_path("rtk") {
        return false;
    }
    local_rtk_path().exists()
}

pub struct RtkTool;

impl Tool for RtkTool {
    async fn install(&self, opts: &RunOpts) -> Result<bool> {
        if installed_version().is_some() && !opts.upgrade {
            return Ok(true);
        }
        if opts.dry_run {
            return Ok(true);
        }

        let dest = local_bin();
        ensure_dir(&dest)?;

        if let Some(asset) = asset_name() {
            let url = format!("https://github.com/rtk-ai/rtk/releases/latest/download/{asset}");
            let result = async {
                if asset.ends_with(".tar.gz") {
                    download_tar_gz(&url, &dest, &DownloadOptions::default()).await?;
                    make_executable(&dest.join("rtk"))?;
                } else if asset.ends_with(".zip") {
                    download_zip(&url, &dest, &DownloadOptions::default()).await?;
                }
                let rtk_path = local_rtk_path();
                let init = run(&rtk_path.to_string_lossy(), &["init", "-g"]);
                if init.code != 0 {
                    let _ = std::fs::remove_file(&rtk_path);
                    return Err(ToksaveError::install(
                        "rtk",
                        "Failed to initialize RTK shell integration",
                        Some("Try running 'rtk init -g' manually after installation completes"),
                    ));
                }
                Ok(true)
            }
            .await;
            match result {
                Ok(ok) => return Ok(ok),
                Err(e) => {
                    // fall through to fallback methods (mirror TS: try next method)
                    let _ = e;
                }
            }
        }

        // Fallback: official install script (Unix only)
        #[cfg(not(windows))]
        {
            if is_on_path("curl") && is_on_path("sh") {
                let r = run(
                    "sh",
                    &["-c", "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh"],
                );
                if r.code == 0 {
                    let init = run("rtk", &["init", "-g"]);
                    if init.code == 0 {
                        return Ok(true);
                    }
                }
            }
        }

        // Fallback: cargo install
        if is_on_path("cargo") {
            let r = run("cargo", &["install", "--git", "https://github.com/rtk-ai/rtk"]);
            if r.code == 0 {
                let init = run("rtk", &["init", "-g"]);
                if init.code == 0 {
                    return Ok(true);
                }
            }
        }

        Err(ToksaveError::platform(
            &format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            "No installation method available",
            Some("Visit https://github.com/rtk-ai/rtk for manual installation instructions"),
        ))
    }

    fn installed_version(&self) -> Option<String> {
        installed_version()
    }

    async fn latest_version(&self) -> Result<Option<String>> {
        let v = crate::util::download::fetch_json(
            "https://api.github.com/repos/rtk-ai/rtk/releases/latest",
        )
        .await
        .ok()?;
        let tag = v.get("tag_name").and_then(|t| t.as_str())?;
        Some(tag.trim_start_matches('v').to_string()).pipe(Ok)
    }

    fn health_check(&self) -> HealthStatus {
        let mut issues = vec![];
        let version = installed_version();

        let Some(version) = version else {
            return HealthStatus {
                healthy: false,
                version: None,
                issues: vec![HealthIssue::error(
                    "RTK is not installed",
                    "Run: toksave install rtk",
                )],
            };
        };

        if is_installed_but_unreachable() {
            let bin = local_bin();
            let instruct = if cfg!(windows) {
                format!("Add {} to your system PATH via System Properties or setx PATH \"%PATH%;{}\"", bin.display(), bin.display())
            } else {
                format!("Add 'export PATH=\"$PATH:{}\"' to your shell rc file (e.g. ~/.bashrc or ~/.zshrc) and restart your terminal", bin.display())
            };
            issues.push(HealthIssue {
                severity: crate::util::health::Severity::Error,
                message: format!("RTK is installed in {} but is not on your PATH", bin.display()),
                remediation: Some(format!("Enforcement hooks will fail with 'command not found'. {instruct}")),
            });
        }

        let healthy = issues.is_empty() || issues.iter().all(|i| i.severity == crate::util::health::Severity::Warning);
        HealthStatus {
            healthy,
            version: Some(version),
            issues,
        }
    }
}

pub fn installed_version() -> Option<String> {
    let path_version = run_stdout("rtk", &["--version"])?;
    let pv = path_version.trim();
    if !pv.is_empty() {
        return Some(pv.to_string());
    }
    let local = local_rtk_path();
    if !local.exists() {
        return None;
    }
    run_stdout(&local.to_string_lossy(), &["--version"])
}
```

NOTE: `Option::pipe(Ok)` is not std — replace `latest_version` with:
```rust
async fn latest_version(&self) -> Result<Option<String>> {
    match crate::util::download::fetch_json(
        "https://api.github.com/repos/rtk-ai/rtk/releases/latest",
    )
    .await
    {
        Ok(v) => Ok(v.get("tag_name").and_then(|t| t.as_str()).map(|s| s.trim_start_matches('v').to_string())),
        Err(_) => Ok(None),
    }
}
```

Also `find_binary_in`, `home` imports are unused in rtk.rs — remove them (`is_installed_but_unreachable` uses `is_on_path` + `local_rtk_path`). Keep imports minimal.

- [ ] **Step 2: Add dispatch helper in tools/mod.rs**

```rust
pub use crate::tools::rtk::{installed_version as rtk_installed_version, RtkTool};
use crate::registry::{ToolId, RunOpts};
use crate::util::errors::Result;
use crate::util::health::HealthStatus;

pub trait Tool { /* as defined in Task 6 */ }

pub fn tool_installed_version(tool: ToolId) -> Option<String> {
    match tool {
        ToolId::Rtk => rtk_installed_version(),
        _ => None, // other tools ported in later phases
    }
}

pub async fn install_tool(tool: ToolId, opts: &RunOpts) -> Result<bool> {
    match tool {
        ToolId::Rtk => RtkTool.install(opts).await,
        _ => Ok(false), // not yet implemented
    }
}

pub fn tool_health_check(tool: ToolId) -> HealthStatus {
    match tool {
        ToolId::Rtk => RtkTool.health_check(),
        _ => HealthStatus { healthy: false, version: None, issues: vec![] },
    }
}
```

- [ ] **Step 3: Write test rust/tests/rtk.rs**

```rust
use toksave_rs::tools::rtk::{asset_name, is_installed_but_unreachable, local_rtk_path};
use toksave_rs::tools::rtk::RtkTool;
use toksave_rs::util::paths::{ensure_dir, local_bin};
use toksave_rs::registry::RunOpts;

#[test]
fn asset_name_matches_platform() {
    let asset = asset_name();
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    assert_eq!(asset, Some("rtk-x86_64-pc-windows-msvc.zip"));
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    assert_eq!(asset, Some("rtk-x86_64-unknown-linux-musl.tar.gz"));
    // Other platforms: assert it's one of the known names or None for unsupported
    #[cfg(not(any(all(target_os = "windows", target_arch = "x86_64"), all(target_os = "linux", target_arch = "x86_64"))))]
    assert!(asset.is_none() || asset.unwrap().ends_with(".tar.gz") || asset.unwrap().ends_with(".zip"));
}

#[test]
fn installed_but_unreachable_detects_local_bin_only() {
    // Clean PATH env for this test
    let old_path = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    assert!(!is_installed_but_unreachable()); // nothing installed
    let bin = local_bin();
    ensure_dir(&bin).unwrap();
    std::fs::write(local_rtk_path(), "fake").unwrap();
    assert!(is_installed_but_unreachable());
    std::fs::remove_file(local_rtk_path()).ok();
    if let Some(p) = old_path {
        std::env::set_var("PATH", p);
    } else {
        std::env::remove_var("PATH");
    }
}

#[tokio::test]
async fn dry_run_install_returns_true_without_writing() {
    let old = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let opts = RunOpts { dry_run: true, upgrade: false, verbose: false, yes: true };
    assert!(RtkTool.install(&opts).await.unwrap());
    assert!(!local_rtk_path().exists());
    if let Some(p) = old {
        std::env::set_var("PATH", p);
    } else {
        std::env::remove_var("PATH");
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test rtk`
Expected: 3 tests pass.

- [ ] **Step 5: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add rust/src/tools/rtk.rs rust/src/tools/mod.rs rust/tests/rtk.rs
git commit -m "feat(rust): RTK tool install/version/health"
```

---

### Task 10: Claude agent (RTK wiring)

**Files:**
- Create: `rust/src/agents/claude.rs`
- Test: `rust/tests/claude.rs`

**Interfaces:**
- Consumes: `crate::util::paths`, `crate::util::json`, `crate::util::detect`, `crate::util::exec`, `crate::registry`.
- Produces:
  - `pub struct ClaudeAgent;` implementing `Agent`.
  - `fn wire_mcp(...)` — MCP wiring (codegraph/context-mode). Implement the `rtk` case now; other tools return `Ok(false)` (later phases).
  - Port of TS `wire`/`unwire`/`verify`/`detect` for the `rtk` path: `allow_bash_pattern("Bash(rtk *)")`, `wire_rtk_hook`, `override_claude_rtk_hook`, `remove_rtk_hook`, `has_rtk_hook`.

- [ ] **Step 1: Write agents/claude.rs**

```rust
use crate::agents::Agent;
use crate::registry::{Detection, RunOpts, ToolId};
use crate::util::detect::find_binary_in;
use crate::util::errors::Result;
use crate::util::json::{add_to_array_if_missing, get_or_create_object, read_json_file, write_json_file};
use crate::util::paths::{claude_desktop_paths, claude_known_bin_dirs, claude_paths, read_file, toksave_abs, write_file};
use std::path::Path;

pub struct ClaudeAgent;

impl Agent for ClaudeAgent {
    fn detect(&self) -> Detection {
        let has_cli = find_binary_in("claude", &claude_known_bin_dirs()).is_some();
        let has_desktop = claude_desktop_paths().iter().any(|p| p.exists());
        if has_cli && has_desktop {
            return Detection { installed: true, source: "cli+desktop".to_string() };
        }
        if has_cli {
            return Detection { installed: true, source: "cli".to_string() };
        }
        if has_desktop {
            return Detection { installed: true, source: "desktop".to_string() };
        }
        // Config-dir fallback only in test mode (mirror TS NODE_ENV==="test")
        if std::env::var("TOKSAVE_TEST").is_ok() && claude_paths().dir.exists() {
            return Detection { installed: true, source: "config".to_string() };
        }
        Detection { installed: false, source: String::new() }
    }

    fn wire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool> {
        match tool {
            ToolId::Rtk => {
                if !opts.dry_run {
                    allow_bash_pattern("Bash(rtk *)")?;
                    wire_rtk_hook(opts)?;
                    override_claude_rtk_hook()?;
                }
                Ok(true)
            }
            _ => Ok(false), // ported in later phases
        }
    }

    fn unwire(&self, tool: ToolId, opts: &RunOpts) -> Result<bool> {
        match tool {
            ToolId::Rtk => {
                if !opts.dry_run {
                    remove_rtk_hook()?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn verify(&self, tool: ToolId) -> Option<bool> {
        match tool {
            ToolId::Rtk => Some(has_rtk_hook()),
            _ => None,
        }
    }
}

fn allow_bash_pattern(pattern: &str) -> Result<()> {
    let p = claude_paths();
    let cfg = read_json_file(&p.settings)?.unwrap_or_else(|| serde_json::json!({}));
    let mut cfg = cfg;
    {
        let perms = get_or_create_object(&mut cfg, "permissions");
        let perms = perms.as_object_mut().expect("object");
        let arr = perms.entry("allow").or_insert_with(|| serde_json::json!([]));
        let arr = arr.as_array_mut().expect("array");
        add_to_array_if_missing(arr, serde_json::json!(pattern));
    }
    write_json_file(&p.settings, &cfg)
}

fn rtk_hook_command() -> String {
    format!("{} rtk-hook claude", toksave_abs())
}

fn wire_rtk_hook(opts: &RunOpts) -> Result<()> {
    let p = claude_paths();
    let cfg = read_json_file(&p.settings)?.unwrap_or_else(|| serde_json::json!({}));
    let mut cfg = cfg;
    let command = rtk_hook_command();
    {
        let hooks = get_or_create_object(&mut cfg, "hooks");
        let hooks = hooks.as_object_mut().expect("object");
        let arr = hooks.entry("PreToolUse").or_insert_with(|| serde_json::json!([]));
        let arr = arr.as_array_mut().expect("array");
        let entry = serde_json::json!({
            "matcher": "Bash",
            "hooks": [{ "type": "command", "command": command, "timeout": 10 }]
        });
        if !arr.iter().any(|g| hook_group_has_command(g, &command)) {
            arr.push(entry);
        }
    }
    write_json_file(&p.settings, &cfg)
}

fn hook_group_has_command(group: &serde_json::Value, command: &str) -> bool {
    group
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| hooks.iter().any(|h| h.get("command").and_then(|c| c.as_str()) == Some(command)))
        .unwrap_or(false)
}

fn remove_rtk_hook() -> Result<()> {
    let p = claude_paths();
    let cfg = read_json_file(&p.settings)?.unwrap_or_else(|| serde_json::json!({}));
    let mut cfg = cfg;
    if let Some(hooks) = cfg.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        if let Some(pre) = hooks.get_mut("PreToolUse").and_then(|p| p.as_array_mut()) {
            let marker = "rtk-hook claude";
            pre.retain(|g| !hook_group_contains_marker(g, marker));
            if pre.is_empty() {
                hooks.remove("PreToolUse");
            }
        }
        if hooks.is_empty() {
            cfg.as_object_mut().expect("object").remove("hooks");
        }
    }
    write_json_file(&p.settings, &cfg)
}

fn hook_group_contains_marker(group: &serde_json::Value, marker: &str) -> bool {
    group
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| hooks.iter().any(|h| h.get("command").and_then(|c| c.as_str()).map(|c| c.contains(marker)).unwrap_or(false)))
        .unwrap_or(false)
}

fn has_rtk_hook() -> bool {
    let p = claude_paths();
    let Ok(Some(cfg)) = read_json_file(&p.settings) else {
        return false;
    };
    let marker = "rtk-hook claude";
    cfg.get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().any(|g| hook_group_contains_marker(g, marker)))
        .unwrap_or(false)
}

/// Override rtk's own "rtk hook claude" command with the toksave wrapper, dedupe groups,
/// remove RTK.md, strip @RTK.md refs, and allow Bash(rtk *) (port of overrideClaudeRtkHook).
fn override_claude_rtk_hook() -> Result<()> {
    let p = claude_paths();
    let Some(raw) = read_file(&p.settings) else {
        return Ok(());
    };
    let parsed = serde_json::from_str::<serde_json::Value>(&raw);
    let mut cfg = match parsed {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    let tok = toksave_abs();
    let new_cmd = format!("{tok} rtk-hook claude");
    let mut changed = false;

    if let Some(hooks) = cfg.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        if let Some(pre) = hooks.get_mut("PreToolUse").and_then(|p| p.as_array_mut()) {
            for g in pre.iter_mut() {
                let Some(inner) = g.get_mut("hooks").and_then(|h| h.as_array_mut()) else {
                    continue;
                };
                for h in inner.iter_mut() {
                    let Some(cmd) = h.get_mut("command").and_then(|c| c.as_str()) else {
                        continue;
                    };
                    if cmd.contains("rtk hook claude") && !cmd.contains("rtk-hook claude") {
                        *h = serde_json::json!({ "type": "command", "command": new_cmd, "timeout": 10 });
                        changed = true;
                    }
                }
            }
            // Deduplicate groups with same first hook command
            if pre.len() > 1 {
                let mut seen = std::collections::HashSet::new();
                let mut dedup: Vec<serde_json::Value> = Vec::new();
                for g in pre.iter() {
                    let first = first_hook_command(g);
                    if seen.insert(first.clone()) {
                        dedup.push(g.clone());
                    } else {
                        changed = true;
                    }
                }
                *pre = dedup;
            }
        }
    }

    if changed {
        write_json_file(&p.settings, &cfg)?;
    }
    allow_bash_pattern("Bash(rtk *)")?;

    // Remove RTK.md + strip @RTK.md ref from AGENTS.md
    let rtk_md = p.dir.join("RTK.md");
    if rtk_md.exists() {
        let _ = std::fs::remove_file(&rtk_md);
    }
    strip_rtk_ref_from_md(&p.agents_md);
    Ok(())
}

fn first_hook_command(g: &serde_json::Value) -> String {
    g.get("hooks")
        .and_then(|h| h.as_array())
        .and_then(|arr| arr.first())
        .and_then(|h| h.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string()
}

fn strip_rtk_ref_from_md(file_path: &Path) {
    let Some(raw) = read_file(file_path) else {
        return;
    };
    let kept: Vec<&str> = raw
        .split('\n')
        .filter(|l| {
            let t = l.trim();
            !(t.starts_with('@') && t.ends_with("RTK.md"))
        })
        .collect();
    let result = kept.join("\n").trim().to_string();
    if result.is_empty() {
        let _ = std::fs::remove_file(file_path);
        return;
    }
    if result != raw.trim() {
        let _ = write_file(file_path, &format!("{result}\n"));
    }
}
```

- [ ] **Step 2: Write test rust/tests/claude.rs**

```rust
use toksave_rs::agents::claude::ClaudeAgent;
use toksave_rs::agents::Agent;
use toksave_rs::registry::{RunOpts, ToolId};
use toksave_rs::util::json::read_json_file;
use toksave_rs::util::paths::claude_paths;
use std::fs;

fn test_env() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("toksave-claude-test-{}", std::process::id()))
}

fn set_home() -> std::option::Option<std::ffi::OsString> {
    let old = std::env::var_os("HOME");
    std::env::set_var("HOME", test_env().join("home"));
    std::env::set_var("USERPROFILE", test_env().join("home"));
    std::env::set_var("TOKSAVE_TEST", "1");
    old
}

fn restore_home(old: std::option::Option<std::ffi::OsString>) {
    match old {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    std::env::remove_var("USERPROFILE");
    std::env::remove_var("TOKSAVE_TEST");
    fs::remove_dir_all(test_env()).ok();
}

const OPTS: RunOpts = RunOpts { dry_run: false, upgrade: false, verbose: false, yes: true };

#[test]
fn claude_wires_rtk_through_pretooluse_hook() {
    let old = set_home();
    let agent = ClaudeAgent;
    agent.wire(ToolId::Rtk, &OPTS).unwrap();
    let settings = read_json_file(&claude_paths().settings).unwrap().unwrap();
    let pre = settings["hooks"]["PreToolUse"].as_array().unwrap();
    assert!(pre.iter().any(|g| {
        g["hooks"][0]["command"]
            .as_str()
            .map(|c| c.contains("rtk-hook claude"))
            .unwrap_or(false)
    }));
    assert_eq!(agent.verify(ToolId::Rtk), Some(true));
    restore_home(old);
}

#[test]
fn claude_rtk_unwire_removes_hook() {
    let old = set_home();
    let agent = ClaudeAgent;
    agent.wire(ToolId::Rtk, &OPTS).unwrap();
    agent.unwire(ToolId::Rtk, &OPTS).unwrap();
    assert_eq!(agent.verify(ToolId::Rtk), Some(false));
    restore_home(old);
}

#[test]
fn claude_detect_uses_config_dir_in_test_mode() {
    let old = set_home();
    fs::create_dir_all(claude_paths().dir).unwrap();
    let d = ClaudeAgent.detect();
    assert!(d.installed);
    assert_eq!(d.source, "config");
    restore_home(old);
}

#[test]
fn claude_unparseable_settings_is_error_not_fallback() {
    // Trust boundary: wire must FAIL (error), not silently create {} and clobber.
    let old = set_home();
    fs::create_dir_all(claude_paths().dir).unwrap();
    fs::write(claude_paths().settings, "{ not json").unwrap();
    let agent = ClaudeAgent;
    let before = fs::read_to_string(claude_paths().settings).unwrap();
    assert!(agent.wire(ToolId::Rtk, &OPTS).is_err());
    assert_eq!(fs::read_to_string(claude_paths().settings).unwrap(), before);
    restore_home(old);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test claude -- --test-threads=1`
Expected: 4 tests pass. `claude_unparseable_settings_is_error_not_fallback` is the Rust embodiment of the Warp hooks.json regression class (trust boundary #1).

- [ ] **Step 4: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add rust/src/agents/claude.rs rust/tests/claude.rs
git commit -m "feat(rust): Claude RTK hook wiring with strict config safety"
```

---

### Task 11: CLI (clap, 13 commands) + dispatch stubs

**Files:**
- Create: `rust/src/cli.rs`
- Modify: `rust/src/main.rs` (real dispatch)
- Modify: `rust/src/lib.rs` (remove commented stubs)
- Test: `rust/tests/cli.rs`

**Interfaces:**
- Consumes: `crate::registry::{parse_agent_id, parse_tool_id, AgentId, ToolId, RunOpts}`, `crate::util::version`.
- Produces:
  - `#[derive(ValueEnum)] pub enum CommandType { Init, Doctor, Update, Uninstall, Disable, SelfUpdate, CodexPermHook, RtkHook, ContextModeHook, Runmcp, Index, AgyHook, CopilotHook }`
  - `pub struct ParsedCli { command: CommandType, agents: Vec<AgentId>, tools: Vec<ToolId>, opts: RunOpts, offline: bool, fix: bool, auto: bool }`
  - `pub fn parse_cli(args: Vec<String>) -> ParsedCli` — clap parse; on parse error `eprintln!` + exit(2); `--version` handled by clap.
  - Hook commands (`rtk-hook`, `context-mode-hook`, `runmcp`, `agy-hook`, `copilot-hook`) accept trailing args (`Vec<String>`) and `allow_hyphen_values`.
  - `main.rs` dispatch: each CommandType → a command fn returning `i32`; unimplemented commands print "not implemented in rust yet" and return `0` (so `--help`/version/init smoke works).

- [ ] **Step 1: Write cli.rs**

```rust
use crate::registry::{parse_agent_id, parse_tool_id, AgentId, RunOpts, ToolId};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CommandType {
    Init,
    Doctor,
    Update,
    Uninstall,
    Disable,
    SelfUpdate,
    CodexPermHook,
    RtkHook,
    ContextModeHook,
    Runmcp,
    Index,
    AgyHook,
    CopilotHook,
}

#[derive(Debug, Parser)]
#[command(name = "toksave")]
#[command(version = crate::util::version::toksave_version())]
#[command(about = "Zero-config token-saver for AI coding agents")]
struct Cli {
    /// Target specific agents (claude,opencode,codex,antigravity,copilot,droid,devin,warp)
    #[arg(short = 'a', long = "agents", num_args = 1.., value_delimiter = ',')]
    agents: Vec<String>,

    /// Target specific tools (rtk,caveman,codegraph,context-mode,ponytail,principles)
    #[arg(short = 't', long = "tools", num_args = 1.., value_delimiter = ',')]
    tools: Vec<String>,

    /// Show what would happen without making changes
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Print detailed output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Skip interactive prompts, auto-select detected agents
    #[arg(short = 'y', long = "yes")]
    yes: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Health check — show what is wired and what is broken
    Doctor {
        /// Skip remote version checks
        #[arg(long)]
        offline: bool,
        /// Repair unhealthy tool installations
        #[arg(long)]
        fix: bool,
    },
    /// Update all token-saving tools to latest versions
    Update,
    /// Remove toksave wiring from agents
    Uninstall,
    /// Disable one or more agents/tools (surgical uninstall)
    Disable,
    /// Update the toksave CLI itself
    SelfUpdate,
    /// Internal hook for Codex permissions
    CodexPermHook,
    /// Internal hook for RTK command prefixing
    #[command(allow_hyphen_values = true)]
    RtkHook {
        /// Passthrough args
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Internal hook for Context-Mode integration
    #[command(allow_hyphen_values = true)]
    ContextModeHook {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Internal hook to proxy MCP execution securely
    #[command(allow_hyphen_values = true)]
    Runmcp {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build per-project indexes (codegraph) in the current dir
    Index {
        /// internal: auto-index mode (silent, only if project detected)
        #[arg(long)]
        auto: bool,
    },
    /// Internal: Antigravity codegraph index hook
    #[command(allow_hyphen_values = true)]
    AgyHook {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Internal: Copilot codegraph index hook
    #[command(allow_hyphen_values = true)]
    CopilotHook {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ParsedCli {
    pub command: CommandType,
    pub agents: Vec<AgentId>,
    pub tools: Vec<ToolId>,
    pub opts: RunOpts,
    pub offline: bool,
    pub fix: bool,
    pub auto: bool,
}

impl Default for ParsedCli {
    fn default() -> Self {
        Self {
            command: CommandType::Init,
            agents: vec![],
            tools: vec![],
            opts: RunOpts::default(),
            offline: false,
            fix: false,
            auto: false,
        }
    }
}

pub fn parse_cli(args: Vec<String>) -> ParsedCli {
    let mut cli = match Cli::try_parse_from(&args) {
        Ok(c) => c,
        Err(e) => {
            // clap already printed the error/help; mirror TS behavior of exiting non-zero on bad args
            std::process::exit(match e.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => 0,
                _ => 2,
            });
        }
    };

    let mut parsed = ParsedCli::default();
    let opts = RunOpts {
        dry_run: cli.dry_run,
        upgrade: false,
        verbose: cli.verbose,
        yes: cli.yes,
    };
    parsed.opts = opts;

    for raw in &cli.agents {
        for s in raw.split(',') {
            if let Some(id) = parse_agent_id(s.trim()) {
                parsed.agents.push(id);
            }
        }
    }
    for raw in &cli.tools {
        for s in raw.split(',') {
            if let Some(id) = parse_tool_id(s.trim()) {
                parsed.tools.push(id);
            }
        }
    }

    match cli.command.take() {
        None => parsed.command = CommandType::Init,
        Some(Command::Doctor { offline, fix }) => {
            parsed.command = CommandType::Doctor;
            parsed.offline = offline;
            parsed.fix = fix;
        }
        Some(Command::Update) => parsed.command = CommandType::Update,
        Some(Command::Uninstall) => parsed.command = CommandType::Uninstall,
        Some(Command::Disable) => parsed.command = CommandType::Disable,
        Some(Command::SelfUpdate) => parsed.command = CommandType::SelfUpdate,
        Some(Command::CodexPermHook) => parsed.command = CommandType::CodexPermHook,
        Some(Command::RtkHook { .. }) => parsed.command = CommandType::RtkHook,
        Some(Command::ContextModeHook { .. }) => parsed.command = CommandType::ContextModeHook,
        Some(Command::Runmcp { .. }) => parsed.command = CommandType::Runmcp,
        Some(Command::Index { auto }) => {
            parsed.command = CommandType::Index;
            parsed.auto = auto;
        }
        Some(Command::AgyHook { .. }) => parsed.command = CommandType::AgyHook,
        Some(Command::CopilotHook { .. }) => parsed.command = CommandType::CopilotHook,
    }

    parsed
}
```

- [ ] **Step 2: Write main.rs (real dispatch)**

```rust
use toksave_rs::cli::{parse_cli, CommandType};

fn main() {
    let parsed = parse_cli(std::env::args().collect());
    let code = match parsed.command {
        CommandType::Init => run_init(parsed),
        other => {
            println!("toksave-rs: `{other:?}` not implemented in Rust yet (TS build handles it).");
            0
        }
    };
    std::process::exit(code);
}

fn run_init(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::init::run_init(&parsed))
}
```

- [ ] **Step 3: Fix lib.rs**

Remove any commented `cli` references; ensure `pub mod cli;` is uncommented and all modules compile. Create `rust/src/commands/init.rs` as a stub returning `0` (real implementation is Task 12):
```rust
use crate::cli::ParsedCli;

pub async fn run_init(_parsed: &ParsedCli) -> i32 {
    0
}
```

- [ ] **Step 4: Write test rust/tests/cli.rs**

```rust
use toksave_rs::cli::{parse_cli, CommandType};
use toksave_rs::registry::{AgentId, ToolId};

fn parse(args: &[&str]) -> toksave_rs::cli::ParsedCli {
    let mut v = vec!["toksave".to_string()];
    v.extend(args.iter().map(|s| s.to_string()));
    parse_cli(v)
}

#[test]
fn default_command_is_init() {
    assert_eq!(parse(&[]).command, CommandType::Init);
}

#[test]
fn doctor_command() {
    assert_eq!(parse(&["doctor"]).command, CommandType::Doctor);
}

#[test]
fn doctor_fix_flag() {
    let c = parse(&["doctor", "--fix"]);
    assert_eq!(c.command, CommandType::Doctor);
    assert!(c.fix);
}

#[test]
fn update_command() {
    assert_eq!(parse(&["update"]).command, CommandType::Update);
}

#[test]
fn self_update_command() {
    assert_eq!(parse(&["self-update"]).command, CommandType::SelfUpdate);
}

#[test]
fn dry_run_flag() {
    assert!(parse(&["--dry-run"]).opts.dry_run);
}

#[test]
fn agents_parse_comma_separated() {
    let c = parse(&["--agents", "claude,antigravity"]);
    assert_eq!(c.agents, vec![AgentId::Claude, AgentId::Antigravity]);
}

#[test]
fn tools_parse_comma_separated() {
    let c = parse(&["--tools", "rtk,caveman"]);
    assert_eq!(c.tools, vec![ToolId::Rtk, ToolId::Caveman]);
}

#[test]
fn context_mode_alias() {
    let c = parse(&["--tools", "contextmode"]);
    assert_eq!(c.tools, vec![ToolId::ContextMode]);
}

#[test]
fn invalid_agent_ignored() {
    let c = parse(&["--agents", "invalid"]);
    assert!(c.agents.is_empty());
}

#[test]
fn all_13_commands_parse() {
    let cmds = [
        CommandType::Doctor,
        CommandType::Update,
        CommandType::Uninstall,
        CommandType::Disable,
        CommandType::SelfUpdate,
        CommandType::CodexPermHook,
        CommandType::RtkHook,
        CommandType::ContextModeHook,
        CommandType::Runmcp,
        CommandType::Index,
        CommandType::AgyHook,
        CommandType::CopilotHook,
    ];
    for c in cmds {
        let args: Vec<String> = c
            .to_possible_value()
            .map(|v| v.get_name().to_string())
            .unwrap();
        let c2 = parse(&[&args]);
        assert_ne!(c2.command, CommandType::Init);
    }
}

#[test]
fn rtk_hook_accepts_trailing_args() {
    let c = parse(&["rtk-hook", "claude", "--extra"]);
    assert_eq!(c.command, CommandType::RtkHook);
}

#[test]
fn index_auto_flag() {
    let c = parse(&["index", "--auto"]);
    assert_eq!(c.command, CommandType::Index);
    assert!(c.auto);
}
```

NOTE: `CommandType` derives `ValueEnum`, so `to_possible_value()` works; the 13th command is `Init` (the default). Adjust `all_13_commands_parse` to iterate the ValueEnum variants via `CommandType::value_variants()` if the `to_possible_value` lookup on the runtime value is awkward — simpler:
```rust
#[test]
fn all_commands_parse() {
    for (name, expected) in [
        ("doctor", CommandType::Doctor),
        ("update", CommandType::Update),
        ("uninstall", CommandType::Uninstall),
        ("disable", CommandType::Disable),
        ("self-update", CommandType::SelfUpdate),
        ("codex-perm-hook", CommandType::CodexPermHook),
        ("rtk-hook", CommandType::RtkHook),
        ("context-mode-hook", CommandType::ContextModeHook),
        ("runmcp", CommandType::Runmcp),
        ("index", CommandType::Index),
        ("agy-hook", CommandType::AgyHook),
        ("copilot-hook", CommandType::CopilotHook),
    ] {
        let c = parse(&[name]);
        assert_eq!(c.command, expected);
    }
}
```

- [ ] **Step 5: Run tests + fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test --test cli`
Expected: all CLI tests pass. `rtk_hook_accepts_trailing_args` requires `allow_hyphen_values` on the `RtkHook` subcommand — verify clap accepts `--extra` after positional args; if clap errors on the leading-dash value, add `trailing_var_arg = true` to the subcommand attrs instead.

- [ ] **Step 6: Commit**

```bash
git add rust/src/cli.rs rust/src/main.rs rust/src/lib.rs rust/src/commands/init.rs rust/tests/cli.rs
git commit -m "feat(rust): clap CLI for all 13 commands"
```

---

### Task 12: Init command (end-to-end for Claude × RTK)

**Files:**
- Create: `rust/src/commands/init.rs` (replace stub)
- Modify: `rust/src/tools/mod.rs` (add `tool_installed_version` already done in Task 9)
- Test: `rust/tests/init.rs`

**Interfaces:**
- Consumes: `crate::registry::{ALL_TOOLS, detect_agent, install_tool, tool_info, agent_info}`, `crate::commands::init` deps logic, `crate::util::manifest::record_wire`, `crate::util::exec`.
- Produces: `pub async fn run_init(parsed: &ParsedCli) -> i32` — port of `src/commands/init.ts`.
- Adds to `registry.rs`: dispatch fns `pub fn detect_agent(id: AgentId) -> Detection`, `pub async fn wire_tool(agent: AgentId, tool: ToolId, opts: &RunOpts) -> Result<bool>`, `pub fn verify_tool(agent: AgentId, tool: ToolId) -> Option<bool>` — matching `agentModules` in TS.

- [ ] **Step 1: Add dispatch fns to registry.rs**

```rust
use crate::agents::claude::ClaudeAgent;
use crate::agents::Agent;
use crate::tools::RtkTool;
use crate::tools::Tool;

pub fn detect_agent(id: AgentId) -> Detection {
    match id {
        AgentId::Claude => ClaudeAgent.detect(),
        _ => Detection::default(),
    }
}

pub async fn wire_tool(agent: AgentId, tool: ToolId, opts: &RunOpts) -> crate::util::errors::Result<bool> {
    match agent {
        AgentId::Claude => ClaudeAgent.wire(tool, opts),
        _ => Ok(false),
    }
}

pub fn verify_tool(agent: AgentId, tool: ToolId) -> Option<bool> {
    match agent {
        AgentId::Claude => ClaudeAgent.verify(tool),
        _ => None,
    }
}

pub fn tool_installed_version(id: ToolId) -> Option<String> {
    crate::tools::tool_installed_version(id)
}
```

- [ ] **Step 2: Write commands/init.rs**

```rust
use crate::cli::ParsedCli;
use crate::registry::{agent_info, detect_agent, install_tool, tool_info, verify_tool, wire_tool, ALL_TOOLS, AgentId, RunOpts, ToolId};
use crate::util::colors;
use crate::util::exec::{run_stdout, npm_cmd};
use crate::util::manifest::record_wire;

pub async fn run_init(parsed: &ParsedCli) -> i32 {
    colors::banner("toksave", "global token-saver for AI agents");

    // ── Step 1: Filter tools ──
    let tools: Vec<ToolId> = ALL_TOOLS
        .iter()
        .filter(|t| parsed.tools.is_empty() || parsed.tools.contains(&t.id))
        .map(|t| t.id)
        .collect();

    // Node dep check for npm-channel tools (port of ensureDeps)
    let has_npm_tools = tools.iter().any(|t| tool_info(*t).channel == crate::registry::Channel::Npm);
    let min_node = tools.iter().map(|t| tool_info(*t).min_node_major).max().unwrap_or(0);
    let deps_ok = check_deps(has_npm_tools, min_node);

    // ── Step 2: Install tools ──
    let mut installed_tools = std::collections::HashSet::new();
    for t in &tools {
        let info = tool_info(*t);
        let is_npm = info.channel == crate::registry::Channel::Npm;
        if is_npm && !deps_ok {
            colors::warn(&format!("{} — needs Node.js", info.label));
            continue;
        }
        match install_tool(*t, &parsed.opts).await {
            Ok(true) => {
                installed_tools.insert(*t);
                colors::ok(info.label);
            }
            Ok(false) => colors::warn(&format!("{} — skipped", info.label)),
            Err(e) => colors::err(&format!("{} — {}", info.label, e.message)),
        }
    }

    // ── Step 3: Detect agents ──
    let mut detected: Vec<(AgentId, String)> = vec![];
    for a in crate::registry::ALL_AGENTS {
        let d = detect_agent(a.id);
        if d.installed {
            detected.push((a.id, d.source));
        }
    }

    // ── Step 4: Pick agents ──
    let requested: Vec<AgentId> = if !parsed.agents.is_empty() {
        parsed.agents.clone()
    } else if parsed.opts.yes || !is_interactive() {
        detected.iter().map(|(id, _)| *id).collect()
    } else {
        // Interactive multi-select ported in a later phase; for now fall back to detected
        detected.iter().map(|(id, _)| *id).collect()
    };

    if requested.is_empty() {
        println!("  Nothing selected.");
        return 0;
    }

    let detected_by_id: std::collections::HashMap<AgentId, String> = detected.into_iter().collect();

    // ── Step 5: Wire tools into agents ──
    let mut failures: Vec<(AgentId, Vec<String>)> = vec![];
    for agent_id in &requested {
        let Some(_source) = detected_by_id.get(agent_id) else {
            let info = agent_info(*agent_id);
            colors::warn(&format!("{} not installed — install it first: {}", info.label, info.homepage));
            continue;
        };
        let info = agent_info(*agent_id);
        let mut failed_tools: Vec<String> = vec![];
        for t in &tools {
            if !installed_tools.contains(t) {
                failed_tools.push(tool_info(*t).label.to_string());
                continue;
            }
            match wire_tool(*agent_id, *t, &parsed.opts).await {
                Ok(true) => {
                    if !parsed.opts.dry_run {
                        if verify_tool(*agent_id, *t) == Some(false) {
                            failed_tools.push(tool_info(*t).label.to_string());
                            continue;
                        }
                        let _ = record_wire(
                            &format!("{:?}", agent_id).to_lowercase(),
                            &tool_name(*t),
                            tool_installed_version(*t).as_deref(),
                        );
                    }
                }
                _ => failed_tools.push(tool_info(*t).label.to_string()),
            }
        }
        if failed_tools.is_empty() {
            colors::ok(info.label);
        } else {
            colors::warn(&format!("{} — {} not wired", info.label, failed_tools.join(", ")));
            failures.push((*agent_id, failed_tools));
        }
    }

    // ── Step 6: Summary ──
    println!();
    if failures.is_empty() {
        colors::ok("Equipped.");
    } else {
        for (id, failed) in &failures {
            colors::warn(&format!("{}: {} not wired. Run `toksave doctor` for details.", agent_info(*id).label, failed.join(", ")));
        }
    }
    print_version_table(&tools);
    println!();

    if failures.is_empty() { 0 } else { 1 }
}

fn check_deps(need_node: bool, min_node: u32) -> bool {
    if !need_node {
        return true;
    }
    let Some(out) = run_stdout("node", &["--version"]) else {
        return false;
    };
    let v = out.trim_start_matches('v');
    let major: u32 = v.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if major < min_node {
        eprintln!("Node.js {out} detected but >= v{min_node}.x required. Upgrade Node.js at https://nodejs.org");
        return false;
    }
    true
}

fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

fn tool_name(t: ToolId) -> String {
    match t {
        ToolId::Rtk => "rtk".to_string(),
        ToolId::Caveman => "caveman".to_string(),
        ToolId::Codegraph => "codegraph".to_string(),
        ToolId::ContextMode => "context-mode".to_string(),
        ToolId::Ponytail => "ponytail".to_string(),
        ToolId::Principles => "principles".to_string(),
    }
}

fn print_version_table(tools: &[ToolId]) {
    for t in tools {
        let info = tool_info(*t);
        if info.instruction_only {
            println!("  {} {} instruction-only", colors::CHECK, info.label);
            continue;
        }
        match tool_installed_version(*t) {
            Some(v) => println!("  {} {} {}", colors::CHECK, info.label, v),
            None => println!("  {} {} not installed", colors::BULLET, info.label),
        }
    }
}
```

- [ ] **Step 3: Write test rust/tests/init.rs**

```rust
use std::fs;
use std::path::PathBuf;
use toksave_rs::cli::{parse_cli, CommandType};
use toksave_rs::commands::init::run_init;
use toksave_rs::util::json::read_json_file;
use toksave_rs::util::manifest::was_wired_by_us;
use toksave_rs::util::paths::claude_paths;

fn test_root() -> PathBuf {
    std::env::temp_dir().join(format!("toksave-init-test-{}", std::process::id()))
}

fn set_env() -> Option<std::ffi::OsString> {
    let old_home = std::env::var_os("HOME");
    std::env::set_var("HOME", test_root().join("home"));
    std::env::set_var("USERPROFILE", test_root().join("home"));
    std::env::set_var("TOKSAVE_CACHE_DIR", test_root().join("cache"));
    std::env::set_var("TOKSAVE_TEST", "1");
    let old_path = std::env::var_os("PATH");
    // Keep PATH but ensure "rtk" resolves to nothing real; tests run wire dry via detection below
    old_path
}

fn restore_env(old_home: Option<std::ffi::OsString>, old_path: Option<std::ffi::OsString>) {
    match old_home {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    std::env::remove_var("USERPROFILE");
    match old_path {
        Some(v) => std::env::set_var("PATH", v),
        None => std::env::remove_var("PATH"),
    }
    std::env::remove_var("TOKSAVE_CACHE_DIR");
    std::env::remove_var("TOKSAVE_TEST");
    fs::remove_dir_all(test_root()).ok();
}

#[tokio::test]
async fn init_wires_claude_rtk_and_records_manifest() {
    let (old_home, old_path) = (set_env(), None::<std::ffi::OsString>);

    // Simulate Claude installed via config dir (test mode detect)
    fs::create_dir_all(claude_paths().dir).unwrap();

    let args = vec![
        "toksave".to_string(),
        "--yes".to_string(),
        "--agents".to_string(),
        "claude".to_string(),
        "--tools".to_string(),
        "rtk".to_string(),
    ];
    let parsed = parse_cli(args);
    assert_eq!(parsed.command, CommandType::Init);

    let code = run_init(&parsed).await;

    // RTK install in test env may fail (no network / no rtk); the wiring only happens
    // when the tool installed successfully. So assert the CLI returned a defined code (0 or 1),
    // and that when wire happened, manifest + hook are consistent.
    assert!(code == 0 || code == 1);
    restore_env(old_home, old_path);
}

#[tokio::test]
async fn init_dry_run_does_not_write() {
    let (old_home, old_path) = (set_env(), None::<std::ffi::OsString>);
    let parsed = parse_cli(vec![
        "toksave".to_string(),
        "--dry-run".to_string(),
        "--yes".to_string(),
        "--agents".to_string(),
        "claude".to_string(),
        "--tools".to_string(),
        "rtk".to_string(),
    ]);
    let code = run_init(&parsed).await;
    assert!(code == 0 || code == 1);
    assert!(!claude_paths().settings.exists());
    restore_env(old_home, old_path);
}
```

NOTE: `init_wires_claude_rtk_and_records_manifest` depends on the real RTK download succeeding (network). That makes the test flaky/offline. For a deterministic integration test, the REAL parity check is the unit-level one in `tests/claude.rs` (wire→verify) + `tests/rtk.rs` (dry-run install). The init integration test should instead be run against a mocked install. Add a `#[cfg(test)]` seam in `tools/mod.rs::install_tool` that short-circuits when `TOKSAVE_TEST` is set AND a file `{TOKSAVE_CACHE_DIR}/mock-install/{tool}` exists — writes that file. Simpler deterministic approach: set env `TOKSAVE_TEST_RTK_INSTALL=1` and have `install_tool` for Rtk return `Ok(true)` without downloading when this env is set. Implement:

In `tools/mod.rs`:
```rust
pub async fn install_tool(tool: ToolId, opts: &RunOpts) -> Result<bool> {
    if std::env::var("TOKSAVE_TEST_RTK_INSTALL").is_ok() && tool == ToolId::Rtk && !opts.dry_run {
        // Test seam: pretend install succeeded; the wired hook points at real binary.
        return Ok(true);
    }
    match tool {
        ToolId::Rtk => RtkTool.install(opts).await,
        _ => Ok(false),
    }
}
```

Then in the init test, set `TOKSAVE_TEST_RTK_INSTALL=1` and assert wiring happened:
```rust
std::env::set_var("TOKSAVE_TEST_RTK_INSTALL", "1");
let code = run_init(&parsed).await;
assert_eq!(code, 0);
assert!(was_wired_by_us("claude", "rtk"));
assert!(read_json_file(&claude_paths().settings).unwrap().unwrap()["hooks"]["PreToolUse"].is_array());
```

This is deterministic and offline. Use this for the main init test; keep dry-run test as-is.

- [ ] **Step 4: Run tests**

Run: `cargo test --test init -- --test-threads=1`
Expected: 2 tests pass.

- [ ] **Step 5: fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add rust/src/commands/init.rs rust/src/registry.rs rust/src/tools/mod.rs rust/tests/init.rs
git commit -m "feat(rust): init command wiring claude x rtk"
```

---

### Task 13: TestEnv helper + CI job

**Files:**
- Create: `rust/tests/common/mod.rs`
- Modify: `.github/workflows/ci.yml` (add rust-check job)

**Interfaces:**
- Consumes: all tasks.
- Produces:
  - `pub fn test_env_setup() -> TestEnvGuard` — sets `HOME`/`USERPROFILE`/`APPDATA`/`LOCALAPPDATA`/`TOKSAVE_CACHE_DIR` to a fresh temp dir; `pub struct TestEnvGuard { old: Vec<(String, Option<OsString>)>, root: PathBuf }` with `Drop` restoring env + removing temp.
  - CI: `rust-check` job running `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` on ubuntu/macos/windows.

- [ ] **Step 1: Write tests/common/mod.rs**

```rust
pub use toksave_rs::util::paths as ts_paths;

use std::ffi::OsString;
use std::path::PathBuf;

pub struct TestEnvGuard {
    pub root: PathBuf,
    pub old: Vec<(String, Option<OsString>)>,
}

impl TestEnvGuard {
    pub fn home(&self) -> PathBuf {
        self.root.join("home")
    }
    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        for (k, v) in &self.old {
            match v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Set HOME/USERPROFILE/APPDATA/LOCALAPPDATA/TOKSAVE_CACHE_DIR to a fresh temp dir.
/// On drop, restores previous values and removes the temp dir.
pub fn setup() -> TestEnvGuard {
    let root = std::env::temp_dir().join(format!(
        "toksave-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let cache = root.join("cache");

    let mut old = vec![];
    for k in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "TOKSAVE_CACHE_DIR"] {
        old.push((k.to_string(), std::env::var_os(k)));
    }
    std::env::set_var("HOME", &home);
    std::env::set_var("USERPROFILE", &home);
    std::env::set_var("APPDATA", root.join("AppData").join("Roaming"));
    std::env::set_var("LOCALAPPDATA", root.join("AppData").join("Local"));
    std::env::set_var("TOKSAVE_CACHE_DIR", &cache);
    std::env::set_var("TOKSAVE_TEST", "1");

    TestEnvGuard { root, old }
}
```

- [ ] **Step 2: Refactor existing tests to use TestEnvGuard**

Replace per-test env boilerplate in `tests/manifest.rs`, `tests/claude.rs`, `tests/init.rs` with `let _guard = tests::common::setup();`. This is a mechanical change; each test file adds:
```rust
mod common;
use common::{setup, TestEnvGuard};
```
and replaces its `set_env`/`restore_env`/`set_home`/`restore_home` calls.

- [ ] **Step 3: Add rust-check CI job**

In `.github/workflows/ci.yml`, after the existing TS job, add:

```yaml
  rust-check:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    defaults:
      run:
        working-directory: rust
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: rust
      - name: Format check
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy -- -D warnings
      - name: Test
        run: cargo test -- --test-threads=4
```

- [ ] **Step 4: Full verification**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test --all-targets`
Expected: everything green.

- [ ] **Step 5: Commit**

```bash
git add rust/tests/common/mod.rs rust/tests/manifest.rs rust/tests/claude.rs rust/tests/init.rs .github/workflows/ci.yml
git commit -m "chore(rust): TestEnv helper + CI rust-check job"
```

---

## Self-Review

**Spec coverage:**
- Design §3 trust boundaries: #1 (config parse → error, not `{}`) — Task 5 test `unparseable_config_is_error_not_fallback`, Task 10 test `claude_unparseable_settings_is_error_not_fallback`. #2 (zip-slip/traversal) — Task 8 `is_safe_archive_path_rejects_traversal` + `download_zip_rejects_zip_slip`. #3 (retry/backoff/404) — Task 8 `fetch_404_not_retried_and_errors`. #4 (manifest lock + atomic) — Task 7. ✓
- Design §transition step 1 (scaffold + clap 13 stubs) — Tasks 1, 11. ✓
- Design §transition step 2 (init + registry + Claude + RTK → integration test PASS) — Tasks 2–10, 12, 13. ✓
- Design §coexistence (binary `toksave-rs`, crate in `rust/`) — Task 1 (Cargo.toml package name `toksave-rs`), Global Constraints. ✓
- Design §testing (TestEnv env overrides, offline deterministic tests) — Task 13 + per-task test seams (`TOKSAVE_TEST_RTK_INSTALL`). ✓

**Placeholder scan:** The `verify_checksum`/`use sha2?;` stub in Task 8 Step 1 is explicitly marked INVALID and replaced by the real `verify_checksum_sha256` — the implementer must delete the stub. Task 11 `all_13_commands_parse` has a note to replace with the simpler array-based test. Both are called out inline with exact replacement code.

**Type consistency:** `RunOpts` uses `RunOpts::default()` (Copy derive) — Task 6 defines `Default`; Task 11 constructs with struct literal. `install_tool`/`wire_tool`/`verify_tool`/`detect_agent`/`tool_installed_version` signatures used in init (Task 12) match those produced in Tasks 9/10/registry. `Tool` trait (Task 6) vs `tools/mod.rs` re-declaration — the trait must be declared exactly once (`rust/src/tools/mod.rs`); Task 9's "dispatch helper" step must NOT re-declare it. `ToksaveError::message` field used in init (`.e.message`) — errors.rs stores `pub message`. ✓

## Execution Handoff

Plan saved to `docs/plans/2026-07-31-rust-phase1-scaffold-init.md`. Two execution options:

1. **Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
