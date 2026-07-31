// use toksave_rs::cli::{parse_cli, Command};
use toksave_rs::util::version::toksave_version;

fn main() {
    println!("toksave-rs {}", toksave_version());
    // let _ = parse_cli(std::env::args().collect());
}
