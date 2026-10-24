use super::EffectInstanceId;
use super::EntityId;
use super::GameEventType;
use super::HealthSnapshot;
use super::MapId;
use super::NpcDefinitionId;
use super::ObservedEffect;
use super::SkillId;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum GameEvent {
    TargetSelected {
        entity_id: Option<EntityId>,
    },
    NpcDialogOpened {
        entity_id: EntityId,
    },
    NpcDialogClosed {
        entity_id: EntityId,
    },
    SelfRestoration {
        entity_id: EntityId,
        skill_id: SkillId,
        value: u32,
        hit: super::HitDetails,
    },
    CharacterIdentified {
        entity_id: EntityId,
        character_id: super::CharacterId,
        level: u32,
    },
    CastStarted {
        source_id: EntityId,
        preparation: super::CastPreparation,
    },
    OdyleEnergy {
        energy: super::OdyleEnergy,
    },
    InstanceTimer {
        timer: super::InstanceTimer,
    },
    WorldBossList {
        list: super::WorldBossList,
    },
    HitLanded {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: SkillId,
    },
    ServerBattleChanged {
        entity_id: EntityId,
        battle: super::ServerBattle,
    },

    CombatStarted {
        entity_id: EntityId,
    },
    CombatEnded {
        entity_id: EntityId,
    },
    EncounterStarted {
        entity_id: EntityId,
        npc_id: NpcDefinitionId,
    },
    EncounterEnded {
        entity_id: EntityId,
        npc_id: NpcDefinitionId,
    },
    PropertiesChanged {
        entity_id: EntityId,
        health: HealthSnapshot,
        mana: super::ManaSnapshot,
        stamina: super::StaminaSnapshot,
    },
    SkillConfirmed {
        skill_id: SkillId,
    },
    CooldownChanged {
        group_id: super::CooldownGroupId,
        remaining_ms: u32,
    },
    Party {
        roster: super::PartyRoster,
    },
    Damage {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: SkillId,
        value: u32,
        hit: super::HitDetails,
    },
    HealingPool {
        entity_id: EntityId,
        source_id: EntityId,
        instance_id: super::EffectInstanceId,
        skill_id: SkillId,
        effect_row: u8,
        remaining: u32,
    },
    Healing {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: SkillId,
        value: u32,
        hit: Option<super::HitDetails>,
    },
    NonValueHit {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: SkillId,
        result_code: u64,
    },
    DamageUnknown {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: Option<SkillId>,
    },
    Miss {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: SkillId,
    },
    Resist {
        entity_id: EntityId,
        source_id: EntityId,
        skill_id: SkillId,
        effect_ids: Vec<super::EffectId>,
    },

    Clock {
        server_ms: u64,
    },
    MapChanged {
        map_id: MapId,
    },
    SelfIdentified {
        entity_id: EntityId,
        authoritative: bool,
    },
    Identity {
        entity_id: EntityId,
        name: Option<String>,
        own: bool,
        server_id: Option<u16>,
        class_name: Option<super::CharacterClass>,
        level: Option<u32>,
    },
    SkillUsed {
        entity_id: Option<EntityId>,
        source_id: EntityId,
        skill_id: SkillId,
    },
    EffectApplied {
        entity_id: EntityId,
        effect: ObservedEffect,
        restarted: bool,
    },
    EffectUpdated {
        entity_id: EntityId,
        effect: ObservedEffect,
        restarted: bool,
    },
    EffectRemoved {
        entity_id: EntityId,
        instance_id: EffectInstanceId,
        reason: u8,
        removed_by: Option<EntityId>,
        removal_skill: Option<SkillId>,
        effect: Option<ObservedEffect>,
    },
    EffectsListed {
        entity_id: EntityId,
        effects: Vec<ObservedEffect>,
    },
    Spawned {
        entity_id: EntityId,
        npc_id: Option<NpcDefinitionId>,
        spawn_kind: u8,
        health: Option<HealthSnapshot>,
    },
    Died {
        entity_id: EntityId,
    },
    Despawned {
        entity_id: EntityId,
    },
    Attribution {
        entity_id: EntityId,
        actor_id: EntityId,
    },
    ComboAvailable {
        previous_skill_id: SkillId,
        skill_id: SkillId,
    },
    ComboUnavailable {
        skill_id: SkillId,
    },
    StatsChanged {
        entity_id: Option<EntityId>,
        values: BTreeMap<u16, i32>,
    },
    SkillsLearned {
        skills: Vec<super::LearnedSkill>,
    },
    SkillLevelChanged {
        skill: super::LearnedSkill,
    },
    SkillFormsListed {
        skills: Vec<super::SkillForms>,
    },
}

