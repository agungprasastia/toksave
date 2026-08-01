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

const LABEL_WIDTH: usize = 30;

/// Task progress, mirroring the old TS `Progress`: on a TTY renders an
/// animated bar + label, on non-TTY just prints the label / final message.
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

    pub fn start(&mut self, label: &str) {
        if !self.tty {
            println!("  {label}");
            return;
        }
        let bar = ProgressBar::new(100);
        bar.set_style(
            ProgressStyle::with_template("{spinner} {msg}{spaces}[{bar:20}] {percent}%")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        bar.enable_steady_tick(Duration::from_millis(80));
        bar.set_message(truncate(label));
        bar.set_position(0);
        self.bar = Some(bar);
    }

    /// Clone of the underlying bar, for wiring real download progress
    /// (init.rs sets it as the download sink so the bar reflects bytes).
    pub fn bar(&self) -> Option<ProgressBar> {
        self.bar.clone()
    }

    pub fn stop(&mut self, message: &str) {
        if let Some(bar) = self.bar.take() {
            bar.set_message(truncate(&strip_ansi(message)));
            bar.finish_and_clear();
            println!("  {}", strip_ansi(message));
        } else {
            println!("  {message}");
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate(s: &str) -> String {
    let plain = strip_ansi(s);
    if plain.len() <= LABEL_WIDTH {
        plain.to_string()
    } else {
        plain.chars().take(LABEL_WIDTH - 3).collect::<String>() + "..."
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

    let max_label_len = options
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
        println!("{} {}", "●".magenta().bold(), title.magenta().bold());

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
                "○ ".to_string()
            };

            let padded_label = format!("{:<width$}", opt.label, width = max_label_len);
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

            println!("{prefix}{icon}{display_label}    {tag}  {hint}");
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
