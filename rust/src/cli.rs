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
