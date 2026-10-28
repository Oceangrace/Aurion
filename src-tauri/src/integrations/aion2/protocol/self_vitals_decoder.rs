use super::BinaryReader;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::HealthSnapshot;
use crate::core::game::ManaSnapshot;
use crate::core::game::StaminaSnapshot;

/// Current and maximum HP and MP inside 0x3633, sent on every map load. Without it the own HP
/// stays unknown until the first change, because 0x8d00 sends only changed values.
pub(super) struct SelfVitalsDecoder;

impl SelfVitalsDecoder {
    const HEADER_LENGTH: usize = 8;

    /// `body` includes the opcode. `None` when the block is absent or not unique.
    pub(super) fn decode(body: &[u8]) -> Result<Option<GameEvent>, String> {
        let mut starts = body
            .windows(Self::HEADER_LENGTH)
            .enumerate()
            .filter(|(_, window)| {
                matches!(
                    *window,
                    [0x02, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00 | 0x80, 0xd2]
                )
            })
            .map(|(at, _)| at);

        let (Some(start), None) = (starts.next(), starts.next()) else {
            return Ok(None);
        };

        let mut header = BinaryReader::new(body);

        header.skip(2)?;

        let entity_id = EntityId::new(header.varint()?)?;

        let mut reader = BinaryReader::new(body);

        reader.seek(start + Self::HEADER_LENGTH)?;

        for _ in 0..4 {
            if !f32::from_bits(reader.u32()?).is_finite() {
                return Err("Invalid own position".into());
            }
        }

        reader.skip(2)?;

        if reader.byte()? != 1 {
            return Err("Unknown own vitals marker".into());
        }

        let health = Self::pair(reader.varint()?, reader.varint()?)?;

        let mana = Self::pair(u64::from(reader.u32()?), u64::from(reader.u32()?))?;

        Ok(Some(GameEvent::PropertiesChanged {
            entity_id,
            health: HealthSnapshot::new(Some(health.0), Some(health.1)),
            mana: ManaSnapshot::new(Some(mana.0), Some(mana.1)),
            stamina: StaminaSnapshot::default(),
        }))
    }

    fn pair(current: u64, maximum: u64) -> Result<(u32, u32), String> {
        let current = u32::try_from(current).map_err(|_| "Invalid own vital value")?;

        let maximum = u32::try_from(maximum).map_err(|_| "Invalid own vital maximum")?;

        if maximum == 0 || current > maximum {
            return Err("Invalid own vital pair".into());
        }

        Ok((current, maximum))
    }
}
