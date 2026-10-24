use super::world_boss_cache::WorldBossCache;
use super::world_boss_tracker::WorldBossTracker;
use crate::core::game::GameEvent;
use crate::core::game::InstanceTimer;
use crate::core::game::MapId;
use crate::core::game::OdyleEnergy;
use crate::core::game::WorldBossList;
use crate::core::game::WorldSnapshot;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct WorldTracker {
    energy: Option<OdyleEnergy>,
    timers: BTreeMap<(MapId, u8), InstanceTimer>,
    bosses: WorldBossTracker,
}

impl WorldTracker {
    pub(super) fn restore_bosses(&mut self, lists: &[WorldBossList], server: Option<u16>) {
        self.bosses.restore(lists, server);
    }

    pub(super) fn boss_cache(&self) -> &WorldBossCache {
        self.bosses.cache()
    }

    pub(super) fn replace_boss_cache(&mut self, cache: WorldBossCache) {
        self.bosses.replace_cache(cache);
    }

    pub(super) fn reset(&mut self) {
        self.energy = None;
        self.timers.clear();
        self.bosses.reset();
    }

    pub(super) fn observe(&mut self, event: &GameEvent, server: Option<u16>) {
        self.bosses.observe(event, server);

        match event {
            GameEvent::OdyleEnergy { energy } => self.energy = Some(energy.clone()),
            GameEvent::InstanceTimer { timer } => {
                if self.timers.len() < 256
                    || self.timers.contains_key(&(timer.map_id(), timer.kind()))
                {
                    self.timers
                        .insert((timer.map_id(), timer.kind()), timer.clone());
                }
            }

            GameEvent::MapChanged { map_id } => {
                self.timers.retain(|(map, _), _| map == map_id);
            }

            _ => {}
        }
    }

    pub(super) fn snapshot(&self, now_us: u64) -> WorldSnapshot {
        WorldSnapshot::new(
            self.energy.clone(),
            self.timers.values().cloned().collect(),
            self.bosses.snapshot(now_us),
        )
    }
}
