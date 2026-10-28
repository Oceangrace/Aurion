use super::BinaryReader;
use super::periodic_kind::PeriodicKind;
use crate::core::game::EffectInstanceId;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::HitDetails;
use crate::core::game::SkillId;
use crate::integrations::aion2::catalog::RuntimeCatalog;

pub(super) struct PeriodicDecoder;

impl PeriodicDecoder {
    pub(super) fn decode(
        reader: &mut BinaryReader<'_>,
        catalog: &RuntimeCatalog,
    ) -> Result<Vec<GameEvent>, String> {
        let entity_id = EntityId::new(reader.varint()?)?;

        let Some(kind) = PeriodicKind::parse(reader.byte()?) else {
            return Ok(Vec::new());
        };

        let source_id = EntityId::new(reader.varint()?)?;

        let instance = reader.varint()?;

        // This field stores a SkillEffect identifier; its final two digits identify the effect row.
        let effect = reader.u32()?;

        if catalog.non_health_skill_effect(effect) {
            return Ok(Vec::new());
        }

        let effect_skill = effect / 100;

        let effect_row = (effect % 100) as u8;

        let mut value = reader.varint()?;

        if kind == PeriodicKind::HealingPool {
            return Self::pool(reader, entity_id, source_id, instance, effect_row, value);
        }

        let mut pool_remaining = None;

        // A periodic heal tick carries the remaining pool here; the healed amount follows.
        if kind == PeriodicKind::PeriodicHealing {
            pool_remaining = u32::try_from(value).ok();
            value = reader.varint()?;

            if reader.remaining() != 4 {
                return Err("Unknown periodic heal suffix".into());
            }
        }

        if !(1..=99_999_999).contains(&value) {
            return Err("Invalid periodic value".into());
        }

        let skill_id = SkillId::new(Self::skill(reader, effect_skill)?)?;

        let healing = kind.healing()
            || (matches!(kind, PeriodicKind::Damage | PeriodicKind::EffectDamage)
                && catalog
                    .skill(skill_id.get())
                    .is_some_and(|skill| skill.healing_only()));

        if catalog
            .skill(skill_id.get())
            .is_some_and(|skill| skill.mana_only())
            || (!healing && source_id == entity_id)
        {
            return Ok(Vec::new());
        }

        let value = value as u32;

        let hit =
            HitDetails::new(None, Vec::new(), kind.periodic(), None).with_skill_effect(effect);

        let hit = match pool_remaining {
            Some(remaining) => hit.with_pool_remaining(remaining, effect_row),
            None => hit,
        };

        Ok(vec![if healing {
            GameEvent::Healing {
                entity_id,
                source_id,
                skill_id,
                value,
                hit: Some(hit),
            }
        } else {
            GameEvent::Damage {
                entity_id,
                source_id,
                skill_id,
                value,
                hit,
            }
        }])
    }

    /// Kind 9 also repeats for NPC effects that never tick; consumers confirm it by the first tick.
    fn pool(
        reader: &mut BinaryReader<'_>,
        entity_id: EntityId,
        source_id: EntityId,
        instance: u64,
        effect_row: u8,
        remaining: u64,
    ) -> Result<Vec<GameEvent>, String> {
        if reader.remaining() != 4 || remaining > 99_999_999 || instance == 0 {
            return Ok(Vec::new());
        }

        Ok(vec![GameEvent::HealingPool {
            entity_id,
            source_id,
            instance_id: EffectInstanceId::new(instance)?,
            skill_id: SkillId::new(reader.u32()?)?,
            effect_row,
            remaining: remaining as u32,
        }])
    }

    /// Full bodies end with the exact skill form; the SkillEffect field keeps only its family.
    fn skill(reader: &mut BinaryReader<'_>, effect_skill: u32) -> Result<u32, String> {
        if reader.remaining() < 4 {
            return Ok(effect_skill);
        }

        reader.skip(reader.remaining() - 4)?;

        reader.u32()
    }
}
