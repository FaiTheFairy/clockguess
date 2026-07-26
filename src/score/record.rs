use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{cli::GameMode, score::SessionStats};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub mode: GameMode,
    pub points: u32,
    pub correct: u32,
    pub incorrect: u32,
    pub exact: u32,
    pub total_answer_time: u64,
    pub played_at_unix: u64,
}

impl ScoreRecord {
    pub fn from_session(mode: GameMode, stats: &SessionStats) -> Self {
        Self {
            mode,
            points: stats.points(),
            correct: stats.correct(),
            incorrect: stats.incorrect(),
            exact: stats.exact(),
            total_answer_time: stats.total_answer_time().as_secs(),
            played_at_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
