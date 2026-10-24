use super::EffectSnapshot;
use super::EntityId;
use super::HealthSnapshot;
use super::NpcDefinitionId;
use super::ObservedList;
use super::StaminaSnapshot;
use super::UnitIdentity;
use crate::core::game::CastPreparation;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UnitSnapshot {
    owner_id: Option<EntityId>,
    boss: Option<bool>,
    player_summon: Option<bool>,
    id: EntityId,
    identity: UnitIdentity,
    health: HealthSnapshot,
    mana: super::ManaSnapshot,
    stamina: StaminaSnapshot,
    action: Option<super::ActionSnapshot>,
    cast: Option<CastPreparation>,
    server_battle: Option<super::ServerBattle>,
    effects: ObservedList<EffectSnapshot>,
    present: bool,
    npc_id: Option<NpcDefinitionId>,
}

impl UnitSnapshot {
    pub(crate) fn new(
        id: EntityId,
        identity: UnitIdentity,
        health: HealthSnapshot,
        effects: ObservedList<EffectSnapshot>,
        present: bool,
        npc_id: Option<NpcDefinitionId>,
    ) -> Self {
        Self {
            owner_id: None,
            boss: None,
            player_summon: None,
            id,
            identity,
            health,
            mana: Default::default(),
            stamina: Default::default(),
            action: None,
            cast: None,
            server_battle: None,
            effects,
            present,
            npc_id,
        }
    }

    pub(crate) fn with_cast(mut self, cast: Option<super::CastPreparation>) -> Self {
        self.cast = cast;
        self
    }

    pub(crate) fn cast(&self) -> Option<&super::CastPreparation> {
        self.cast.as_ref()
    }

    pub(crate) fn with_server_battle(mut self, battle: Option<super::ServerBattle>) -> Self {
        self.server_battle = battle;
        self
    }

    pub(crate) fn server_battle(&self) -> Option<&super::ServerBattle> {
        self.server_battle.as_ref()
    }

    pub(crate) fn with_activity(
        mut self,
        mana: super::ManaSnapshot,
        action: Option<super::ActionSnapshot>,
    ) -> Self {
        self.mana = mana;
        self.action = action;
        self
    }

    pub(crate) fn mana(&self) -> &super::ManaSnapshot {
        &self.mana
    }

    pub(crate) fn with_stamina(mut self, stamina: StaminaSnapshot) -> Self {
        self.stamina = stamina;
        self
    }

    pub(crate) fn stamina(&self) -> &StaminaSnapshot {
        &self.stamina
    }

    pub(crate) fn action(&self) -> Option<&super::ActionSnapshot> {
        self.action.as_ref()
    }

    pub(crate) fn with_traits(
        mut self,
        owner: Option<EntityId>,
        boss: Option<bool>,
        summon: Option<bool>,
    ) -> Self {
        self.owner_id = owner;
        self.boss = boss;
        self.player_summon = summon;
        self
    }

    pub(crate) fn owner_id(&self) -> Option<EntityId> {
        self.owner_id
    }

    pub(crate) fn boss(&self) -> Option<bool> {
        self.boss
    }

    pub(crate) fn player_summon(&self) -> Option<bool> {
        self.player_summon
    }

    pub(crate) fn id(&self) -> EntityId {
        self.id
    }

    pub(crate) fn identity(&self) -> &UnitIdentity {
        &self.identity
    }

    pub(crate) fn health(&self) -> &HealthSnapshot {
        &self.health
    }

    pub(crate) fn effects(&self) -> &ObservedList<EffectSnapshot> {
        &self.effects
    }

    pub(crate) fn npc_id(&self) -> Option<NpcDefinitionId> {
        self.npc_id
    }

    pub(crate) fn present(&self) -> bool {
        self.present
    }
}
