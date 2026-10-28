use super::npc_recovery::NpcRecovery;
use super::player_recovery::PlayerRecovery;
use super::self_tracker::SelfTracker;
use super::server_clock::ServerClock;
use super::target_tracker::TargetTracker;
use super::unit_state::UnitState;
use crate::core::game::ActionSnapshot;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::SkillId;
use crate::core::game::TargetObservation;
use crate::core::game::UnitSnapshot;
use crate::integrations::aion2::catalog::RuntimeCatalog;
use std::collections::BTreeMap;

pub(super) struct EntityTracker {
    player: SelfTracker,
    entities: BTreeMap<EntityId, UnitState>,
    target: TargetTracker,
}

impl EntityTracker {
    const MAX_ENTITIES: usize = 4096;

    pub(super) fn new() -> Self {
        Self {
            player: SelfTracker::new(),
            entities: BTreeMap::new(),
            target: TargetTracker::default(),
        }
    }

    pub(super) fn player(&self) -> Option<EntityId> {
        self.player.id()
    }

    pub(super) fn server(&self) -> Option<u16> {
        self.player.server()
    }

    pub(super) fn has_player(&self) -> bool {
        self.player()
            .is_some_and(|id| self.entities.contains_key(&id))
    }

    pub(super) fn units(&self) -> &BTreeMap<EntityId, UnitState> {
        &self.entities
    }

    pub(super) fn target(&self) -> TargetObservation {
        self.target.snapshot()
    }

    pub(super) fn reset(&mut self, interrupted: bool) {
        self.entities.clear();
        self.target = TargetTracker::default();

        if !interrupted {
            self.player = SelfTracker::new();
        }
    }

    fn travel(&mut self) {
        let own = self.player();

        self.entities.retain(|id, _| Some(*id) == own);

        for unit in self.entities.values_mut() {
            unit.travel();
        }
    }

    pub(super) fn identity_changed(&self, event: &GameEvent) -> bool {
        if self.player().is_none() {
            return false;
        }

        match event {
            GameEvent::SelfIdentified {
                entity_id,
                authoritative: true,
            } => self.player.id() != Some(*entity_id),

            GameEvent::Identity {
                entity_id,
                own,
                name,
                server_id,
                class_name,
                ..
            } => {
                let is_player = self.player.id() == Some(*entity_id);

                (*own && !is_player)
                    || (is_player
                        && server_id
                            .zip(self.player.server())
                            .is_some_and(|(new, old)| new != old))
                    || (is_player
                        && self.entities.get(entity_id).is_some_and(|unit| {
                            let identity = unit.identity_info();

                            name.as_deref()
                                .zip(identity.name())
                                .is_some_and(|(new, old)| new != old)
                                || server_id
                                    .zip(identity.server_id())
                                    .is_some_and(|(new, old)| new != old)
                                || class_name
                                    .zip(identity.class_name())
                                    .is_some_and(|(new, old)| new != old)
                        }))
            }

            _ => false,
        }
    }

    pub(super) fn restoration(&self, event: &GameEvent) -> Option<GameEvent> {
        let GameEvent::SelfRestoration {
            entity_id,
            skill_id,
            value,
            hit,
        } = event
        else {
            return None;
        };

        let known = self.player() == Some(*entity_id)
            || self.entities.get(entity_id).is_some_and(|unit| {
                unit.npc_id().is_none()
                    && (unit.identity_info().character_id().is_some()
                        || unit.identity_info().class_name().is_some())
            });

        known.then(|| GameEvent::Healing {
            entity_id: *entity_id,
            source_id: *entity_id,
            skill_id: *skill_id,
            value: *value,
            hit: Some(hit.clone()),
        })
    }

    pub(super) fn restore_player(&mut self, recovery: &PlayerRecovery) -> Result<(), String> {
        if let Some(id) = self.player() {
            self.entity(id)?
                .restore_resources(recovery.health(), recovery.mana());
        }

        Ok(())
    }

    pub(super) fn restore_npcs(&mut self, npcs: &[NpcRecovery], now_us: u64) -> Result<(), String> {
        for npc in npcs.iter().take(Self::MAX_ENTITIES / 2) {
            if npc.valid(now_us) && !self.entities.contains_key(&npc.entity_id()) {
                self.entity(npc.entity_id())?.restore_npc(npc);
            }
        }

        Ok(())
    }

