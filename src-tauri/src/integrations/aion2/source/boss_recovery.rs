use super::boss_checkpoint::BossCheckpoint;
use crate::integrations::aion2::capture::CaptureProcess;
use crate::integrations::aion2::state::WorldBossCache;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub(super) struct BossRecovery {
    path: PathBuf,
    process: Option<CaptureProcess>,
    saved: Vec<u8>,
    attempted_at: Option<Instant>,
}

impl BossRecovery {
    const MAX_BYTES: u64 = 1_048_576;
    const SAVE_INTERVAL: Duration = Duration::from_secs(1);

    pub(super) fn new(path: PathBuf) -> Self {
        Self {
            path,
            process: None,
            saved: Vec::new(),
            attempted_at: None,
        }
    }

    pub(super) fn synchronize(
        &mut self,
        process: &CaptureProcess,
        now_us: u64,
    ) -> Option<WorldBossCache> {
        if self.process.as_ref() == Some(process) {
            return None;
        }

        self.process = Some(process.clone());
        self.saved.clear();
        self.attempted_at = None;
        Some(self.read(process, now_us).unwrap_or_default())
    }

    fn read(&mut self, process: &CaptureProcess, now_us: u64) -> Option<WorldBossCache> {
        let mut bytes = Vec::new();

        File::open(&self.path)
            .ok()?
            .take(Self::MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .ok()?;

        if bytes.len() as u64 > Self::MAX_BYTES {
            return None;
        }

        let checkpoint: BossCheckpoint = serde_json::from_slice(&bytes).ok()?;

        let cache = checkpoint.cache(process, now_us)?;

        self.saved = bytes;

        Some(cache)
    }

    pub(super) fn save(&mut self, cache: &WorldBossCache) {
        if self
            .attempted_at
            .is_some_and(|at| at.elapsed() < Self::SAVE_INTERVAL)
        {
            return;
        }

        self.attempted_at = Some(Instant::now());

        if let Err(error) = self.write(cache) {
            eprintln!("Cannot save world boss cache: {error}");
        }
    }

    pub(super) fn flush(&mut self, cache: &WorldBossCache) {
        self.attempted_at = None;
        self.save(cache);
    }

    fn write(&mut self, cache: &WorldBossCache) -> Result<(), String> {
        let Some(process) = self.process.clone() else {
            return Ok(());
        };

        let bytes = serde_json::to_vec(&BossCheckpoint::new(process, cache.clone()))
            .map_err(|error| error.to_string())?;

        if bytes == self.saved {
            return Ok(());
        }

        if bytes.len() as u64 > Self::MAX_BYTES {
            return Err("World boss cache exceeds the size limit".into());
        }

        let temporary = self.path.with_extension("tmp");

        let mut file = File::create(&temporary).map_err(|error| error.to_string())?;

        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(temporary, &self.path).map_err(|error| error.to_string())?;
        self.saved = bytes;

        Ok(())
    }
}
