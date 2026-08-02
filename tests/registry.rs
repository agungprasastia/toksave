use toksave::registry::{
    ALL_AGENTS, ALL_TOOLS, AgentId, ToolId, agent_info, parse_agent_id, parse_tool_id, tool_info,
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
