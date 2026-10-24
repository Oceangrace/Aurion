use super::combat_ledger::CombatLedger;
use super::combat_tracker::CombatTracker;
use super::entity_tracker::EntityTracker;
use super::party_tracker::PartyTracker;
use super::unit_state::UnitState;
use crate::core::game::CombatRecord;
use crate::core::game::CombatSnapshot;
use crate::core::game::EntityId;
use crate::core::game::EventRecord;
use crate::core::game::GameEvent;
use crate::integrations::aion2::catalog::RuntimeCatalog;

pub(super) struct CombatSession {
    combat: CombatTracker,
    ledger: CombatLedger,
}

impl CombatSession {
    pub(super) fn new(idle_ms: u64) -> Self {
        Self {
            combat: CombatTracker::new(idle_ms),
            ledger: CombatLedger::default(),
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Self::new(self.combat.idle_ms());
    }

    pub(super) fn observe(
        &mut self,
        event: &GameEvent,
        at_us: u64,
        entities: &EntityTracker,
        party: &PartyTracker,
        catalog: &RuntimeCatalog,
    ) -> Vec<EventRecord> {
        let own_alive = entities
            .player()
            .and_then(|id| entities.units().get(&id))
            .is_none_or(|unit| unit.current_health() != Some(0));

        let source_friendly = Self::friendly(event.source(), entities, party)
            && (own_alive || event.source() != entities.player());

        let recipient_friendly = Self::friendly(event.recipient(), entities, party)
            && (own_alive || event.recipient() != entities.player());

        let candidate = if source_friendly && !recipient_friendly {
            event.recipient()
        } else if recipient_friendly && !source_friendly {
            event.source()
        } else {
            None
        };

        let opponent = candidate
            .and_then(|id| {
                entities
                    .units()
                    .get(&id)
                    .filter(|unit| unit.current_health() != Some(0))
                    .and_then(UnitState::npc_id)
                    .map(|npc| (id, npc))
            })
            .and_then(|(id, npc)| {
                catalog
                    .npc(npc.get())
                    .map(|traits| (id, npc, traits.boss()))
            });

        let derived = self.combat.observe(
            event,
            at_us,
            entities.player(),
            (source_friendly, recipient_friendly),
            opponent,
            matches!(event, GameEvent::Died { .. })
                && party.defeated(entities.player(), entities.units()),
        );

        if derived
            .iter()
            .any(|record| matches!(record.event(), GameEvent::CombatStarted { .. }))
        {
            self.ledger = Default::default();
        }

        let ends_now = matches!(event, GameEvent::Died { .. })
            && derived.iter().any(|record| {
                record.at_us() == at_us && matches!(record.event(), GameEvent::CombatEnded { .. })
            });

        if self.combat.snapshot(at_us, entities.player()).active() == Some(true) || ends_now {
            self.ledger.observe(event, at_us);
        }

        derived
    }

    fn friendly(id: Option<EntityId>, entities: &EntityTracker, party: &PartyTracker) -> bool {
        let Some(mut id) = id else {
            return false;
        };

        for _ in 0..8 {
            if entities.player() == Some(id) || party.contains(id, entities.units()) {
                return true;
            }

            let Some(owner) = entities.units().get(&id).and_then(UnitState::owner_id) else {
                return false;
            };

            id = owner;
        }

        false
    }

    pub(super) fn tick(&mut self, now_us: u64, player: Option<EntityId>) -> Vec<EventRecord> {
        self.combat.tick(now_us, player)
    }

    pub(super) fn snapshot(&self, now_us: u64, player: Option<EntityId>) -> CombatSnapshot {
        self.combat.snapshot(now_us, player)
    }

    pub(super) fn statistics(&self) -> Vec<CombatRecord> {
        self.ledger.snapshot()
    }
}
