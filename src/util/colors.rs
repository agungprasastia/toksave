use colored::Colorize;

pub const CHECK: &str = "✔ ";
pub const CROSS: &str = "✖ ";
pub const WARN: &str = "⚠ ";
pub const BULLET: &str = "• ";

pub fn ok(msg: &str) {
    println!("  {} {}", CHECK.green(), msg);
}

pub fn err(msg: &str) {
    eprintln!("  {} {}", CROSS.red(), msg);
}

pub fn warn(msg: &str) {
    println!("  {} {}", WARN.yellow(), msg);
}

pub fn info(msg: &str) {
    println!("  {} {}", "ℹ ".cyan(), msg);
}

pub fn banner(title: &str, subtitle: &str) {
    println!();
    println!(
        "  {}{}",
        title.bold().cyan(),
        format!("  {subtitle}").dimmed()
    );
    println!();
}

pub fn verbose(msg: &str, is_verbose: bool) {
    if is_verbose {
        println!("  [v] {}", msg.dimmed());
    }
}
