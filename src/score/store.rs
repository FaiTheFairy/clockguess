use std::{fs, io, path::PathBuf};

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

    pub unsafe fn new() -> Self {
        Self::try_new().unwrap()
    }

    pub fn load(&self) -> io::Result<Vec<ScoreRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&self.path)?;

        serde_json::from_str(&contents)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn save(&self, records: &[ScoreRecord]) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(records).map_err(io::Error::other)?;

        fs::write(&self.path, contents)
    }

    pub fn add(&self, record: ScoreRecord) -> io::Result<()> {
        let mut records = self.load()?;
        records.push(record);
        self.save(&records)
    }
}
