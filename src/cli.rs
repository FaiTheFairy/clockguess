use clap::{Parser, ValueEnum};

use crate::Difficulty;

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
    #[arg(short, long)]
    pub rounds: Option<u32>,

    /// Override the detected terminal width
    #[arg(long)]
    pub width: Option<u16>,

    /// Override the detected terminal height
    #[arg(long)]
    pub height: Option<u16>,

    /// Do not clear the terminal between rounds.
    #[arg(long)]
    pub no_clear: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Theme {
    Classic,
    Monochrome,
    Unicode,
}
