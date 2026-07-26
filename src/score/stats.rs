use std::time::Duration;

use crate::difficulty::Difficulty;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RoundOutcome {
    pub difference_seconds: u32,
    pub elapsed: Duration,
    pub correct: bool,
}

impl RoundOutcome {
    pub const fn is_exact(self) -> bool {
        self.difference_seconds == 0
    }
}

#[derive(Clone, Debug, Default)]
pub struct SessionStats {
    correct: u32,
    incorrect: u32,
    exact: u32,
    points: u32,
    total_answer_time: Duration,
    best_answer_time: Option<Duration>,
    quit: bool,
}

impl SessionStats {
    #[allow(dead_code)]
    pub const fn correct(&self) -> u32 {
        self.correct
    }

    #[allow(dead_code)]
    pub const fn incorrect(&self) -> u32 {
        self.incorrect
    }

    #[allow(dead_code)]
    pub const fn exact(&self) -> u32 {
        self.exact
    }

    pub const fn points(&self) -> u32 {
        self.points
    }

    pub const fn attempted(&self) -> u32 {
        self.correct + self.incorrect
    }

    pub fn accuracy(&self) -> Option<f64> {
        let attempted = self.attempted();

        if attempted == 0 {
            return None;
        }

        Some(f64::from(self.correct) / f64::from(attempted))
    }

    pub fn average_answer_time(&self) -> Option<Duration> {
        let attempted = self.attempted();

        if attempted == 0 {
            return None;
        }

        Some(self.total_answer_time / attempted)
    }

    pub const fn best_answer_time(&self) -> Option<Duration> {
        self.best_answer_time
    }

    pub fn record(&mut self, outcome: RoundOutcome, difficulty: Difficulty) {
        self.total_answer_time += outcome.elapsed;

        self.best_answer_time = Some(
            self.best_answer_time
                .map_or(outcome.elapsed, |current_best| {
                    current_best.min(outcome.elapsed)
                }),
        );

        if outcome.correct {
            self.correct += 1;

            if outcome.is_exact() {
                self.exact += 1;
            }

            self.points = self
                .points
                .saturating_add(calculate_points(difficulty, outcome.difference_seconds));
        } else {
            self.incorrect += 1;
        }
    }

    pub const fn total_answer_time(&self) -> Duration {
        self.total_answer_time
    }

    pub const fn quit(&mut self) {
        self.quit = true;
    }

    pub const fn has_quit(&self) -> bool {
        self.quit
    }
}

pub const fn calculate_points(difficulty: Difficulty, difference_seconds: u32) -> u32 {
    let base_points = match difficulty {
        Difficulty::Hour => 100,
        Difficulty::TenMinutes => 200,
        Difficulty::FiveMinutes => 300,
        Difficulty::Minute => 500,
        Difficulty::ThirtySeconds => 750,
        Difficulty::Exact => 1000,
    };

    if difference_seconds == 0 {
        base_points + base_points / 2
    } else {
        base_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_answer_receives_bonus() {
        assert_eq!(calculate_points(Difficulty::Minute, 0), 750,);
    }

    #[test]
    fn non_exact_correct_answer_receives_base_points() {
        assert_eq!(calculate_points(Difficulty::Minute, 15), 500,);
    }

    #[test]
    fn records_correct_answer() {
        let mut stats = SessionStats::default();

        stats.record(
            RoundOutcome {
                difference_seconds: 0,
                elapsed: Duration::from_secs(2),
                correct: true,
            },
            Difficulty::Minute,
        );

        assert_eq!(stats.attempted(), 1);
        assert_eq!(stats.correct(), 1);
        assert_eq!(stats.incorrect(), 0);
        assert_eq!(stats.exact(), 1);
        assert_eq!(stats.points(), 750);
    }

    #[test]
    fn calculates_accuracy() {
        let mut stats = SessionStats::default();

        stats.record(
            RoundOutcome {
                difference_seconds: 0,
                elapsed: Duration::from_secs(2),
                correct: true,
            },
            Difficulty::Minute,
        );

        stats.record(
            RoundOutcome {
                difference_seconds: 120,
                elapsed: Duration::from_secs(4),
                correct: false,
            },
            Difficulty::Minute,
        );

        assert_eq!(stats.accuracy(), Some(0.5));
    }
}
