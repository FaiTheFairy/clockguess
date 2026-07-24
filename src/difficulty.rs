use crate::clock::ClockTime;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Difficulty {
    Hour,
    FiveMinutes,
    Minute,
    ThirtySeconds,
    Exact,
}

impl Difficulty {
    /// The interval to which the player is expected to read the clock
    pub fn precision_seconds(self) -> u32 {
        match self {
            Difficulty::Hour => 60 * 60,
            Difficulty::FiveMinutes => 5 * 60,
            Difficulty::Minute => 60,
            Difficulty::ThirtySeconds => 30,
            Difficulty::Exact => 0,
        }
    }

    /// An answer rounded to an interval may be at most half an interval away
    pub fn tolerance_seconds(self) -> u32 {
        self.precision_seconds() / 2
    }

    pub fn description(self) -> &'static str {
        match self {
            Difficulty::Hour => "nearest hour",
            Difficulty::FiveMinutes => "nearest five minutes",
            Difficulty::Minute => "nearest minute",
            Difficulty::ThirtySeconds => "nearest thirty seconds",
            Difficulty::Exact => "exact time",
        }
    }

    pub fn accepts(self, expected: ClockTime, answer: ClockTime) -> bool {
        expected.analog_difference(answer) <= self.tolerance_seconds()
    }

    pub fn show_seconds_by_default(self) -> bool {
        match self {
            Difficulty::ThirtySeconds => true,
            Difficulty::Exact => true,
            Difficulty::Minute => false,
            Difficulty::Hour => false,
            Difficulty::FiveMinutes => false,
        }
    }
}
