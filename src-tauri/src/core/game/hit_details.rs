use super::HitModifier;
use super::HitOutcome;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HitDetails {
    outcome: Option<HitOutcome>,
    modifiers: Vec<HitModifier>,
    periodic: bool,
    restored_hp: Option<u32>,
    power_scalar: Option<u32>,
    additional_hits: u8,
    additional_damage: u32,
    life_steal: Option<u32>,
    protections: Vec<u32>,
    pool_remaining: Option<u32>,
    pool_effect_row: Option<u8>,
    skill_effect_id: Option<u32>,
}

impl HitDetails {
    pub(crate) fn new(
        outcome: Option<HitOutcome>,
        modifiers: Vec<HitModifier>,
        periodic: bool,
        restored_hp: Option<u32>,
    ) -> Self {
        Self {
            outcome,
            modifiers,
            periodic,
            restored_hp,
            power_scalar: None,
            additional_hits: 0,
            additional_damage: 0,
            life_steal: None,
            protections: Vec::new(),
            pool_remaining: None,
            pool_effect_row: None,
            skill_effect_id: None,
        }
    }

    pub(crate) fn amounts(
        mut self,
        scalar: Option<u32>,
        additional: &[u32],
        life_steal: Option<u32>,
        protections: Vec<u32>,
    ) -> Self {
        self.power_scalar = scalar;
        self.additional_hits = u8::try_from(additional.len()).unwrap_or(u8::MAX);
        self.additional_damage = additional
            .iter()
            .fold(0, |sum, hit| sum.saturating_add(*hit));
        self.life_steal = life_steal;
        self.protections = protections;

        self
    }

    pub(crate) fn with_pool_remaining(mut self, remaining: u32, effect_row: u8) -> Self {
        self.pool_remaining = Some(remaining);
        self.pool_effect_row = Some(effect_row);

        self
    }

    pub(crate) fn with_skill_effect(mut self, id: u32) -> Self {
        self.skill_effect_id = Some(id);

        self
    }

    pub(crate) fn skill_effect_id(&self) -> Option<u32> {
        self.skill_effect_id
    }

    pub(crate) fn pool_remaining(&self) -> Option<u32> {
        self.pool_remaining
    }

    pub(crate) fn pool_effect_row(&self) -> Option<u8> {
        self.pool_effect_row
    }

    pub(crate) fn additional_hits(&self) -> u8 {
        self.additional_hits
    }

    pub(crate) fn additional_damage(&self) -> u32 {
        self.additional_damage
    }

    pub(crate) fn restored_hp(&self) -> Option<u32> {
        self.restored_hp
    }

    pub(crate) fn power_scalar(&self) -> Option<u32> {
        self.power_scalar
    }

    pub(crate) fn life_steal(&self) -> Option<u32> {
        self.life_steal
    }

    pub(crate) fn protections(&self) -> &[u32] {
        &self.protections
    }

    pub(crate) fn outcome(&self) -> Option<HitOutcome> {
        self.outcome
    }

    pub(crate) fn periodic(&self) -> bool {
        self.periodic
    }

    pub(crate) fn modifiers(&self) -> &[HitModifier] {
        &self.modifiers
    }
}
