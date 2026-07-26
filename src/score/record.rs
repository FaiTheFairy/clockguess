use serde::{Deserialize, Serialize};

use crate::{
    cli::{Cli, GameMode},
    difficulty::Difficulty,
    score::SessionStats,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub mode: GameMode,
    pub difficulty: Difficulty,
    pub points: u32,
    pub correct: u32,
    pub incorrect: u32,
    pub exact: u32,
    pub time_limit_seconds: Option<u64>,
    pub round_limit: Option<u32>,
    pub total_answer_time_ms: u64,
    pub played_at_unix: u64,
}

impl ScoreRecord {
    pub fn from_session(cli: &Cli, stats: &SessionStats) -> Self {
        Self {
            mode: cli.mode,
            difficulty: cli.difficulty,
            points: stats.points(),
            correct: stats.correct(),
            incorrect: stats.incorrect(),
            exact: stats.exact(),
            time_limit_seconds: if cli.mode == GameMode::RapidFire {
                Some(cli.rapid_seconds)
            } else {
                None
            },
            round_limit: match cli.mode {
                GameMode::Challenge => Some(cli.rounds),
                _ => None,
            },
            total_answer_time_ms: u64::try_from(stats.total_answer_time().as_millis())
                .unwrap_or(u64::MAX),
            played_at_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
