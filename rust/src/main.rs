use toksave_rs::cli::{parse_cli, CommandType};

fn main() {
    let parsed = parse_cli(std::env::args().collect());
    let code = match &parsed.command {
        CommandType::Init => run_init(parsed),
        CommandType::Doctor => run_doctor(parsed.clone()),
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

fn run_doctor(parsed: toksave_rs::cli::ParsedCli) -> i32 {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(toksave_rs::commands::doctor::run_doctor(
        &parsed,
        parsed.offline,
        parsed.fix,
    ))
}