impl GameEvent {
    pub(crate) fn kind(&self) -> super::ObservedEventKind {
        match self {
            Self::SelfRestoration { .. } => super::ObservedEventKind::SelfRestoration,
            Self::OdyleEnergy { .. } => super::ObservedEventKind::OdyleEnergy,
            Self::InstanceTimer { .. } => super::ObservedEventKind::InstanceTimer,
            Self::WorldBossList { .. } => super::ObservedEventKind::WorldBossList,
            Self::CharacterIdentified { .. } => super::ObservedEventKind::CharacterIdentified,
            Self::CastStarted { .. } => super::ObservedEventKind::CastStarted,
            Self::HitLanded { .. } => super::ObservedEventKind::HitLanded,
            Self::ServerBattleChanged { .. } => super::ObservedEventKind::ServerBattleChanged,
            Self::CombatStarted { .. } => super::ObservedEventKind::CombatStarted,
            Self::CombatEnded { .. } => super::ObservedEventKind::CombatEnded,
            Self::EncounterStarted { .. } => super::ObservedEventKind::EncounterStarted,
            Self::EncounterEnded { .. } => super::ObservedEventKind::EncounterEnded,
            Self::PropertiesChanged { .. } => super::ObservedEventKind::PropertiesChanged,
            Self::SkillConfirmed { .. } => super::ObservedEventKind::SkillConfirmed,
            Self::CooldownChanged { .. } => super::ObservedEventKind::CooldownChanged,
            Self::Party { .. } => super::ObservedEventKind::Party,
            Self::Damage { .. } => super::ObservedEventKind::Damage,
            Self::Healing { .. } => super::ObservedEventKind::Healing,
            Self::HealingPool { .. } => super::ObservedEventKind::HealingPool,
            Self::TargetSelected { .. } => super::ObservedEventKind::TargetSelected,
            Self::NpcDialogOpened { .. } => super::ObservedEventKind::NpcDialogOpened,
            Self::NpcDialogClosed { .. } => super::ObservedEventKind::NpcDialogClosed,
            Self::DamageUnknown { .. } => super::ObservedEventKind::DamageUnknown,
            Self::NonValueHit { .. } => super::ObservedEventKind::NonValueHit,
            Self::Miss { .. } => super::ObservedEventKind::Miss,
            Self::Resist { .. } => super::ObservedEventKind::Resist,
            Self::Clock { .. } => super::ObservedEventKind::Clock,
            Self::MapChanged { .. } => super::ObservedEventKind::MapChanged,
            Self::SelfIdentified { .. } => super::ObservedEventKind::SelfIdentified,
            Self::Identity { .. } => super::ObservedEventKind::Identity,
            Self::SkillUsed { .. } => super::ObservedEventKind::SkillUsed,
            Self::EffectApplied { .. } => super::ObservedEventKind::EffectApplied,
            Self::EffectUpdated { .. } => super::ObservedEventKind::EffectUpdated,
            Self::EffectRemoved { .. } => super::ObservedEventKind::EffectRemoved,
            Self::EffectsListed { .. } => super::ObservedEventKind::EffectsListed,
            Self::Spawned { .. } => super::ObservedEventKind::Spawned,
            Self::Died { .. } => super::ObservedEventKind::Died,
            Self::Despawned { .. } => super::ObservedEventKind::Despawned,
            Self::Attribution { .. } => super::ObservedEventKind::Attribution,
            Self::ComboAvailable { .. } => super::ObservedEventKind::ComboAvailable,
            Self::ComboUnavailable { .. } => super::ObservedEventKind::ComboUnavailable,
            Self::StatsChanged { .. } => super::ObservedEventKind::StatsChanged,
            Self::SkillsLearned { .. } => super::ObservedEventKind::SkillsLearned,
            Self::SkillLevelChanged { .. } => super::ObservedEventKind::SkillLevelChanged,
            Self::SkillFormsListed { .. } => super::ObservedEventKind::SkillFormsListed,
        }
    }

