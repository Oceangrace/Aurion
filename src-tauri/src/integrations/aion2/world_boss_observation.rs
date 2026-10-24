use crate::core::game::WorldBoss;
use crate::core::game::WorldBossList;

pub(crate) struct WorldBossObservation<'a> {
    list: Option<&'a WorldBossList>,
    boss: Option<&'a WorldBoss>,
    now_us: u64,
}

impl<'a> WorldBossObservation<'a> {
    const FRESH_US: u64 = 3_000_000;

    pub(crate) fn new(list: Option<&'a WorldBossList>, id: u32, now_us: u64) -> Self {
        Self {
            list,
            boss: list.and_then(|list| list.bosses().iter().find(|boss| boss.world_map_id() == id)),
            now_us,
        }
    }

    pub(crate) fn known(&self) -> bool {
        self.boss.is_some()
    }

    pub(crate) fn observed_alive(&self) -> Option<bool> {
        self.boss.map(WorldBoss::alive)
    }

    pub(crate) fn compatible_alive(&self) -> bool {
        self.observed_alive() == Some(true) || self.expected()
    }

    pub(crate) fn fresh_alive(&self) -> Option<bool> {
        let fresh = self.list.is_some_and(|list| {
            !list.restored()
                && self.now_us >= list.observed_at_us()
                && self.now_us - list.observed_at_us() <= Self::FRESH_US
        });

        self.boss.filter(|_| fresh).map(WorldBoss::alive)
    }

    pub(crate) fn respawns_at(&self) -> Option<u64> {
        self.boss
            .filter(|boss| !boss.alive())
            .and_then(WorldBoss::time_ms)
            .filter(|time| *time > 0)
    }

    pub(crate) fn remaining(&self) -> Option<u64> {
        self.respawns_at()
            .filter(|time| *time > self.now_us / 1000)
            .map(|time| time - self.now_us / 1000)
    }

    pub(crate) fn expected(&self) -> bool {
        self.respawns_at()
            .is_some_and(|time| time <= self.now_us / 1000)
    }

    pub(crate) fn restored(&self) -> bool {
        self.list.is_some_and(WorldBossList::restored)
    }

    pub(crate) fn age_ms(&self) -> Option<u64> {
        self.list
            .map(|list| self.now_us.saturating_sub(list.observed_at_us()) / 1000)
    }
}
