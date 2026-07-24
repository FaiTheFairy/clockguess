use clap::{Parser, ValueEnum};

use crate::difficulty::Difficulty;

// Clap's ranged parser accepts i64 bounds even when parsing u16 values.
const MIN_WIDTH: i64 = 20;
const MIN_HEIGHT: i64 = 10;

#[derive(Parser, Debug)]
#[command(version, about = "Practice reading an analog clock")]
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

    /// Whether or not to show seconds hand. Auto mode uses difficulty to decide.
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
