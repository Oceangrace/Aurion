use super::effect_state::EffectState;
use super::npc_recovery::NpcRecovery;
use super::server_clock::ServerClock;
use crate::core::game::ActionSnapshot;
use crate::core::game::CastPreparation;
use crate::core::game::CharacterClass;
use crate::core::game::CharacterId;
use crate::core::game::EffectInstanceId;
use crate::core::game::EntityId;
use crate::core::game::HealthSnapshot;
use crate::core::game::ManaSnapshot;
use crate::core::game::NpcDefinitionId;
use crate::core::game::ObservedEffect;
use crate::core::game::ObservedList;
use crate::core::game::ServerBattle;
use crate::core::game::SkillId;
use crate::core::game::StaminaSnapshot;
use crate::core::game::UnitIdentity;
use crate::core::game::UnitSnapshot;
use crate::integrations::aion2::catalog::RuntimeCatalog;
use std::collections::BTreeMap;

pub(super) struct UnitState {
    identity: UnitIdentity,
    owner_id: Option<EntityId>,
    current_health: Option<u32>,
    maximum_health: Option<u32>,
    current_mana: Option<u32>,
    maximum_mana: Option<u32>,
    stamina: StaminaSnapshot,
    health_restored: bool,
    mana_restored: bool,
    health_maximum_restored: bool,
    mana_maximum_restored: bool,
    health_observed_at_us: u64,
    recovered_spawn: bool,
    observed_health: HealthSnapshot,
    observed_mana: ManaSnapshot,
    action: Option<ActionSnapshot>,
    cast: Option<CastPreparation>,
    server_battle: Option<ServerBattle>,
    effects: BTreeMap<EffectInstanceId, EffectState>,
    effects_complete: bool,
    effects_observed: bool,
    present: bool,
    npc_id: Option<NpcDefinitionId>,
}

impl UnitState {
    const MAX_EFFECTS: usize = 1024;

    pub(super) fn new() -> Self {
        Self {
            identity: UnitIdentity::default(),
            owner_id: None,
            current_health: None,
            maximum_health: None,
            current_mana: None,
            maximum_mana: None,
            stamina: StaminaSnapshot::default(),
            health_restored: false,
            mana_restored: false,
            health_maximum_restored: false,
            mana_maximum_restored: false,
            health_observed_at_us: 0,
            recovered_spawn: false,
            observed_health: HealthSnapshot::default(),
            observed_mana: ManaSnapshot::default(),
            action: None,
            cast: None,
            server_battle: None,
            effects: BTreeMap::new(),
            effects_complete: false,
            effects_observed: false,
            present: true,
            npc_id: None,
        }
    }

    pub(super) fn spawn(
        &mut self,
        npc_id: Option<NpcDefinitionId>,
        health: Option<&HealthSnapshot>,
        at_us: u64,
    ) {
        if self.recovered_spawn
            || !self.present
            || self.npc_id.zip(npc_id).is_some_and(|(old, new)| old != new)
        {
            *self = Self::new();
        }

        self.npc_id = npc_id.or(self.npc_id);
        self.present = true;

        if let Some(health) = health {
            self.health(health.current(), health.maximum(), at_us);
        }
    }

    pub(super) fn identity(
        &mut self,
        name: &Option<String>,
        class: Option<CharacterClass>,
        server: Option<u16>,
        level: Option<u32>,
    ) {
        if self.recovered_spawn
            || !self.present
            || self
                .identity
                .name()
                .zip(name.as_deref())
                .is_some_and(|(old, new)| old != new)
            || self
                .identity
                .server_id()
                .zip(server)
                .is_some_and(|(old, new)| old != new)
        {
            *self = Self::new();
        }

        self.identity.observe(name, class, server, level);
        self.present = true;
    }

    pub(super) fn identify_character(&mut self, id: CharacterId, level: u32) {
        if self.npc_id.is_some()
            || self.owner_id.is_some()
            || self
                .identity
                .character_id()
                .is_some_and(|previous| previous != id)
        {
            let name = self.identity.name().map(str::to_owned);

            *self = Self::new();
            self.identity.observe(&name, None, None, None);
        }

        self.identity.identify_character(id);
        self.identity.observe(&None, None, None, Some(level));
    }

    pub(super) fn identity_info(&self) -> &UnitIdentity {
        &self.identity
    }

    pub(super) fn npc_id(&self) -> Option<NpcDefinitionId> {
        self.npc_id
    }

    pub(super) fn present(&self) -> bool {
        self.present
    }

    pub(super) fn owner_id(&self) -> Option<EntityId> {
        self.owner_id
    }

    pub(super) fn action(&mut self, action: ActionSnapshot) {
        self.finish_cast(action.skill_id());
        self.action = Some(action);

        if self.current_health == Some(0) {
            self.current_health = None;
        }

        self.present = true;
    }

