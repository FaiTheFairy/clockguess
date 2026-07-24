use std::{f64::consts::TAU, str::FromStr};

use anyhow::{Context, ensure};
use rand::RngExt;

pub const HOURS_PER_CLOCK_CYCLE: u32 = 12;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const SECONDS_PER_MINUTE: u32 = 60;

pub const SECONDS_PER_HOUR: u32 = MINUTES_PER_HOUR * SECONDS_PER_MINUTE;
pub const SECONDS_PER_CLOCK_CYCLE: u32 = HOURS_PER_CLOCK_CYCLE * SECONDS_PER_HOUR;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
    second: u8,
}

impl ClockTime {
    #[cfg(test)]
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self::try_new(hour, minute, second)
            .expect("clock components must be within their valid ranges")
    }

    pub fn try_new(hour: u8, minute: u8, second: u8) -> anyhow::Result<Self> {
        ensure!(hour < 24, "hour must be between 0 and 23");
        ensure!(minute < 60, "minute must be between 0 and 59");
        ensure!(second < 60, "second must be between 0 and 59");

        Ok(Self {
            hour,
            minute,
            second,
        })
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();

        Self {
            hour: rng.random_range(0..24),
            minute: rng.random_range(0..60),
            second: rng.random_range(0..60),
        }
    }

    pub const fn hour_12(self) -> u8 {
        match self.hour % 12 {
            0 => 12,
            hour => hour,
        }
    }

    pub fn total_seconds(self) -> u32 {
        u32::from(self.hour) * SECONDS_PER_HOUR
            + u32::from(self.minute) * SECONDS_PER_MINUTE
            + u32::from(self.second)
    }

    /// Returns the time's position within a 12-hour analog-clock cycle
    pub fn analog_seconds(self) -> u32 {
        self.total_seconds() % SECONDS_PER_CLOCK_CYCLE
    }

    /// Finds the shortest distance, in seconds, between two times on a 12-hour clock.
    ///
    /// For example, 11:59 and 12:01 are two minutes apart rather than
    /// eleven hours and fifty-eight minutes apart.
    pub fn analog_difference(self, other: Self) -> u32 {
        let direct_difference = self.analog_seconds().abs_diff(other.analog_seconds());

        direct_difference.min(SECONDS_PER_CLOCK_CYCLE - direct_difference)
    }

    pub fn hour_angle(self) -> f64 {
        // `self.hour` gives value between 0 and 23, `hour_angle` assumes base-12 hour
        let hours = f64::from(self.hour % 12);
        let minutes = f64::from(self.minute);
        let seconds = f64::from(self.second);

        (hours + minutes / 60.0 + seconds / 3600.0) / 12.0 * TAU
    }

    pub fn minute_angle(self) -> f64 {
        let minutes = f64::from(self.minute);
        let seconds = f64::from(self.second);

        (minutes + seconds / 60.0) / 60.0 * TAU
    }

    pub fn second_angle(self) -> f64 {
        f64::from(self.second) / 60.0 * TAU
    }

    pub const fn display_analog(self) -> ClockTimeDisplayAnalog {
        ClockTimeDisplayAnalog(Self {
            hour: self.hour_12(),
            minute: self.minute,
            second: self.second,
        })
    }

    fn parse_compact(input: &str) -> anyhow::Result<Self> {
        fn parse_component(value: &str, name: &str) -> anyhow::Result<u8> {
            value
                .parse()
                .with_context(|| format!("{name} must be a number"))
        }

        let (hour, minute, second) = match input.len() {
            1 | 2 => {
                let hour = parse_component(input, "hour")?;
                (hour, 0, 0)
            }

            3 => {
                let hour = parse_component(&input[..1], "hour")?;
                let minute = parse_component(&input[1..], "minute")?;
                (hour, minute, 0)
            }

            4 => {
                let hour = parse_component(&input[..2], "hour")?;
                let minute = parse_component(&input[2..], "minute")?;
                (hour, minute, 0)
            }

            5 => {
                let hour = parse_component(&input[..1], "hour")?;
                let minute = parse_component(&input[1..3], "minute")?;
                let second = parse_component(&input[3..], "second")?;
                (hour, minute, second)
            }

            6 => {
                let hour = parse_component(&input[..2], "hour")?;
                let minute = parse_component(&input[2..4], "minute")?;
                let second = parse_component(&input[4..], "second")?;
                (hour, minute, second)
            }

            _ => anyhow::bail!("compact time must use H, HH, HMM, HHMM, HMMSS, or HHMMSS"),
        };

        Self::try_new(hour, minute, second)
    }

    fn parse_separated(input: &str) -> anyhow::Result<Self> {
        let mut parts = input
            .trim()
            .split(&[':', ' ', '.'])
            .filter(|part| !part.is_empty());

        let hour = parts
            .next()
            .expect("non-empty input always has a first component")
            .parse()
            .context("hour must be a number")?;

        let minute = parts
            .next()
            .unwrap_or("0")
            .parse()
            .context("minute must be a number")?;

        let second = parts
            .next()
            .unwrap_or("0")
            .parse()
            .context("second must be a number")?;

        ensure!(
            parts.next().is_none(),
            "use HH, HH:MM, HH:MM:SS, or compact military time"
        );

        Self::try_new(hour, minute, second)
    }
}

