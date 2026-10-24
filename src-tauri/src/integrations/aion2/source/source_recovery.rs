use super::session_checkpoint::SessionCheckpoint;
use crate::core::game::GameSnapshot;
use crate::core::game::SourceStatus;
use crate::integrations::aion2::capture::CaptureProcess;
use crate::integrations::aion2::network::StreamKey;
use crate::integrations::aion2::state::GameState;
use crate::integrations::aion2::state::PlayerRecovery;
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub(super) struct SourceRecovery {
    path: PathBuf,
    pending: Option<SessionCheckpoint>,
    saved: Vec<u8>,
    failed_at: Option<Instant>,
    saved_at: Option<Instant>,
}

impl SourceRecovery {
    // Character metadata is small; reject oversized or incompatible cache files.
    const MAX_BYTES: u64 = 1024 * 1024;
    const RETRY_DELAY: Duration = Duration::from_secs(5);
    // Resource changes are frequent in combat; bound durable writes to one per second.
    const SAVE_INTERVAL: Duration = Duration::from_secs(1);

    pub(super) fn new(path: PathBuf) -> Self {
        Self {
            path,
            pending: None,
            saved: Vec::new(),
            failed_at: None,
            saved_at: None,
        }
    }

    pub(super) fn load(&mut self) {
        self.pending = None;

        let Ok(file) = File::open(&self.path) else {
            return;
        };

        let mut bytes = Vec::new();

        if file
            .take(Self::MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .is_ok()
            && bytes.len() as u64 <= Self::MAX_BYTES
        {
            self.pending = serde_json::from_slice(&bytes).ok();

            if self.pending.is_some() {
                self.saved = bytes;
            }
        }
    }

    /// Called once decoded traffic establishes the active stream, before applying its events.
    pub(super) fn restore(
        &mut self,
        state: &mut GameState,
        process: Option<&CaptureProcess>,
        stream: Option<&StreamKey>,
        now_us: u64,
    ) -> Result<bool, String> {
        let Some(checkpoint) = self.pending.as_ref() else {
            return Ok(false);
        };

        if state.has_player() || process.is_none_or(|process| !checkpoint.same_process(process)) {
            self.pending = None;

            return Ok(false);
        }

        if !process
            .zip(stream)
            .is_some_and(|(process, stream)| checkpoint.matches(process, stream))
        {
            return Ok(false);
        }

        state.restore_player(checkpoint.player())?;
        state.restore_bosses(checkpoint.legacy_bosses());
        state.restore_npcs(checkpoint.map_id(), checkpoint.npcs(), now_us)?;
        self.pending = None;

        Ok(true)
    }

    pub(super) fn clear(&mut self) {
        self.pending = None;

        if self.saved.is_empty() {
            return;
        }

        if let Err(error) = fs::remove_file(&self.path)
            && error.kind() != ErrorKind::NotFound
        {
            self.failed_at = Some(Instant::now());
            eprintln!("Cannot remove game session checkpoint: {error}");

            return;
        }

        self.saved.clear();
    }

    pub(super) fn save(
        &mut self,
        game: &GameSnapshot,
        process: Option<CaptureProcess>,
        stream: Option<&StreamKey>,
    ) {
        if game.header().status() != SourceStatus::Live || process.is_none() {
            return;
        }

        // Background sockets may speak first. Wait for the cached world connection
        // instead of deleting its checkpoint while the character is still unknown.
        if self.pending.is_some() {
            return;
        }

        if self
            .saved_at
            .is_some_and(|last| last.elapsed() < Self::SAVE_INTERVAL)
        {
            return;
        }

        if self
            .failed_at
            .is_some_and(|last| last.elapsed() < Self::RETRY_DELAY)
        {
            return;
        }

        let Some(player) = PlayerRecovery::capture(game) else {
            self.clear();

            return;
        };

        let Some((process, stream)) = process.zip(stream) else {
            return;
        };

        let checkpoint = SessionCheckpoint::new(process, stream.clone(), player).with_npcs(game);

        if let Err(error) = self.write(&checkpoint) {
            self.failed_at = Some(Instant::now());
            eprintln!("Cannot save game session checkpoint: {error}");
        }
    }

    fn write(&mut self, checkpoint: &SessionCheckpoint) -> Result<(), String> {
        let bytes = serde_json::to_vec(checkpoint).map_err(|error| error.to_string())?;

        if bytes == self.saved {
            self.saved_at = Some(Instant::now());

            return Ok(());
        }

        let temporary = self.path.with_extension("tmp");

        let mut file = File::create(&temporary).map_err(|error| error.to_string())?;

        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(temporary, &self.path).map_err(|error| error.to_string())?;
        self.saved = bytes;
        self.failed_at = None;
        self.saved_at = Some(Instant::now());

        Ok(())
    }

    pub(super) fn flush(
        &mut self,
        game: &GameSnapshot,
        process: Option<CaptureProcess>,
        stream: Option<&StreamKey>,
    ) {
        self.saved_at = None;
        self.save(game, process, stream);
    }
}