    pub(crate) fn cast_preparation(&self) -> Option<&super::CastPreparation> {
        match self {
            Self::CastStarted { preparation, .. } => Some(preparation),
            _ => None,
        }
    }

    pub(crate) fn event_type(&self) -> Option<GameEventType> {
        Some(match self {
            Self::CastStarted { .. } => GameEventType::CastStarted,
            Self::HitLanded { .. } => GameEventType::HitLanded,
            Self::ServerBattleChanged { .. } => GameEventType::ServerBattleChanged,
            Self::Damage { .. } => GameEventType::Damage,
            Self::Healing { .. } => GameEventType::Healing,
            Self::Miss { .. } => GameEventType::Miss,
            Self::Resist { .. } => GameEventType::Resist,
            Self::CombatStarted { .. } => GameEventType::CombatStarted,
            Self::CombatEnded { .. } => GameEventType::CombatEnded,
            Self::EncounterStarted { .. } => GameEventType::EncounterStarted,
            Self::EncounterEnded { .. } => GameEventType::EncounterEnded,
            Self::SkillUsed { .. } => GameEventType::SkillUsed,
            Self::EffectApplied { .. } => GameEventType::EffectApplied,
            Self::EffectUpdated { .. } => GameEventType::EffectUpdated,
            Self::EffectRemoved { .. } => GameEventType::EffectRemoved,
            Self::Died { .. } => GameEventType::Death,
            Self::Despawned { .. } => GameEventType::Despawn,
            Self::MapChanged { .. } => GameEventType::MapChanged,
            Self::PropertiesChanged { health, .. } if health.current().is_some() => {
                GameEventType::HealthChanged
            }

            Self::ComboAvailable { .. } => GameEventType::ComboAvailable,
            Self::ComboUnavailable { .. } => GameEventType::ComboUnavailable,
            _ => return None,
        })
    }

    pub(crate) fn recipient(&self) -> Option<EntityId> {
        match self {
            Self::CastStarted { preparation, .. } => preparation.target_id(),
            Self::SkillUsed { entity_id, .. } => *entity_id,
            Self::CombatStarted { entity_id, .. }
            | Self::CombatEnded { entity_id, .. }
            | Self::EncounterStarted { entity_id, .. }
            | Self::EncounterEnded { entity_id, .. }
            | Self::PropertiesChanged { entity_id, .. }
            | Self::Damage { entity_id, .. }
            | Self::SelfRestoration { entity_id, .. }
            | Self::Healing { entity_id, .. }
            | Self::DamageUnknown { entity_id, .. }
            | Self::HitLanded { entity_id, .. }
            | Self::ServerBattleChanged { entity_id, .. }
            | Self::NonValueHit { entity_id, .. }
            | Self::Miss { entity_id, .. }
            | Self::Resist { entity_id, .. }
            | Self::EffectApplied { entity_id, .. }
            | Self::EffectUpdated { entity_id, .. }
            | Self::EffectRemoved { entity_id, .. }
            | Self::Died { entity_id, .. }
            | Self::Despawned { entity_id, .. } => Some(*entity_id),
            _ => None,
        }
    }

