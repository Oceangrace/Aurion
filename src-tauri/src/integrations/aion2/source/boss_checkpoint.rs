use crate::integrations::aion2::capture::CaptureProcess;
use crate::integrations::aion2::state::WorldBossCache;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BossCheckpoint {
    version: u8,
    process: CaptureProcess,
    cache: WorldBossCache,
}

impl BossCheckpoint {
    pub(super) fn new(process: CaptureProcess, cache: WorldBossCache) -> Self {
        Self {
            version: 1,
            process,
            cache,
        }
    }

    pub(super) fn cache(self, process: &CaptureProcess, now_us: u64) -> Option<WorldBossCache> {
        (self.version == 1 && &self.process == process && self.cache.valid(now_us))
            .then_some(self.cache)
    }
}
