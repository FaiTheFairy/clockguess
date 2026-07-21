use std::{f64::consts::TAU, str::FromStr};

use anyhow::Context;
use rand::RngExt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
    second: u8,
}

impl ClockTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        assert!(hour < 24, "hour cannot be 24 or greater");
        assert!(minute < 60, "minute cannot be 60 or greater");
        assert!(second < 60, "second cannot be 60 or greater");
        Self {
            hour,
            minute,
            second,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        let hour = rng.random_range(0..24);
        let minute = rng.random_range(0..60);
        let second = rng.random_range(0..60);
        Self::new(hour, minute, second)
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
        self.hour_24() as u32 * 3600 + self.minute() as u32 * 60 + self.second() as u32
    }

    pub fn hour_angle(self) -> f64 {
        // `self.hour` gives value between 0 and 23, `hour_angle` assumes base-12 hour
        let hours = f64::from(self.hour % 12);
        let minutes = f64::from(self.minute);
        let seconds = f64::from(self.second);

        (hours + minutes / 60.0 + seconds / 3600.0) / 12.0 * TAU
    }

    pub fn minute_angle(self) -> f64 {
        ((self.minute as f64 + self.second as f64 / 60.0) / 60.0) * TAU
    }

    pub fn second_angle(self) -> f64 {
        self.second as f64 / 60.0 * TAU
    }
}

impl FromStr for ClockTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim().split(':');
        let hour = split.next().context("time is empty")?.parse()?;
        let minute = split.next().unwrap_or("0").parse()?;
        let second = split.next().unwrap_or("0").parse()?;

        Ok(ClockTime::new(hour, minute, second))
    }
}