    pub(super) fn finish_cast(&mut self, skill_id: SkillId) {
        if self
            .cast
            .as_ref()
            .is_some_and(|cast| cast.skill_id() == skill_id)
        {
            self.cast = None;
        }
    }

    pub(super) fn begin_cast(&mut self, preparation: CastPreparation) {
        self.cast = Some(preparation);
    }

    pub(super) fn observe_class(&mut self, catalog: &RuntimeCatalog) {
        // Own action metadata fills a missing class without replacing server identity.
        if self.identity.class_name().is_some() {
            return;
        }

        let class = self
            .action
            .as_ref()
            .and_then(|action| catalog.skill(action.skill_id().get()))
            .and_then(|skill| skill.class_name());

        self.identity.observe(&None, class, None, None);
    }

    pub(super) fn mana(&mut self, mana: &ManaSnapshot) {
        self.current_mana = mana.current().or(self.current_mana);
        self.maximum_mana = mana.maximum().or(self.maximum_mana);

        if mana.current().is_some() {
            self.mana_restored = false;
        }

        if mana.maximum().is_some() {
            self.mana_maximum_restored = false;
        }

        if self.mana_maximum_restored
            && self
                .current_mana
                .zip(self.maximum_mana)
                .is_some_and(|(value, max)| value > max)
        {
            self.maximum_mana = None;
        }
    }

    pub(super) fn stamina(&mut self, stamina: &StaminaSnapshot) {
        self.stamina.observe(stamina);
    }

    pub(super) fn restore_npc(&mut self, recovery: &NpcRecovery) {
        self.npc_id = Some(recovery.npc_id());
        self.recovered_spawn = true;
        self.restore_resources(recovery.health(), &ManaSnapshot::default());
    }

    pub(super) fn restore_resources(&mut self, health: &HealthSnapshot, mana: &ManaSnapshot) {
        // Recovery can run after a fresh packet created this unit. Never replace live values.
        if self.current_health.is_none() {
            self.current_health = health.current();
            self.health_restored = self.current_health.is_some();
            self.health_observed_at_us = health.observed_at_us();
        }

        if self.current_mana.is_none() {
            self.current_mana = mana.current();
            self.mana_restored = self.current_mana.is_some();
        }

        if self.maximum_health.is_none() {
            self.maximum_health = health.maximum();
            self.health_maximum_restored = self.maximum_health.is_some();
        }

        if self.maximum_mana.is_none() {
            self.maximum_mana = mana.maximum();
            self.mana_maximum_restored = self.maximum_mana.is_some();
        }
    }

    pub(super) fn observe_resources(
        &mut self,
        health: &HealthSnapshot,
        mana: &ManaSnapshot,
    ) -> bool {
        let changed = health
            .current()
            .is_some_and(|value| Some(value) != self.observed_health.current())
            || health
                .maximum()
                .is_some_and(|value| Some(value) != self.observed_health.maximum())
            || mana
                .current()
                .is_some_and(|value| Some(value) != self.observed_mana.current())
            || mana
                .maximum()
                .is_some_and(|value| Some(value) != self.observed_mana.maximum());

        self.observed_health = HealthSnapshot::new(
            health.current().or(self.observed_health.current()),
            health.maximum().or(self.observed_health.maximum()),
        );
        self.observed_mana = ManaSnapshot::new(
            mana.current().or(self.observed_mana.current()),
            mana.maximum().or(self.observed_mana.maximum()),
        );

        changed
    }

    pub(super) fn owner(&mut self, owner: EntityId) {
        self.owner_id = Some(owner);
    }

    pub(super) fn server_battle(&mut self, battle: ServerBattle) {
        self.server_battle = Some(battle);
    }

    pub(super) fn travel(&mut self) {
        self.stamina = StaminaSnapshot::new(None, self.stamina.maximum());
        self.observed_health = HealthSnapshot::default();
        self.observed_mana = ManaSnapshot::default();
        self.server_battle = None;
        self.cast = None;
        self.effects.clear();
        self.effects_complete = false;
        self.effects_observed = false;
        self.current_health = None;
        self.maximum_health = None;
        self.current_mana = None;
        self.maximum_mana = None;
        self.health_restored = false;
        self.mana_restored = false;
        self.health_maximum_restored = false;
        self.mana_maximum_restored = false;
        self.health_observed_at_us = 0;
        self.recovered_spawn = false;
        self.action = None;
    }

    pub(super) fn current_health(&self) -> Option<u32> {
        self.current_health
    }