    pub(super) fn observe(
        &mut self,
        event: &GameEvent,
        at_us: u64,
        catalog: &RuntimeCatalog,
    ) -> Result<(), String> {
        match event {
            GameEvent::CharacterIdentified {
                entity_id,
                character_id,
                level,
            } => self
                .entity(*entity_id)?
                .identify_character(*character_id, *level),
            GameEvent::CastStarted {
                source_id,
                preparation,
            } => self.entity(*source_id)?.begin_cast(preparation.clone()),
            GameEvent::PropertiesChanged {
                entity_id,
                health,
                mana,
                stamina,
            } => {
                let unit = self.entity(*entity_id)?;

                unit.health(health.current(), health.maximum(), at_us);
                unit.mana(mana);
                unit.stamina(stamina);
            }

            GameEvent::Attribution {
                entity_id,
                actor_id,
            } => {
                let summon = self
                    .entities
                    .get(entity_id)
                    .and_then(UnitState::npc_id)
                    .and_then(|id| catalog.npc(id.get()))
                    .is_some_and(|npc| npc.player_summon());

                if summon {
                    self.entity(*entity_id)?.owner(*actor_id);
                }
            }

            GameEvent::ServerBattleChanged { entity_id, battle } => {
                self.entity(*entity_id)?.server_battle(battle.clone())
            }

            GameEvent::MapChanged { .. } => self.travel(),

            GameEvent::SelfIdentified {
                entity_id,
                authoritative,
            } => {
                self.entity(*entity_id)?;
                self.player.identify(*entity_id, *authoritative);
            }

            GameEvent::Identity {
                entity_id,
                name,
                own,
                server_id,
                class_name,
                level,
            } => {
                if *own {
                    self.player.identify(*entity_id, true);
                }

                self.entity(*entity_id)?
                    .identity(name, *class_name, *server_id, *level);
            }

            GameEvent::Spawned {
                entity_id,
                npc_id,
                health,
                ..
            } => self
                .entity(*entity_id)?
                .spawn(*npc_id, health.as_ref(), at_us),
            GameEvent::EffectApplied {
                entity_id,
                effect,
                restarted,
            }
            | GameEvent::EffectUpdated {
                entity_id,
                effect,
                restarted,
            } => {
                let limit = catalog
                    .effect(effect.effect_id().get())
                    .filter(|_| matches!(event, GameEvent::EffectApplied { .. }))
                    .map(|metadata| metadata.stack_limit());

                self.entity(*entity_id)?
                    .put_effect(effect.clone(), at_us, *restarted, limit)?;
            }

            GameEvent::EffectsListed { entity_id, effects } => {
                self.entity(*entity_id)?.list_effects(effects, at_us)?
            }

            GameEvent::EffectRemoved {
                entity_id,
                instance_id,
                ..
            } => self.entity(*entity_id)?.remove_effect(*instance_id),
            GameEvent::SkillUsed {
                entity_id,
                source_id,
                skill_id,
            } => {
                let metadata = catalog.skill(skill_id.get());

                let action = ActionSnapshot::new(
                    *skill_id,
                    at_us,
                    metadata.map(|skill| skill.casting_ms()),
                    metadata.map(|skill| skill.cooldown_ms()),
                    metadata.and_then(|skill| skill.cooldown_group()),
                )
                .with_recipient(entity_id.filter(|id| id != source_id));

                let charge_skill = metadata
                    .and_then(|skill| skill.charge_skill())
                    .map(SkillId::new)
                    .transpose()?;

                let unit = self.entity(*source_id)?;

                if let Some(charge_skill) = charge_skill {
                    unit.finish_cast(charge_skill);
                }

                unit.action(action);
            }

            GameEvent::Damage {
                entity_id,
                source_id,
                ..
            }
            | GameEvent::Healing {
                entity_id,
                source_id,
                ..
            }
            | GameEvent::HitLanded {
                entity_id,
                source_id,
                ..
            }
            | GameEvent::DamageUnknown {
                entity_id,
                source_id,
                ..
            }
            | GameEvent::Miss {
                entity_id,
                source_id,
                ..
            }
            | GameEvent::Resist {
                entity_id,
                source_id,
                ..
            } => {
                self.entity(*entity_id)?;
                self.entity(*source_id)?;
            }

            GameEvent::Died { entity_id } => {
                self.entity(*entity_id)?.health(Some(0), None, at_us);
            }

            GameEvent::Despawned { entity_id } => {
                self.entity(*entity_id)?.despawn();
            }

            _ => {}
        }

        self.target.observe(event, &self.entities, self.player.id());

        if let Some(server) = self
            .player()
            .and_then(|id| self.entities.get(&id))
            .and_then(|unit| unit.identity_info().server_id())
        {
            self.player.locate(server);
        }

        Ok(())
    }

    pub(super) fn snapshot(
        &mut self,
        clock: &ServerClock,
        now_us: u64,
        catalog: &RuntimeCatalog,
    ) -> BTreeMap<EntityId, UnitSnapshot> {
        if let Some(id) = self.player()
            && let Some(player) = self.entities.get_mut(&id)
        {
            player.observe_class(catalog);
        }

        self.entities
            .iter()
            .map(|(id, unit)| (*id, unit.snapshot(*id, clock, now_us, catalog)))
            .collect()
    }

    pub(super) fn entity(&mut self, id: EntityId) -> Result<&mut UnitState, String> {
        if self.entities.len() >= Self::MAX_ENTITIES && !self.entities.contains_key(&id) {
            // Units leave only on map change; evict a despawned one instead of failing the source.
            let keep = [self.player.id(), self.target.entity_id()];

            let stale = self
                .entities
                .iter()
                .find(|(id, unit)| !unit.present() && !keep.contains(&Some(**id)))
                .map(|(id, _)| *id)
                .ok_or("Observed entity limit")?;

            self.entities.remove(&stale);
        }

        Ok(self.entities.entry(id).or_insert_with(UnitState::new))
    }
}
