use super::BinaryReader;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::HealthSnapshot;
use crate::core::game::ManaSnapshot;
use crate::core::game::StaminaSnapshot;

pub(super) struct HealthPairDecoder;

impl HealthPairDecoder {
    pub(super) fn decode(reader: &mut BinaryReader<'_>) -> Result<Vec<GameEvent>, String> {
        let entity_id = EntityId::new(reader.varint()?)?;

        let current = u32::try_from(reader.varint()?).map_err(|_| "Invalid current health")?;

        let maximum = u32::try_from(reader.varint()?).map_err(|_| "Invalid maximum health")?;

        if maximum == 0 || current > maximum {
            return Err("Invalid health pair".into());
        }

        // The remaining fields are not identified as MP by either reference reader.
        Ok(vec![GameEvent::PropertiesChanged {
            entity_id,
            health: HealthSnapshot::new(Some(current), Some(maximum)),
            mana: ManaSnapshot::default(),
            stamina: StaminaSnapshot::default(),
        }])
    }
}
