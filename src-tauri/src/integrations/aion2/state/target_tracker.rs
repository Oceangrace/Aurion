use super::unit_state::UnitState;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::TargetObservation;
use std::collections::BTreeMap;

pub(super) struct TargetTracker {
    observation: TargetObservation,
    pending: Option<EntityId>,
}

impl Default for TargetTracker {
    fn default() -> Self {
        Self {
            observation: TargetObservation::Unknown,
            pending: None,
        }
    }
}

impl TargetTracker {
    pub(super) fn snapshot(&self) -> TargetObservation {
        self.observation.clone()
    }

    pub(super) fn entity_id(&self) -> Option<EntityId> {
        self.observation.entity_id()
    }

    pub(super) fn observe(
        &mut self,
        event: &GameEvent,
        entities: &BTreeMap<EntityId, UnitState>,
        player: Option<EntityId>,
    ) {
        match event {
            GameEvent::TargetSelected { entity_id } => {
                self.pending = *entity_id;
                self.observation = match entity_id {
                    Some(id)
                        if entities
                            .get(id)
                            .is_none_or(|unit| unit.current_health() != Some(0)) =>
                    {
                        TargetObservation::Selected(*id)
                    }
                    _ => TargetObservation::Cleared,
                };
            }
            GameEvent::SkillUsed {
                entity_id,
                source_id,
                ..
            } if player == Some(*source_id)
                && matches!(
                    self.observation,
                    TargetObservation::Unknown | TargetObservation::LastSkillRecipient(_)
                ) =>
            {
                if let Some(id) = entity_id
                    && id != source_id
                    && entities.contains_key(id)
                {
                    self.observation = TargetObservation::LastSkillRecipient(*id);
                }
            }
            GameEvent::Despawned { entity_id } => {
                if self.pending == Some(*entity_id) {
                    self.pending = None;
                }

                self.clear(*entity_id);
            }
            GameEvent::Died { entity_id } => self.clear(*entity_id),
            GameEvent::MapChanged { .. } => *self = Self::default(),
            _ => {}
        }

        self.confirm(entities);
    }

    fn clear(&mut self, id: EntityId) {
        if self.entity_id() == Some(id) {
            self.observation = match self.observation {
                TargetObservation::Selected(_) | TargetObservation::Cleared => {
                    TargetObservation::Cleared
                }
                _ => TargetObservation::Unknown,
            };
        }
    }

    fn confirm(&mut self, entities: &BTreeMap<EntityId, UnitState>) {
        let Some(id) = self.pending else {
            return;
        };

        if !matches!(self.observation, TargetObservation::Cleared) {
            self.pending = None;

            return;
        }

        if entities
            .get(&id)
            .is_some_and(|unit| unit.present() && unit.current_health() != Some(0))
        {
            self.observation = TargetObservation::Selected(id);
            self.pending = None;
        }
    }
}
