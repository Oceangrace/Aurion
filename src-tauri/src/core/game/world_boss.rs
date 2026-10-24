use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorldBoss {
    world_map_id: u32,
    alive: bool,
    time_ms: Option<u64>,
}

impl WorldBoss {
    pub(crate) fn new(world_map_id: u32, alive: bool, time_ms: Option<u64>) -> Self {
        Self {
            world_map_id,
            alive,
            time_ms,
        }
    }

    pub(crate) fn world_map_id(&self) -> u32 {
        self.world_map_id
    }

    pub(crate) fn alive(&self) -> bool {
        self.alive
    }

    pub(crate) fn time_ms(&self) -> Option<u64> {
        self.time_ms
    }
}
