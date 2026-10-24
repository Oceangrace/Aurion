use super::binary_reader::BinaryReader;
use super::decoder_kind::DecoderKind;
use super::identity_decoder::IdentityDecoder;
use crate::core::game::GameEvent;
use crate::core::game::HealthSnapshot;
use crate::core::game::ManaSnapshot;
use crate::core::game::StaminaSnapshot;

pub(super) struct PlayerVitalsDecoder;

impl PlayerVitalsDecoder {
    pub(super) fn decode(body: &[u8]) -> Result<Option<GameEvent>, String> {
        let GameEvent::Identity {
            entity_id,
            name: Some(_),
            ..
        } = IdentityDecoder::decode(body, DecoderKind::PlayerSpawn)?
        else {
            return Ok(None);
        };

        let mut reader = BinaryReader::new(body);

        reader.skip(2)?;
        reader.varint()?;
        reader.skip(5)?;

        let name_bytes = reader.byte()?;

        reader.skip(usize::from(name_bytes))?;
        reader.skip(4)?;

        let flags = reader.bytes(4)?;

        if !matches!(flags[0], 1 | 2)
            || !matches!(flags[1], 1 | 2)
            || flags[2] & !0xc8 != 0
            || !matches!(flags[3], 0x12 | 0x16 | 0x52 | 0x92 | 0xd2)
        {
            return Ok(None);
        }

        Self::position(&mut reader)?;
        reader.skip(2)?;

        if flags[2] & 8 != 0 {
            for _ in 0..3 {
                if !f32::from_bits(reader.u32()?).is_finite() {
                    return Err("Invalid player movement vector".into());
                }
            }
        }

        if reader.byte()? != 1 {
            return Ok(None);
        }

        let current = u32::try_from(reader.varint()?).map_err(|_| "Invalid player health")?;

        let maximum =
            u32::try_from(reader.varint()?).map_err(|_| "Invalid player health maximum")?;

        let mana = reader.u32()?;

        let maximum_mana = reader.u32()?;

        if maximum == 0 || current > maximum || mana > maximum_mana {
            return Err("Invalid player vital pair".into());
        }

        Ok(Some(GameEvent::PropertiesChanged {
            entity_id,
            health: HealthSnapshot::new(Some(current), Some(maximum)),
            mana: ManaSnapshot::new(Some(mana), Some(maximum_mana)),
            stamina: StaminaSnapshot::default(),
        }))
    }

    fn position(reader: &mut BinaryReader<'_>) -> Result<(), String> {
        for _ in 0..4 {
            if !f32::from_bits(reader.u32()?).is_finite() {
                return Err("Invalid player appearance position".into());
            }
        }

        Ok(())
    }
}
