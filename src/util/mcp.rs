//! MCP wiring health. Key-exists is not enough: leftover `/$bunfs/` paths,
//! dead binaries, and wrong `runmcp` argv all count as unwired so
//! `toksave doctor --fix` will rewrite them.
//!
//! Backslash Windows paths are *not* stale: cmd.exe, Windows PowerShell 5.1,
//! and pwsh 7 all need them as the first token of a shell command. MCP argv
//! still uses `toksave_abs()` (forward slashes / CreateProcess).

use crate::registry::{AgentId, ToolId};
use crate::util::paths::toksave_abs;
use serde_json::Value;
use toml_edit::DocumentMut;

const CODEGRAPH_ARGS: &[&str] = &["runmcp", "codegraph", "serve", "--mcp"];
const CONTEXT_MODE_ARGS: &[&str] = &["runmcp", "context-mode"];

pub fn mcp_server_name(tool: ToolId) -> Option<&'static str> {
    match tool {
        ToolId::Codegraph => Some("codegraph"),
        ToolId::ContextMode => Some("context-mode"),
        _ => None,
    }
}

/// Expected `args` (or OpenCode command-array tail) for a toksave-managed MCP entry.
pub fn expected_runmcp_args(agent: AgentId, tool: ToolId) -> Option<&'static [&'static str]> {
    match (agent, tool) {
        (AgentId::Claude | AgentId::Copilot | AgentId::Opencode, ToolId::Codegraph) => {
            Some(CODEGRAPH_ARGS)
        }
        (AgentId::Claude | AgentId::Copilot | AgentId::Opencode, ToolId::ContextMode) => {
            Some(CONTEXT_MODE_ARGS)
        }
        (AgentId::Cursor, ToolId::Codegraph) => {
            Some(&["runmcp", "--agent", "cursor", "codegraph", "serve", "--mcp"])
        }
        (AgentId::Cursor, ToolId::ContextMode) => {
            Some(&["runmcp", "--agent", "cursor", "context-mode"])
        }
        (AgentId::Warp, ToolId::Codegraph) => {
            Some(&["runmcp", "--agent", "warp", "codegraph", "serve", "--mcp"])
        }
        (AgentId::Warp, ToolId::ContextMode) => {
            Some(&["runmcp", "--agent", "warp", "context-mode"])
        }
        (AgentId::Droid, ToolId::Codegraph) => {
            Some(&["runmcp", "--agent", "droid", "codegraph", "serve", "--mcp"])
        }
        (AgentId::Droid, ToolId::ContextMode) => {
            Some(&["runmcp", "--agent", "droid", "context-mode"])
        }
        (AgentId::Devin, ToolId::Codegraph) => {
            Some(&["runmcp", "--agent", "devin", "codegraph", "serve", "--mcp"])
        }
        (AgentId::Devin, ToolId::ContextMode) => {
            Some(&["runmcp", "--agent", "devin", "context-mode"])
        }
        (AgentId::Antigravity, ToolId::Codegraph) => Some(&[
            "runmcp",
            "--agent",
            "antigravity",
            "codegraph",
            "serve",
            "--mcp",
        ]),
        (AgentId::Antigravity, ToolId::ContextMode) => {
            Some(&["runmcp", "--agent", "antigravity", "context-mode"])
        }
        (AgentId::Codex, ToolId::Codegraph) => {
            Some(&["runmcp", "--agent", "codex", "codegraph", "serve", "--mcp"])
        }
        (AgentId::Codex, ToolId::ContextMode) => {
            Some(&["runmcp", "--agent", "codex", "context-mode"])
        }
        _ => None,
    }
}

pub fn command_is_stale(command: &str) -> bool {
    command.replace('\\', "/").contains("/$bunfs/")
}

pub fn command_is_current_toksave(command: &str) -> bool {
    !command_is_stale(command) && command == toksave_abs()
}

/// Official `rtk init --auto-patch` writes `rtk hook <agent>` next to toksave's
/// `rtk-hook <agent>` wrapper. Both fire and commands become `rtk rtk …`.
pub fn command_is_native_rtk_hook(command: &str, agent: &str) -> bool {
    let native = format!("rtk hook {agent}");
    command.contains(&native)
}

pub fn json_entry_healthy(entry: &Value, expected_args: &[&str]) -> bool {
    if let Some(arr) = entry.get("command").and_then(|c| c.as_array()) {
        let parts: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
        if parts.is_empty() || !command_is_current_toksave(parts[0]) {
            return false;
        }
        return parts.get(1..).is_some_and(|tail| tail == expected_args);
    }
    let Some(cmd) = entry.get("command").and_then(|c| c.as_str()) else {
        return false;
    };
    if !command_is_current_toksave(cmd) {
        return false;
    }
    let args: Vec<&str> = entry
        .get("args")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    args.as_slice() == expected_args
}

pub fn json_tool_healthy(cfg: &Value, servers_key: &str, agent: AgentId, tool: ToolId) -> bool {
    let Some(name) = mcp_server_name(tool) else {
        return false;
    };
    let Some(expected) = expected_runmcp_args(agent, tool) else {
        return false;
    };
    cfg.get(servers_key)
        .and_then(|m| m.get(name))
        .is_some_and(|entry| json_entry_healthy(entry, expected))
}

pub fn toml_tool_healthy(doc: &DocumentMut, agent: AgentId, tool: ToolId) -> bool {
    let Some(name) = mcp_server_name(tool) else {
        return false;
    };
    let Some(expected) = expected_runmcp_args(agent, tool) else {
        return false;
    };
    let Some(servers) = doc.get("mcp_servers").and_then(|s| s.as_table_like()) else {
        return false;
    };
    let Some(server) = servers.get(name).and_then(|s| s.as_table_like()) else {
        return false;
    };
    let Some(cmd) = server.get("command").and_then(|c| c.as_str()) else {
        return false;
    };
    if !command_is_current_toksave(cmd) {
        return false;
    }
    let args: Vec<&str> = server
        .get("args")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    args.as_slice() == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bunfs_command_is_unhealthy() {
        let entry = json!({
            "command": "/$bunfs/root/toksave",
            "args": ["runmcp", "codegraph", "serve", "--mcp"]
        });
        assert!(!json_entry_healthy(&entry, CODEGRAPH_ARGS));
    }

    #[test]
    fn current_toksave_with_matching_args_is_healthy() {
        let entry = json!({
            "command": toksave_abs(),
            "args": ["runmcp", "codegraph", "serve", "--mcp"]
        });
        assert!(json_entry_healthy(&entry, CODEGRAPH_ARGS));
    }

    #[test]
    fn wrong_args_are_unhealthy() {
        let entry = json!({
            "command": toksave_abs(),
            "args": ["npx", "codegraph"]
        });
        assert!(!json_entry_healthy(&entry, CODEGRAPH_ARGS));
    }

    #[test]
    fn bunfs_is_stale_even_with_backslashes() {
        assert!(command_is_stale(r"C:\$bunfs\root\toksave"));
        assert!(command_is_stale("/$bunfs/root/toksave"));
        assert!(!command_is_stale(r"C:\Users\me\toksave.exe"));
        assert!(!command_is_stale("C:/Users/me/toksave.exe"));
    }

    #[test]
    fn native_rtk_hook_does_not_match_toksave_wrapper() {
        assert!(command_is_native_rtk_hook("rtk hook claude", "claude"));
        assert!(!command_is_native_rtk_hook(
            &format!("{} rtk-hook claude", toksave_abs()),
            "claude"
        ));
    }
}
