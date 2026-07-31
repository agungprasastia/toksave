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
    AgentInfo {
        id: AgentId::Claude,
        label: "Claude Code",
        homepage: "https://github.com/anthropics/claude-code",
        cli_bin: "claude",
    },
    AgentInfo {
        id: AgentId::Opencode,
        label: "OpenCode",
        homepage: "https://github.com/anomalyco/opencode",
        cli_bin: "opencode",
    },
    AgentInfo {
        id: AgentId::Codex,
        label: "Codex",
        homepage: "https://github.com/openai/codex",
        cli_bin: "codex",
    },
    AgentInfo {
        id: AgentId::Antigravity,
        label: "Antigravity",
        homepage: "https://antigravity.google",
        cli_bin: "agy",
    },
    AgentInfo {
        id: AgentId::Copilot,
        label: "GitHub Copilot",
        homepage: "https://github.com/github/copilot-cli",
        cli_bin: "copilot",
    },
    AgentInfo {
        id: AgentId::Droid,
        label: "Factory Droid",
        homepage: "https://factory.ai",
        cli_bin: "droid",
    },
    AgentInfo {
        id: AgentId::Devin,
        label: "Devin / Cascade",
        homepage: "https://devin.ai",
        cli_bin: "devin",
    },
    AgentInfo {
        id: AgentId::Warp,
        label: "Warp / Oz",
        homepage: "https://warp.dev",
        cli_bin: "warp",
    },
];

pub const ALL_TOOLS: &[ToolInfo] = &[
    ToolInfo {
        id: ToolId::Rtk,
        label: "RTK",
        homepage: "https://github.com/rtk-ai/rtk",
        channel: Channel::Github,
        min_node_major: 0,
        not_trackable: false,
        instruction_only: false,
    },
    ToolInfo {
        id: ToolId::Caveman,
        label: "Caveman",
        homepage: "https://github.com/JuliusBrussee/caveman",
        channel: Channel::Skill,
        min_node_major: 0,
        not_trackable: false,
        instruction_only: false,
    },
    ToolInfo {
        id: ToolId::Codegraph,
        label: "CodeGraph",
        homepage: "https://github.com/colbymchenry/codegraph",
        channel: Channel::Npm,
        min_node_major: 18,
        not_trackable: false,
        instruction_only: false,
    },
    ToolInfo {
        id: ToolId::ContextMode,
        label: "Context-Mode",
        homepage: "https://github.com/mksglu/context-mode",
        channel: Channel::Npm,
        min_node_major: 22,
        not_trackable: false,
        instruction_only: false,
    },
    ToolInfo {
        id: ToolId::Ponytail,
        label: "Ponytail",
        homepage: "https://github.com/DietrichGebert/ponytail",
        channel: Channel::Npm,
        min_node_major: 0,
        not_trackable: false,
        instruction_only: false,
    },
    ToolInfo {
        id: ToolId::Principles,
        label: "Principles",
        homepage: "https://github.com/multica-ai/andrej-karpathy-skills",
        channel: Channel::Skill,
        min_node_major: 0,
        not_trackable: true,
        instruction_only: true,
    },
];

pub fn agent_info(id: AgentId) -> &'static AgentInfo {
    ALL_AGENTS
        .iter()
        .find(|a| a.id == id)
        .expect("valid agent id")
}

pub fn tool_info(id: ToolId) -> &'static ToolInfo {
    ALL_TOOLS
        .iter()
        .find(|t| t.id == id)
        .expect("valid tool id")
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
        "principles" | "karpathy-skills" | "karpathy" | "karpathyskills" => {
            Some(ToolId::Principles)
        }
        _ => None,
    }
}
