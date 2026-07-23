use std::{f64::consts::TAU, str::FromStr};

use anyhow::{Context, ensure};
use rand::RngExt;

const HOURS_PER_DAY: u32 = 12;
const HOURS_PER_CLOCK_CYCLE: u32 = 12;
const MINUTES_PER_HOUR: u32 = 60;
const SECONDS_PER_MINUTE: u32 = 60;

const SECONDS_PER_HOUR: u32 = MINUTES_PER_HOUR * SECONDS_PER_MINUTE;
const SECONDS_PER_CLOCK_CYCLE: u32 = HOURS_PER_CLOCK_CYCLE * SECONDS_PER_HOUR;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
    second: u8,
}

impl ClockTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        assert!(hour < 24, "hour must be between 0 and 23");
        assert!(minute < 60, "minute must be between 0 and 59");
        assert!(second < 60, "second must be between 0 and 59");

        Self {
            hour,
            minute,
            second,
        }
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

    pub fn hour_12(self) -> u8 {
        match self.hour % 12 {
            0 => 12,
            hour => hour,
        }
    }

    pub fn hour_24(self) -> u8 {
        self.hour
    }

    pub fn minute(self) -> u8 {
        self.minute
    }

    pub fn second(self) -> u8 {
        self.second
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
    fn analog_difference(self, other: Self) -> u32 {
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
}

impl FromStr for ClockTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        ensure!(!s.is_empty(), "time cannot be empty");

        let mut parts = s.trim().split(':');

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

        ensure!(parts.next().is_none(), "use HH, HH:MM, or HH:MM:SS format");

        Self::try_new(hour, minute, second)
    }
}

impl std::fmt::Display for ClockTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
