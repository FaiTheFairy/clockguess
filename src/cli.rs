use clap::{Parser, ValueEnum};

use crate::difficulty::Difficulty;

const MIN_WIDTH: i64 = 20;
const MIN_HEIGHT: i64 = 10;

#[derive(Parser, Debug)]
#[command(version, about = "Practice reading an analog clock")]
pub struct Cli {
    /// Accuracy required for a correct answer.
    #[arg(short, long, value_enum, default_value_t = Difficulty::FiveMinutes)]
    pub difficulty: Difficulty,

    /// Visual theme used by the clock.
    #[arg(short, long, value_enum, default_value_t = Theme::Classic)]
    pub theme: Theme,

    /// Number of rounds to play. Omit to play until quitting.
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..))]
    pub rounds: Option<u32>,

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Theme {
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
    pub fn resolve(self, difficulty: Difficulty) -> bool {
        match self {
            SecondHandMode::Auto => difficulty.show_seconds_by_default(),
            SecondHandMode::Show => true,
            SecondHandMode::Hide => false,
        }
    }
}
