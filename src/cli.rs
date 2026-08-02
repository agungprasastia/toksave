use crate::registry::{AgentId, RunOpts, ToolId, parse_agent_id, parse_tool_id};
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
    pub hook_args: Vec<String>,
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
            hook_args: vec![],
        }
    }
}

pub fn parse_cli(args: Vec<String>) -> ParsedCli {
    let mut cli = match Cli::try_parse_from(&args) {
        Ok(c) => c,
        Err(e) => {
            // clap prints help/version to stdout, errors to stderr, then exits with the right code
            e.exit();
        }
    };

    let mut parsed = ParsedCli::default();
    let opts = RunOpts {
        dry_run: cli.dry_run,
        upgrade: false,
        verbose: cli.verbose,
        yes: cli.yes,
        report: None,
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
        Some(Command::RtkHook { args }) => {
            parsed.command = CommandType::RtkHook;
            parsed.hook_args = args;
        }
        Some(Command::ContextModeHook { args }) => {
            parsed.command = CommandType::ContextModeHook;
            parsed.hook_args = args;
        }
        Some(Command::Runmcp { args }) => {
            parsed.command = CommandType::Runmcp;
            parsed.hook_args = args;
        }
        Some(Command::Index { auto }) => {
            parsed.command = CommandType::Index;
            parsed.auto = auto;
        }
        Some(Command::AgyHook { args }) => {
            parsed.command = CommandType::AgyHook;
            parsed.hook_args = args;
        }
        Some(Command::CopilotHook { args }) => {
            parsed.command = CommandType::CopilotHook;
            parsed.hook_args = args;
        }
    }

    parsed
}
