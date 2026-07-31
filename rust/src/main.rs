use toksave_rs::cli::{parse_cli, CommandType};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = early_hook_dispatch(&args) {
        std::process::exit(code);
    }
    let parsed = parse_cli(args);
    let code = match &parsed.command {
        CommandType::Init => run_init(parsed),
        CommandType::Doctor => run_doctor(parsed.clone()),
        CommandType::Update => run_update(parsed.clone()),
        CommandType::Uninstall => run_uninstall(parsed.clone()),
        CommandType::Disable => run_disable(parsed.clone()),
        CommandType::CodexPermHook => toksave_rs::commands::hooks::codex_perm::run(),
        CommandType::RtkHook => {
            toksave_rs::commands::hooks::rtk::run(parsed.hook_args.first().map(String::as_str))
        }
        CommandType::ContextModeHook => {
            toksave_rs::commands::hooks::context_mode::run(&parsed.hook_args)
        }
        CommandType::AgyHook | CommandType::CopilotHook => {
            // rtk-hook aliases: `agy-hook <variant>` routes to the RTK prefixer.
            if parsed.hook_args.first().is_some_and(|a| {
                [
                    "rtk", "agy", "claude", "codex", "copilot", "droid", "devin", "warp",
                ]
                .contains(&a.as_str())
            }) {
                toksave_rs::commands::hooks::rtk::run(parsed.hook_args.first().map(String::as_str))
            } else {
                toksave_rs::commands::hooks::agy::run_codegraph_index_hook(
                    &toksave_rs::commands::hooks::read_stdin(),
                )
            }
        }
        other => {
            println!("toksave-rs: `{other:?}` not implemented in Rust yet (TS build handles it).");
            0
        }
    };
    std::process::exit(code);
}

/// Mirror TS index.ts early dispatch, before clap arg parsing.
/// Returns Some(exit_code) when handled.
fn early_hook_dispatch(args: &[String]) -> Option<i32> {
    let a0 = args.get(1).map(String::as_str)?;
    let a1 = args.get(2).map(String::as_str).unwrap_or("");
    if (a0 == "agy-hook" || a0 == "copilot-hook") && a1 == "codegraph-index" {
        return Some(toksave_rs::commands::hooks::agy::run_codegraph_index_hook(
            &toksave_rs::commands::hooks::read_stdin(),
        ));
    }
    if a0 == "rtk-hook"
        && [
            "agy", "codex", "claude", "copilot", "droid", "devin", "warp",
        ]
        .contains(&a1)
    {
        return Some(toksave_rs::commands::hooks::rtk::run(Some(a1)));
    }
    None
}

fn run_init(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::init::run_init(&parsed))
}

fn run_doctor(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::doctor::run_doctor(
        &parsed,
        parsed.offline,
        parsed.fix,
    ))
}

fn run_update(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::update::run_update(&parsed))
}

fn run_uninstall(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::uninstall::run_uninstall(&parsed))
}

fn run_disable(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::disable::run_disable(&parsed))
}