    pub(crate) fn source(&self) -> Option<EntityId> {
        match self {
            Self::SelfRestoration { entity_id, .. } => Some(*entity_id),
            Self::CombatStarted { entity_id, .. }
            | Self::CombatEnded { entity_id, .. }
            | Self::EncounterStarted { entity_id, .. }
            | Self::ServerBattleChanged { entity_id, .. }
            | Self::EncounterEnded { entity_id, .. } => Some(*entity_id),
            Self::CastStarted { source_id, .. }
            | Self::Damage { source_id, .. }
            | Self::Healing { source_id, .. }
            | Self::DamageUnknown { source_id, .. }
            | Self::HitLanded { source_id, .. }
            | Self::NonValueHit { source_id, .. }
            | Self::Miss { source_id, .. }
            | Self::Resist { source_id, .. }
            | Self::SkillUsed { source_id, .. } => Some(*source_id),
            Self::EffectApplied { effect, .. } | Self::EffectUpdated { effect, .. } => {
                effect.source_id()
            }

            Self::EffectRemoved {
                effect, removed_by, ..
            } => removed_by.or_else(|| effect.as_ref().and_then(ObservedEffect::source_id)),

            _ => None,
        }
    }

    pub(crate) fn skill_id(&self) -> Option<SkillId> {
        match self {
            Self::EffectRemoved {
                effect,
                removal_skill,
                ..
            } => removal_skill.or_else(|| effect.as_ref().and_then(ObservedEffect::skill_id)),
            Self::CastStarted { preparation, .. } => Some(preparation.skill_id()),
            Self::DamageUnknown { skill_id, .. } => *skill_id,
            Self::SkillConfirmed { skill_id }
            | Self::SelfRestoration { skill_id, .. }
            | Self::Damage { skill_id, .. }
            | Self::Healing { skill_id, .. }
            | Self::Miss { skill_id, .. }
            | Self::Resist { skill_id, .. }
            | Self::SkillUsed { skill_id, .. }
            | Self::HitLanded { skill_id, .. }
            | Self::NonValueHit { skill_id, .. }
            | Self::ComboAvailable { skill_id, .. }
            | Self::ComboUnavailable { skill_id } => Some(*skill_id),
            Self::EffectApplied { effect, .. } | Self::EffectUpdated { effect, .. } => {
                effect.skill_id()
            }

            _ => None,
        }
    }

    pub(crate) fn with_removed_effect(mut self, observed: Option<ObservedEffect>) -> Self {
        if let Self::EffectRemoved { effect, .. } = &mut self {
            *effect = observed;
        }

        self
    }

    pub(crate) fn instance_id(&self) -> Option<super::EffectInstanceId> {
        match self {
            Self::EffectRemoved { instance_id, .. } => Some(*instance_id),
            Self::EffectApplied { effect, .. } | Self::EffectUpdated { effect, .. } => {
                Some(effect.instance_id())
            }

            _ => None,
        }
    }

    pub(crate) fn value(&self) -> Option<u32> {
        match self {
            Self::Damage { value, .. }
            | Self::Healing { value, .. }
            | Self::SelfRestoration { value, .. } => Some(*value),
            Self::PropertiesChanged { health, .. } => health.current(),
            Self::Miss { .. } | Self::Resist { .. } => Some(0),
            _ => None,
        }
    }

    pub(crate) fn outcome(&self) -> Option<super::HitOutcome> {
        match self {
            Self::Miss { .. } => Some(super::HitOutcome::Miss),
            Self::Resist { .. } => Some(super::HitOutcome::Resist),
            _ => self.hit().and_then(super::HitDetails::outcome),
        }
    }

    pub(crate) fn hit(&self) -> Option<&super::HitDetails> {
        match self {
            Self::Damage { hit, .. } | Self::SelfRestoration { hit, .. } => Some(hit),
            Self::Healing { hit, .. } => hit.as_ref(),
            _ => None,
        }
    }

    pub(crate) fn effect_id(&self) -> Option<super::EffectId> {
        match self {
            Self::EffectRemoved { effect, .. } => effect.as_ref().map(ObservedEffect::effect_id),
            Self::EffectApplied { effect, .. } | Self::EffectUpdated { effect, .. } => {
                Some(effect.effect_id())
            }

            _ => None,
        }
    }
}
