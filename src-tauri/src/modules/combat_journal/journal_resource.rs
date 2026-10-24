use crate::core::game::EventRecord;
use crate::core::game::GameEvent;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct JournalResource {
    sequence: u64,
    at_us: u64,
    unit_id: u64,
    health: Option<u32>,
    health_max: Option<u32>,
    mana: Option<u32>,
    mana_max: Option<u32>,
}

impl JournalResource {
    pub(super) fn new(record: &EventRecord) -> Option<Self> {
        let GameEvent::PropertiesChanged {
            entity_id,
            health,
            mana,
            ..
        } = record.event()
        else {
            return None;
        };

        if health.current().is_none()
            && health.maximum().is_none()
            && mana.current().is_none()
            && mana.maximum().is_none()
        {
            return None;
        }

        Some(Self {
            sequence: record.sequence(),
            at_us: record.at_us(),
            unit_id: entity_id.get(),
            health: health.current(),
            health_max: health.maximum(),
            mana: mana.current(),
            mana_max: mana.maximum(),
        })
    }
}
