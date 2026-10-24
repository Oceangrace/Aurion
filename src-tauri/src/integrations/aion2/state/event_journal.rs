use super::entity_tracker::EntityTracker;
use crate::core::game::EventRecord;
use crate::core::game::GameEvent;
use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct EventJournal {
    sequence: u64,
    events: VecDeque<EventRecord>,
}

impl EventJournal {
    const MAX_EVENTS: usize = 512;
    const MAX_AGE_US: u64 = 60_000_000;

    pub(super) fn prepare(
        &self,
        event: &GameEvent,
        entities: &mut EntityTracker,
    ) -> Result<Option<GameEvent>, String> {
        let record = match event {
            GameEvent::HealingPool { .. } | GameEvent::SkillConfirmed { .. } => true,
            GameEvent::PropertiesChanged {
                entity_id,
                health,
                mana,
                ..
            } => entities.entity(*entity_id)?.observe_resources(health, mana),
            GameEvent::ComboAvailable { .. } | GameEvent::ComboUnavailable { .. } => {
                entities.player().is_some()
            }
            _ => event.event_type().is_some(),
        };

        if !record {
            return Ok(None);
        }

        let removed = match event {
            GameEvent::EffectRemoved {
                entity_id,
                instance_id,
                ..
            } => entities
                .units()
                .get(entity_id)
                .and_then(|unit| unit.effect(*instance_id))
                .cloned(),
            _ => None,
        };

        Ok(Some(event.clone().with_removed_effect(removed)))
    }

    pub(super) fn append(&mut self, event: GameEvent, at_us: u64) {
        self.sequence += 1;
        self.events
            .push_back(EventRecord::new(self.sequence, at_us, event));

        while self.events.len() > Self::MAX_EVENTS {
            self.events.pop_front();
        }
    }

    pub(super) fn clear(&mut self) {
        self.events.clear();
    }

    pub(super) fn snapshot(&mut self, now_us: u64) -> Vec<EventRecord> {
        while self
            .events
            .front()
            .is_some_and(|event| now_us.saturating_sub(event.at_us()) > Self::MAX_AGE_US)
        {
            self.events.pop_front();
        }

        self.events.iter().cloned().collect()
    }
}