    pub(super) fn health(&mut self, current: Option<u32>, maximum: Option<u32>, at_us: u64) {
        self.current_health = current.or(self.current_health);
        self.maximum_health = maximum.or(self.maximum_health);

        if current.is_some() {
            self.health_restored = false;
            self.health_observed_at_us = at_us;
        }

        if maximum.is_some() {
            self.health_maximum_restored = false;
        }

        if self.health_maximum_restored
            && self
                .current_health
                .zip(self.maximum_health)
                .is_some_and(|(value, max)| value > max)
        {
            self.maximum_health = None;
        }

        if current == Some(0) {
            self.server_battle = None;
            self.cast = None;
        }
    }

    pub(super) fn put_effect(
        &mut self,
        effect: ObservedEffect,
        at_us: u64,
        restarted: bool,
        stack_limit: Option<usize>,
    ) -> Result<(), String> {
        let id = effect.instance_id();

        if self.effects.len() >= Self::MAX_EFFECTS && !self.effects.contains_key(&id) {
            return Err("Effect instance limit".into());
        }

        if !self.effects.contains_key(&id)
            && let Some(limit) = stack_limit
        {
            let mut matching: Vec<_> = self
                .effects
                .iter()
                .filter(|(_, current)| current.effect().effect_id() == effect.effect_id())
                .map(|(id, current)| (*id, current.first_seen_us()))
                .collect();

            matching.sort_by_key(|(id, start)| (*start, *id));

            for (id, _) in matching
                .iter()
                .take((matching.len() + 1).saturating_sub(limit))
            {
                self.effects.remove(id);
            }
        }

        let changed = self.effects.get(&id).is_some_and(|current| {
            current.effect().effect_id() != effect.effect_id()
                || current.effect().source_id() != effect.source_id()
        });

        let first_seen = if restarted || changed {
            at_us
        } else {
            self.effects
                .get(&id)
                .map_or(at_us, EffectState::first_seen_us)
        };

        self.effects
            .insert(id, EffectState::new(effect, first_seen));
        self.effects_observed = true;

        Ok(())
    }

    pub(super) fn list_effects(
        &mut self,
        effects: &[ObservedEffect],
        at_us: u64,
    ) -> Result<(), String> {
        self.effects
            .retain(|id, _| effects.iter().any(|effect| effect.instance_id() == *id));

        for effect in effects {
            self.put_effect(effect.clone(), at_us, false, None)?;
        }

        self.effects_complete = true;
        self.effects_observed = true;

        Ok(())
    }

    pub(super) fn effect(&self, instance: EffectInstanceId) -> Option<&ObservedEffect> {
        self.effects.get(&instance).map(EffectState::effect)
    }

    pub(super) fn remove_effect(&mut self, instance_id: EffectInstanceId) {
        self.effects.remove(&instance_id);
        self.effects_observed = true;
    }

    pub(super) fn despawn(&mut self) {
        self.stamina = StaminaSnapshot::default();
        self.observed_health = HealthSnapshot::default();
        self.observed_mana = ManaSnapshot::default();
        self.present = false;
        self.server_battle = None;
        self.cast = None;
        self.effects.clear();
        self.effects_complete = true;
    }

    pub(super) fn snapshot(
        &self,
        id: EntityId,
        clock: &ServerClock,
        now_us: u64,
        catalog: &RuntimeCatalog,
    ) -> UnitSnapshot {
        let effects = self
            .effects
            .values()
            .filter_map(|effect| effect.snapshot(clock, now_us))
            .map(|effect| {
                let metadata = catalog.effect(effect.effect().effect_id().get());

                effect
                    .with_disposition(metadata.map(|metadata| metadata.disposition()))
                    .with_display(metadata.map(|metadata| metadata.display()))
                    .with_healing_rows(metadata.map_or(&[], |metadata| metadata.healing_rows()))
            })
            .collect();

        let observed = if self.effects_complete {
            ObservedList::Complete(effects)
        } else if self.effects_observed {
            ObservedList::Partial(effects)
        } else {
            ObservedList::Unknown
        };

        let traits = self.npc_id.and_then(|id| catalog.npc(id.get()));

        UnitSnapshot::new(
            id,
            self.identity.clone(),
            HealthSnapshot::new(self.current_health, self.maximum_health)
                .with_restored(self.health_restored)
                .with_observed_at(self.health_observed_at_us),
            observed,
            self.present,
            self.npc_id,
        )
        .with_cast(self.cast.clone())
        .with_stamina(self.stamina.clone())
        .with_server_battle(self.server_battle.clone())
        .with_activity(
            ManaSnapshot::new(self.current_mana, self.maximum_mana)
                .with_restored(self.mana_restored),
            self.action.clone(),
        )
        .with_traits(
            self.owner_id,
            traits.map(|item| item.boss()),
            traits.map(|item| item.player_summon()),
        )
    }
}
