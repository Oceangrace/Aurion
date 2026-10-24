use super::BinaryReader;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::HealthSnapshot;
use crate::core::game::ManaSnapshot;
use crate::core::game::StaminaSnapshot;

pub(super) struct PropertyDecoder;

impl PropertyDecoder {
    const MAX_STAMINA: u32 = 100_000_000;

    pub(super) fn decode(reader: &mut BinaryReader<'_>) -> Result<Vec<GameEvent>, String> {
        let entity_id = EntityId::new(reader.varint()?)?;

        let flags = reader.byte()?;

        if flags & !3 != 0 {
            return Err("Unknown entity property flags".into());
        }

        let mut mana = None;

        let mut maximum_mana = None;

        let mut health = None;

        let mut maximum_health = None;

        let mut stamina = None;

        let mut maximum_stamina = None;

        if flags & 1 != 0 {
            let count = reader.byte()?;

            for _ in 0..count {
                let property = reader.byte()?;

                let value = reader.u32()?;

                match property {
                    1 => mana = Some(value),
                    8 => maximum_mana = Some(value),
                    3 => stamina = Self::stamina(value, 0).or(stamina),
                    10 => maximum_stamina = Self::stamina(value, 1).or(maximum_stamina),
                    _ => {}
                }
            }
        }

        if flags & 2 != 0 {
            let count = reader.byte()?;

            for _ in 0..count {
                let property = reader.byte()?;

                let value = reader.i64()?;

                if matches!(property, 0 | 7) {
                    let value = u32::try_from(value).map_err(|_| "Invalid health property")?;

                    match property {
                        0 => health = Some(value),
                        7 => maximum_health = Some(value),
                        _ => {}
                    }
                }
            }
        }

        if reader.remaining() != 0 {
            return Err("Unknown entity property suffix".into());
        }

        Ok(vec![GameEvent::PropertiesChanged {
            entity_id,
            health: HealthSnapshot::new(health, maximum_health),
            mana: ManaSnapshot::new(mana, maximum_mana),
            stamina: StaminaSnapshot::new(stamina, maximum_stamina),
        }])
    }

    /// An implausible stamina value is dropped on its own: HP and MP in the same message stay.
    fn stamina(value: u32, minimum: u32) -> Option<u32> {
        (minimum..=Self::MAX_STAMINA)
            .contains(&value)
            .then_some(value)
    }
}
