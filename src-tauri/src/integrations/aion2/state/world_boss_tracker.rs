use super::world_boss_cache::WorldBossCache;
use crate::core::game::GameEvent;
use crate::core::game::MapId;
use crate::core::game::WorldBossList;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct WorldBossTracker {
    server: Option<u16>,
    lists: BTreeMap<MapId, WorldBossList>,
    cache: WorldBossCache,
}

impl WorldBossTracker {
    const MAX_MAPS: usize = 32;

    pub(super) fn reset(&mut self) {
        self.server = None;
        self.lists.clear();
    }

    pub(super) fn cache(&self) -> &WorldBossCache {
        &self.cache
    }

    pub(super) fn replace_cache(&mut self, cache: WorldBossCache) {
        self.reset();
        self.cache = cache;
    }

    fn bind(&mut self, server: Option<u16>) {
        if self.server == server {
            return;
        }

        if self.server.is_some() {
            self.lists.clear();
        }

        self.server = server;

        if let Some(server) = server {
            for list in self.lists.values() {
                self.cache.observe(server, list);
            }
        }
    }

    pub(super) fn observe(&mut self, event: &GameEvent, server: Option<u16>) {
        self.bind(server);

        match event {
            GameEvent::WorldBossList { list } => {
                if self
                    .lists
                    .get(&list.map_id())
                    .is_some_and(|previous| previous.observed_at_us() > list.observed_at_us())
                {
                    return;
                }

                if self.lists.len() < Self::MAX_MAPS || self.lists.contains_key(&list.map_id()) {
                    self.lists.insert(list.map_id(), list.clone());

                    if let Some(server) = self.server {
                        self.cache.observe(server, list);
                    }
                }
            }
            GameEvent::MapChanged { .. } => {
                for list in self.lists.values_mut() {
                    *list = list.clone().with_restored();
                }
            }
            _ => {}
        }
    }

    pub(super) fn restore(&mut self, lists: &[WorldBossList], server: Option<u16>) {
        self.bind(server);

        let Some(server) = self.server else {
            return;
        };

        for list in lists.iter().take(Self::MAX_MAPS) {
            self.cache.observe(server, list);
        }
    }

    pub(super) fn snapshot(&self, now_us: u64) -> Vec<WorldBossList> {
        let Some(server) = self.server else {
            return Vec::new();
        };

        let mut lists: BTreeMap<_, _> = self
            .cache
            .lists(server, now_us)
            .into_iter()
            .map(|list| (list.map_id(), list))
            .collect();

        for list in self
            .lists
            .values()
            .filter(|list| WorldBossCache::valid_list(list, now_us))
        {
            if lists
                .get(&list.map_id())
                .is_none_or(|cached| cached.observed_at_us() <= list.observed_at_us())
            {
                lists.insert(list.map_id(), list.clone());
            }
        }

        lists.into_values().collect()
    }
}
