use crate::core::game::GameSnapshot;
use crate::core::game::MapId;
use crate::integrations::aion2::capture::CaptureProcess;
use crate::integrations::aion2::network::StreamKey;
use crate::integrations::aion2::state::NpcRecovery;
use crate::integrations::aion2::state::PlayerRecovery;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(super) struct SessionCheckpoint {
    version: u8,
    process: CaptureProcess,
    stream: StreamKey,
    player: PlayerRecovery,
    #[serde(default)]
    map_id: Option<MapId>,
    #[serde(default)]
    npcs: Vec<NpcRecovery>,
}

impl SessionCheckpoint {
    pub(super) fn new(process: CaptureProcess, stream: StreamKey, player: PlayerRecovery) -> Self {
        Self {
            version: 1,
            process,
            stream,
            player,
            map_id: None,
            npcs: Vec::new(),
        }
    }

    pub(super) fn with_npcs(mut self, game: &GameSnapshot) -> Self {
        self.map_id = game.map_id();
        self.npcs = NpcRecovery::capture(game);
        self
    }

    pub(super) fn npcs(&self) -> &[NpcRecovery] {
        &self.npcs
    }

    pub(super) fn map_id(&self) -> Option<MapId> {
        self.map_id
    }

    pub(super) fn matches(&self, process: &CaptureProcess, stream: &StreamKey) -> bool {
        self.same_process(process) && &self.stream == stream
    }

    pub(super) fn same_process(&self, process: &CaptureProcess) -> bool {
        self.version == 1 && &self.process == process
    }

    pub(super) fn player(&self) -> &PlayerRecovery {
        &self.player
    }
}
