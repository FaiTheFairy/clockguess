use crate::clock::ClockTime;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Difficulty {
    Hour,
    #[default]
    TenMinutes,
    FiveMinutes,
    Minute,
    ThirtySeconds,
    Exact,
}

impl Difficulty {
    /// The interval to which the player is expected to read the clock
    pub const fn precision_seconds(self) -> u32 {
        match self {
            Self::Hour => 60 * 60,
            Self::TenMinutes => 10 * 60,
            Self::FiveMinutes => 5 * 60,
            Self::Minute => 60,
            Self::ThirtySeconds => 30,
            Self::Exact => 0,
        }
    }

    /// An answer rounded to an interval may be at most half an interval away
    pub const fn tolerance_seconds(self) -> u32 {
        self.precision_seconds() / 2
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Hour => "nearest hour",
            Self::TenMinutes => "nearest ten minutes",
            Self::FiveMinutes => "nearest five minutes",
            Self::Minute => "nearest minute",
            Self::ThirtySeconds => "nearest thirty seconds",
            Self::Exact => "exact time",
        }
    }

    pub fn accepts(self, expected: ClockTime, answer: ClockTime) -> bool {
        expected.analog_difference(answer) <= self.tolerance_seconds()
    }

    pub const fn show_seconds_by_default(self) -> bool {
        match self {
            Self::Exact | Self::ThirtySeconds => true,
            Self::Minute | Self::FiveMinutes | Self::TenMinutes | Self::Hour => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_minute_difficulty_accepts_half_interval_boundary() {
        let expected = ClockTime::new(10, 0, 0);
        let answer = ClockTime::new(10, 2, 30);

        assert!(Difficulty::FiveMinutes.accepts(expected, answer));
    }

    #[test]
    fn five_minute_difficulty_rejects_beyond_half_interval() {
        let expected = ClockTime::new(10, 0, 0);
        let answer = ClockTime::new(10, 2, 31);

        assert!(!Difficulty::FiveMinutes.accepts(expected, answer));
    }
}