impl FromStr for ClockTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        ensure!(!s.is_empty(), "time cannot be empty");

        if s.chars().all(|ch| ch.is_ascii_digit()) {
            return Self::parse_compact(s);
        }

        Self::parse_separated(s)
    }
}

impl std::fmt::Display for ClockTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

pub struct ClockTimeDisplayAnalog(ClockTime);

impl std::fmt::Display for ClockTimeDisplayAnalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.0.hour_12(),
            self.0.minute,
            self.0.second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midnight_displays_as_twelve() {
        assert_eq!(ClockTime::new(0, 15, 0).hour_12(), 12);
    }

    #[test]
    fn noon_displays_as_twelve() {
        assert_eq!(ClockTime::new(12, 15, 0).hour_12(), 12);
    }

    #[test]
    fn analog_difference_is_symmetric() {
        let a = ClockTime::new(11, 59, 0);
        let b = ClockTime::new(12, 1, 0);

        assert_eq!(a.analog_difference(b), b.analog_difference(a));
    }

    #[test]
    fn analog_difference_uses_shortest_path() {
        let a = ClockTime::new(1, 0, 0);
        let b = ClockTime::new(11, 0, 0);

        assert_eq!(a.analog_difference(b), 2 * 60 * 60);
    }

    #[test]
    fn parses_hour_only() {
        assert_eq!("7".parse::<ClockTime>().unwrap(), ClockTime::new(7, 0, 0));
    }

    #[test]
    fn parses_hour_and_minute() {
        assert_eq!(
            "7:30".parse::<ClockTime>().unwrap(),
            ClockTime::new(7, 30, 0)
        );
    }

    #[test]
    fn parses_full_time() {
        assert_eq!(
            "7:30:15".parse::<ClockTime>().unwrap(),
            ClockTime::new(7, 30, 15)
        );
    }

    #[test]
    fn rejects_extra_components() {
        assert!("7:30:15:10".parse::<ClockTime>().is_err());
    }

    #[test]
    fn rejects_invalid_values() {
        assert!("24:00".parse::<ClockTime>().is_err());
        assert!("12:60".parse::<ClockTime>().is_err());
        assert!("12:30:60".parse::<ClockTime>().is_err());
    }

    #[test]
    fn computes_difference_across_twelve_oclock() {
        let before = ClockTime::new(11, 59, 0);
        let after = ClockTime::new(12, 1, 0);

        assert_eq!(before.analog_difference(after), 120);
    }

    #[test]
    fn ignores_am_pm_for_analog_comparison() {
        let morning = ClockTime::new(3, 15, 0);
        let afternoon = ClockTime::new(15, 15, 0);

        assert_eq!(morning.analog_difference(afternoon), 0);
    }

    #[test]
    fn parses_compact_times() {
        assert_eq!(
            "920".parse::<ClockTime>().unwrap(),
            ClockTime::new(9, 20, 0)
        );
        assert_eq!(
            "1320".parse::<ClockTime>().unwrap(),
            ClockTime::new(13, 20, 0)
        );
        assert_eq!(
            "132045".parse::<ClockTime>().unwrap(),
            ClockTime::new(13, 20, 45)
        );
    }

    #[test]
    fn parses_multiple_separators() {
        assert_eq!(
            "9:20".parse::<ClockTime>().unwrap(),
            ClockTime::new(9, 20, 0)
        );
        assert_eq!(
            "9 20".parse::<ClockTime>().unwrap(),
            ClockTime::new(9, 20, 0)
        );
        assert_eq!(
            "9.20".parse::<ClockTime>().unwrap(),
            ClockTime::new(9, 20, 0)
        );
    }
}
