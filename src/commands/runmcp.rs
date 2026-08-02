//! `toksave runmcp` — resolve an MCP server executable (bare tool name or
//! script path) and proxy execution, transparently routing Node shebang
//! scripts through `node`. Mirrors TS src/commands/runmcp.ts.

use crate::util::detect::{find_binary, find_binary_in, resolve_node};
use crate::util::paths;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// True when the file at `path` has a Node shebang (`#!/usr/bin/env node`
/// or any `#!...node[.exe]` line). Non-readable files are not Node scripts.
pub fn is_node_shebang_script(path: &Path) -> bool {
    let mut buf = [0u8; 64];
    let n = match fs::File::open(path).and_then(|mut f| f.read(&mut buf)) {
        Ok(n) => n,
        Err(_) => return false,
    };
    let head = String::from_utf8_lossy(&buf[..n]);
    if head.starts_with("#!/usr/bin/env node") {
        return true;
    }
    for line in head.lines() {
        let Some(rest) = line.strip_prefix("#!") else {
            break;
        };
        if rest
            .split(['/', '\\', ' ', '\t'])
            .any(|w| matches!(w.trim_end_matches('\r'), "node" | "node.exe"))
        {
            return true;
        }
    }
    false
}

/// Expand PATH so GUI-launched agents (minimal env) still find node/npm tools.
/// Mutates process env `PATH`; returns the new PATH string.
pub fn ensure_tool_path() -> String {
    let home = paths::home();
    let mut extras: Vec<PathBuf> = vec![
        home.join(".local").join("bin"),
        home.join(".bun").join("bin"),
        home.join(".cargo").join("bin"),
    ];
    if !cfg!(windows) {
        extras.push(PathBuf::from("/usr/local/bin"));
        extras.push(PathBuf::from("/opt/homebrew/bin"));
    }
    // mise-managed node installs, newest version first
    let mise_node_root = home
        .join(".local")
        .join("share")
        .join("mise")
        .join("installs")
        .join("node");
    let node_bin = if cfg!(windows) { "node.exe" } else { "node" };
    if let Ok(rd) = fs::read_dir(&mise_node_root) {
        let mut bins: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path().join("bin"))
            .filter(|b| b.join(node_bin).is_file())
            .collect();
        bins.sort();
        bins.reverse();
        extras.extend(bins);
    }
    let mise_shims = home.join(".local").join("share").join("mise").join("shims");
    if mise_shims.is_dir() {
        extras.push(mise_shims);
    }

    let path_var = env::var_os("PATH").unwrap_or_default();
    let mut parts: Vec<PathBuf> = env::split_paths(&path_var).collect();
    for dir in extras {
        if !dir.as_os_str().is_empty() && !parts.contains(&dir) {
            parts.insert(0, dir);
        }
    }
    let next = env::join_paths(&parts)
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    // Safe: called on the main thread before any worker threads spawn the child.
    unsafe { env::set_var("PATH", &next) };
    next
}

/// Bare tool name accepted from agent MCP configs: resolve through known user
/// bin dirs (GUI agents often lack mise/bun/cargo dirs on PATH).
pub fn resolve_mcp_executable(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let p = Path::new(name);
    if p.is_absolute() || name.starts_with("./") || name.starts_with("../") {
        return p.exists().then(|| name.to_string());
    }
    if let Some(found) = find_binary(name) {
        return Some(found);
    }
    let home = paths::home();
    let extra = vec![
        paths::local_bin(),
        paths::local_bin()
            .join("..")
            .join("share")
            .join("mise")
            .join("shims"),
        home.join(".bun").join("bin"),
        home.join(".cargo").join("bin"),
    ];
    find_binary_in(name, &extra)
}

/// Pre-index: fire-and-forget `codegraph sync` when project has `.codegraph`.
fn pre_index_if_needed(project_root: &Path) {
    if env::var("TOKSAVE_TEST").is_ok() {
        return;
    }
    let Some(bin) = resolve_mcp_executable("codegraph").or_else(|| find_binary("codegraph")) else {
        return;
    };
    if !project_root.join(".codegraph").exists() {
        return;
    }
    let _ = Command::new(bin)
        .arg("sync")
        .current_dir(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Split out the `--agent <id>` flag. Returns (agent, remaining args).
pub fn parse_agent_flag(args: &[String]) -> (String, Vec<String>) {
    let mut rest = Vec::with_capacity(args.len());
    let mut agent = String::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--agent" && i + 1 < args.len() {
            agent = args[i + 1].clone();
            i += 2;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    (agent, rest)
}

const USAGE: &str = "Usage: toksave runmcp [--agent <id>] <script_path|tool> [args...]";

/// Run `toksave runmcp ...`. Blocks until the child exits; returns its code.
pub fn run(args: &[String]) -> i32 {
    ensure_tool_path();
    if args.is_empty() {
        eprintln!("{USAGE}");
        return 1;
    }
    let (_agent, args) = parse_agent_flag(args);

    // Pre-index: auto sync per-project codegraph index when present.
    if let Ok(cwd) = env::current_dir() {
        let root = crate::commands::hooks::agy::find_project_dir(&cwd);
        pre_index_if_needed(&root);
    }

    let Some(requested) = args.first() else {
        eprintln!("{USAGE}");
        return 1;
    };

    let mut exe = resolve_mcp_executable(requested).unwrap_or_else(|| requested.clone());
    let mut cmd_args: Vec<String> = args[1..].to_vec();

    // Bare name lookup fallback when resolution returned the raw name.
    let p = Path::new(&exe);
    if !p.is_absolute()
        && !exe.contains('/')
        && !exe.contains('\\')
        && let Some(found) = find_binary(&exe).or_else(|| find_binary_in(&exe, &[]))
    {
        exe = found;
    }

    if !Path::new(&exe).exists() && find_binary(&exe).is_none() {
        eprintln!("Error: MCP executable not found: {requested}");
        eprintln!(
            "Install the tool (e.g. npm i -g @colbymchenry/codegraph context-mode) and ensure PATH includes its bin dir."
        );
        return 1;
    }

    if Path::new(&exe).is_file() && is_node_shebang_script(Path::new(&exe)) {
        cmd_args.insert(0, exe.clone());
        match resolve_node() {
            Some(node) => exe = node,
            None => {
                eprintln!("Error: Could not find 'node' executable on PATH to run MCP server.");
                return 1;
            }
        }
    }

    let mut child = match Command::new(&exe)
        .args(&cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: failed to start MCP server: {e}");
            return 1;
        }
    };

    // Proxy child stdout to our stdout in a thread.
    let pump = child.stdout.take().map(|mut out| {
        std::thread::spawn(move || {
            let mut sink = std::io::stdout();
            let _ = std::io::copy(&mut out, &mut sink);
        })
    });

    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: failed to start MCP server: {e}");
            return 1;
        }
    };
    if let Some(h) = pump {
        let _ = h.join();
    }
    status.code().unwrap_or(1)
}
