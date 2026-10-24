use crate::core::game::MapId;
use crate::core::game::WorldBossList;
use crate::integrations::aion2::catalog::FieldBossCatalog;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldBossCache {
    servers: BTreeMap<u16, BTreeMap<MapId, WorldBossList>>,
}

impl WorldBossCache {
    const MAX_SERVERS: usize = 16;
    const MAX_MAPS: usize = 32;
    const MAX_AGE_US: u64 = 14 * 86_400_000_000;
    const CLOCK_TOLERANCE_US: u64 = 300_000_000;

    pub(crate) fn valid(&self, now_us: u64) -> bool {
        self.servers.len() <= Self::MAX_SERVERS
            && self.servers.iter().all(|(server, maps)| {
                (1000..=9999).contains(server)
                    && maps.len() <= Self::MAX_MAPS
                    && maps
                        .iter()
                        .all(|(map, list)| *map == list.map_id() && Self::valid_list(list, now_us))
            })
    }

    pub(crate) fn observe(&mut self, server: u16, list: &WorldBossList) {
        if !(1000..=9999).contains(&server)
            || list.restored()
            || !Self::valid_list(list, list.observed_at_us())
            || (!self.servers.contains_key(&server) && self.servers.len() >= Self::MAX_SERVERS)
        {
            return;
        }

        let maps = self.servers.entry(server).or_default();

        if maps
            .get(&list.map_id())
            .is_some_and(|old| old.observed_at_us() >= list.observed_at_us())
            || (!maps.contains_key(&list.map_id()) && maps.len() >= Self::MAX_MAPS)
        {
            return;
        }

        maps.insert(list.map_id(), list.clone());
    }

    pub(crate) fn lists(&self, server: u16, now_us: u64) -> Vec<WorldBossList> {
        self.servers
            .get(&server)
            .into_iter()
            .flat_map(BTreeMap::values)
            .filter(|list| Self::valid_list(list, now_us))
            .map(|list| list.clone().with_restored())
            .collect()
    }

    pub(super) fn valid_list(list: &WorldBossList, now_us: u64) -> bool {
        let mut ids = BTreeSet::new();

        !list.bosses().is_empty()
            && list.bosses().len() <= 200
            && list.observed_at_us() > 0
            && list.observed_at_us() <= now_us.saturating_add(Self::CLOCK_TOLERANCE_US)
            && now_us.saturating_sub(list.observed_at_us()) <= Self::MAX_AGE_US
            && list.bosses().iter().all(|boss| {
                ids.insert(boss.world_map_id())
                    && FieldBossCatalog::find(boss.world_map_id())
                        .is_some_and(|definition| definition.map_id == list.map_id().get())
                    && boss.time_ms().is_none_or(|time| {
                        time == 0
                            || time <= (list.observed_at_us() / 1000).saturating_add(3 * 86_400_000)
                    })
            })
    }
}
