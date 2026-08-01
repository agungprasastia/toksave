use crate::registry::AgentId;
use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveToPreviousLine, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{stdout, IsTerminal};
use std::time::Duration;

const BAR_WIDTH: &str = "20";
const BAR_COL: usize = 40; // bar/percent column start (fixed alignment)

/// Task progress, mirroring the old tokless UI: on a TTY each task renders
/// check + label, and the bar lives at a fixed column; `stop` finalizes the
/// line with a full green bar + 100%. Sections get a tree header.
pub struct Progress {
    tty: bool,
    bar: Option<ProgressBar>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            tty: std::io::stdout().is_terminal(),
            bar: None,
        }
    }

    /// Section header + tree branch (e.g. `Tools` / `Agents` like tokless).
    pub fn start_section(&mut self, name: &str) {
        println!("{}", name.bold());
        println!("{}", "│".dimmed());
    }

    pub fn start(&mut self, label: &str) {
        if !self.tty {
            println!("  {label}");
            return;
        }
        let bar = ProgressBar::new(100);
        let msg_col = BAR_COL;
        let tpl = format!("  {{msg:<{msg_col}}}[{{bar:{BAR_WIDTH}}}] {{percent:>3}}%{{msg:>0}}");
        let _ = tpl; // template assembled below with style colors
        bar.set_style(
            ProgressStyle::with_template("  {msg}  [{bar:20}] {percent:>3}%")
                .unwrap()
                .progress_chars("█░"),
        );
        bar.set_message(format!("  {label}"));
        bar.enable_steady_tick(Duration::from_millis(80));
        bar.set_position(0);
        self.bar = Some(bar);
    }

    /// Clone of the underlying bar, for wiring real download progress
    /// (init.rs sets it as the download sink so the bar reflects bytes).
    pub fn bar(&self) -> Option<ProgressBar> {
        self.bar.clone()
    }

    /// Terminal state: on TTY renders the final green bar line at a fixed
    /// column ("  ✔ Label [████...] 100% tail"); non-TTY prints the message.
    pub fn stop(&mut self, message: &str) {
        let clean = strip_ansi(message);
        let clean = clean.trim_start_matches('✔').trim_start().to_string();
        if let Some(bar) = self.bar.take() {
            bar.finish_and_clear();
            // Split "Label tail..." → label + tail (version / note).
            let (label, tail) = match clean.split_once(' ') {
                Some((l, t)) => (l.to_string(), t.to_string()),
                None => (clean.clone(), String::new()),
            };
            let padded = format!("  {} {:<BAR_COL$}", "✔".green(), label.bold());
            let green_bar = "█".repeat(20).green();
            let tail_dim = if tail.is_empty() {
                String::new()
            } else {
                format!(" {}", tail.dimmed())
            };
            let warn = clean.contains(" not installed") || clean.contains("skipped");
            if warn {
                println!("  {} {}", "⚠".yellow(), clean.dimmed());
            } else {
                println!("{padded}[{green_bar}] {}{}", "100%".green(), tail_dim);
            }
        } else {
            println!("  {clean}");
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// boxen-style green summary box.
pub fn green_box(title: &str) {
    let plain = strip_ansi(title);
    let width = plain.len() + 4;
    println!();
    println!("  ┌{}┐", "─".repeat(width).green());
    println!("  │  {}  │", title.green().bold());
    println!("  └{}┘", "─".repeat(width).green());
    println!();
}

pub struct SelectOption {
    pub value: AgentId,
    pub label: String,
    pub disabled: bool,
    pub hint: String,
    pub selected: bool,
}

pub fn multi_select(title: &str, mut options: Vec<SelectOption>) -> Vec<AgentId> {
    if !stdout().is_terminal() {
        return options
            .into_iter()
            .filter(|o| o.selected && !o.disabled)
            .map(|o| o.value)
            .collect();
    }

    let mut cursor = options.iter().position(|o| !o.disabled).unwrap_or(0);

    let _max_label_len = options
        .iter()
        .map(|o| o.label.len())
        .max()
        .unwrap_or(16)
        .max(16);

    let _ = enable_raw_mode();
    let mut out = stdout();
    let _ = out.execute(Hide);

    let num_lines = options.len() + 3; // Title + footer rule + controls

    let mut first_render = true;

    // Drain any leftover keys (e.g. Enter keypress from spawning command)
    while event::poll(Duration::from_millis(50)).unwrap_or(false) {
        let _ = event::read();
    }

    loop {
        if !first_render {
            let _ = out.execute(MoveToPreviousLine(num_lines as u16));
        }
        first_render = false;

        // Title line matching original TS tokless/toksave UI
        println!("{} {}\x1b[K", "●".magenta().bold(), title.magenta().bold());

        // Options
        for (i, opt) in options.iter().enumerate() {
            let is_hovered = i == cursor;
            let prefix = if is_hovered {
                "> ".cyan().bold().to_string()
            } else {
                "  ".to_string()
            };

            let icon = if opt.disabled {
                "· ".dimmed().to_string()
            } else if opt.selected {
                "◉ ".green().bold().to_string()
            } else {
                "○ ".dimmed().to_string()
            };

            let padded_label = format!("{:<width$}", opt.label, width = 16);

            let display_label = if opt.disabled {
                padded_label.dimmed().to_string()
            } else {
                padded_label.bold().to_string()
            };

            let tag = if opt.disabled {
                format!("{:<11}", "[MISSING]".yellow())
            } else {
                format!("{:<11}", "[READY]".green())
            };

            let hint = opt.hint.dimmed().to_string();

            println!("{prefix}{icon}{display_label}    {tag}  {hint}\x1b[K");
        }

        // Footer rule
        println!(
            "{}",
            "──────────────────────────────────────────────────────────".dimmed()
        );

        // Controls
        println!(
            "{} {}  ·  {} {}  ·  {} {}  ·  {} {}",
            "↑/↓".yellow(),
            "move".dimmed(),
            "<space>".yellow(),
            "select".dimmed(),
            "<a>".yellow(),
            "all".dimmed(),
            "<enter>".yellow(),
            "confirm".dimmed()
        );

        if let Ok(Event::Key(key)) = event::read() {
            if key.kind != event::KeyEventKind::Press {
                continue;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                let _ = out.execute(Show);
                let _ = disable_raw_mode();
                std::process::exit(1);
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if cursor > 0 {
                        let mut prev = cursor - 1;
                        while prev > 0 && options[prev].disabled {
                            prev -= 1;
                        }
                        if !options[prev].disabled {
                            cursor = prev;
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if cursor + 1 < options.len() {
                        let mut next = cursor + 1;
                        while next + 1 < options.len() && options[next].disabled {
                            next += 1;
                        }
                        if !options[next].disabled {
                            cursor = next;
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if !options[cursor].disabled {
                        options[cursor].selected = !options[cursor].selected;
                    }
                }
                KeyCode::Char('a') => {
                    let all_selected = options.iter().filter(|o| !o.disabled).all(|o| o.selected);
                    for opt in options.iter_mut() {
                        if !opt.disabled {
                            opt.selected = !all_selected;
                        }
                    }
                }
                KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }
    }

    let _ = out.execute(Show);
    let _ = disable_raw_mode();

    options
        .into_iter()
        .filter(|o| o.selected && !o.disabled)
        .map(|o| o.value)
        .collect()
}
