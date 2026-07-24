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
    pub fn precision_seconds(self) -> u32 {
        match self {
            Difficulty::Hour => 60 * 60,
            Difficulty::TenMinutes => 10 * 60,
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
            Difficulty::TenMinutes => "nearest ten minutes",
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
            Difficulty::Exact => true,
            Difficulty::ThirtySeconds => true,
            Difficulty::Minute => false,
            Difficulty::FiveMinutes => false,
            Difficulty::TenMinutes => false,
            Difficulty::Hour => false,
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
