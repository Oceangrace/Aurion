use super::unit_state::UnitState;
use crate::core::game::EntityId;
use crate::core::game::PartyRoster;
use crate::core::game::UnitSnapshot;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct PartyTracker {
    roster: Option<PartyRoster>,
}

impl PartyTracker {
    pub(super) fn defeated(
        &self,
        player: Option<EntityId>,
        entities: &BTreeMap<EntityId, UnitState>,
    ) -> bool {
        if player
            .and_then(|id| entities.get(&id))
            .is_none_or(|unit| unit.current_health() != Some(0))
        {
            return false;
        }

        self.roster.as_ref().is_none_or(|roster| {
            roster.complete()
                && roster.members().iter().all(|member| {
                    entities.iter().any(|(id, unit)| {
                        unit.identity_info().same_character(member.identity())
                            && self.contains(*id, entities)
                            && unit.current_health() == Some(0)
                    })
                })
        })
    }

    pub(super) fn contains(&self, id: EntityId, entities: &BTreeMap<EntityId, UnitState>) -> bool {
        let Some(unit) = entities.get(&id) else {
            return false;
        };

        let identity = unit.identity_info();

        if unit.npc_id().is_some()
            || unit.owner_id().is_some()
            || identity.name().is_none()
            || identity.server_id().is_none()
        {
            return false;
        }

        let in_roster = self.roster.as_ref().is_some_and(|roster| {
            roster
                .members()
                .iter()
                .any(|member| member.identity().same_character(identity))
        });

        let linked = Self::unique(entities.iter().filter_map(|(id, other)| {
            (other.npc_id().is_none()
                && other.owner_id().is_none()
                && other.identity_info().same_character(identity))
            .then_some((*id, other.present()))
        }));

        in_roster && linked == Some(id)
    }

    fn unique(candidates: impl Iterator<Item = (EntityId, bool)>) -> Option<EntityId> {
        let mut present = Vec::new();

        let mut historical = Vec::new();

        for (id, visible) in candidates {
            historical.push(id);

            if visible {
                present.push(id);
            }
        }

        match present.as_slice() {
            [id] => Some(*id),
            [] => match historical.as_slice() {
                [id] => Some(*id),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn observe(&mut self, roster: PartyRoster) {
        self.roster = Some(roster);
    }

    pub(super) fn snapshot(
        &self,
        entities: &BTreeMap<EntityId, UnitSnapshot>,
    ) -> Option<PartyRoster> {
        let roster = self.roster.as_ref()?;

        let members = roster
            .members()
            .iter()
            .map(|member| {
                let entity_id = Self::unique(entities.values().filter_map(|unit| {
                    (unit.identity().same_character(member.identity())
                        && unit.npc_id().is_none()
                        && unit.owner_id().is_none())
                    .then_some((unit.id(), unit.present()))
                }));

                member.clone().with_entity(entity_id)
            })
            .collect();

        Some(PartyRoster::new(
            members,
            roster.complete(),
            roster.dungeon_id(),
        ))
    }
}
