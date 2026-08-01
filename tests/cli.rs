use toksave::cli::{parse_cli, CommandType};
use toksave::registry::{AgentId, ToolId};

fn parse(args: &[&str]) -> toksave::cli::ParsedCli {
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
