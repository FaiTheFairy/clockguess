use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use directories::ProjectDirs;

use crate::score::record::ScoreRecord;

#[derive(Debug)]
pub struct ScoreStore {
    path: PathBuf,
}

impl ScoreStore {
    pub fn try_new() -> io::Result<Self> {
        let project_dir = ProjectDirs::from("com", "faithefairy", "clockguess")
            .ok_or_else(|| io::Error::other("could not determine application data directory"))?;

        let path = project_dir.data_local_dir().join("scores.json");

        Ok(Self { path })
    }

    pub fn load(&self) -> io::Result<Vec<ScoreRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&self.path)?;

        serde_json::from_str(&contents)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn pretty_write_scores(&self, output: &mut impl Write) -> anyhow::Result<()> {
        let mut records = self.load()?;

        records.sort_by_key(|record| std::cmp::Reverse(record.points));

        writeln!(
            output,
            "{:>5} {:<12} {:>8} {:>8} {:>8}",
            "Rank", "Mode", "Score", "Correct", "Exact"
        )?;

        for (index, record) in records.iter().take(10).enumerate() {
            writeln!(
                output,
                "{:<5} {:<12} {:>8} {:>8} {:>8}",
                index + 1,
                record.mode,
                record.points,
                record.correct,
                record.exact
            )?;
        }

        Ok(())
    }

    pub fn save(&self, records: &[ScoreRecord]) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temporary_path = self.path.join(".tmp");

        let contents = serde_json::to_string_pretty(records).map_err(io::Error::other)?;

        fs::write(&temporary_path, contents)?;
        fs::rename(temporary_path, &self.path)?;

        Ok(())
    }

    pub fn add(&self, record: ScoreRecord) -> io::Result<()> {
        let mut records = self.load()?;
        records.push(record);
        self.save(&records)
    }
}
