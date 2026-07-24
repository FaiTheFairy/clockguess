use clap::{Parser, ValueEnum};

use crate::difficulty::Difficulty;

// Clap's ranged parser accepts i64 bounds even when parsing u16 values.
const MIN_WIDTH: i64 = 20;
const MIN_HEIGHT: i64 = 10;

// Source - https://stackoverflow.com/a/79614957
// Posted by roylaurie
// Retrieved 2026-07-24, License - CC BY-SA 4.0

use clap::builder::styling::{AnsiColor, Effects, Style, Styles};

//const NOP: Style = Style::new();
const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
// const WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
// const NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
// const GOOD: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

/// Cargo's color style
/// [source](https://github.com/crate-ci/clap-cargo/blob/master/src/style.rs)
const CARGO_STYLING: Styles = Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .valid(VALID)
    .invalid(INVALID);

#[derive(Parser, Debug)]
#[command(version, about = "Practice reading an analog clock")]
#[clap(styles = CARGO_STYLING)]
pub struct Cli {
    /// Gameplay mode.
    #[arg(short, long, value_enum, default_value_t)]
    pub mode: GameMode,

    /// Number of rounds in challenge mode.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..))]
    pub rounds: u32,

    /// Time limit in seconds for rapid-fire mode.
    #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u64).range(1..))]
    pub rapid_seconds: u64,

    /// Accuracy required for a correct answer.
    #[arg(short, long, value_enum, default_value_t)]
    pub difficulty: Difficulty,

    /// Visual theme used by the clock.
    #[arg(short, long, value_enum, default_value_t)]
    pub theme: ThemeChoice,

    /// Override the detected terminal width
    #[arg(long, value_parser = clap::value_parser!(u16).range(MIN_WIDTH..))]
    pub width: Option<u16>,

    /// Override the detected terminal height
    #[arg(long, value_parser = clap::value_parser!(u16).range(MIN_HEIGHT..))]
    pub height: Option<u16>,

    /// Do not clear the terminal between rounds.
    #[arg(long)]
    pub no_clear: bool,

    /// Whether or not to show seconds hand.
    #[arg(long, value_enum, default_value_t)]
    pub show_seconds: SecondHandMode,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum GameMode {
    #[default]
    Practice,
    Challenge,
    RapidFire,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ThemeChoice {
    #[default]
    Classic,
    Monochrome,
    Unicode,
}

#[derive(Copy, Clone, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum SecondHandMode {
    #[default]
    Auto,
    Show,
    Hide,
}

impl SecondHandMode {
    pub const fn resolve(self, difficulty: Difficulty) -> bool {
        match self {
            Self::Auto => difficulty.show_seconds_by_default(),
            Self::Show => true,
            Self::Hide => false,
        }
    }
}
