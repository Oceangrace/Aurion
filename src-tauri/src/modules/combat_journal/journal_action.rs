use super::journal_action_kind::JournalActionKind;
use crate::core::game::EventRecord;
use crate::core::game::GameEvent;
use crate::integrations::aion2::catalog::RuntimeCatalog;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct JournalAction {
    sequence: u64,
    at_us: u64,
    kind: JournalActionKind,
    source_id: u64,
    recipient_id: Option<u64>,
    skill_id: Option<u32>,
    value: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skill_effect_id: Option<u32>,
}

impl JournalAction {
    pub(super) fn new(record: &EventRecord, catalog: &RuntimeCatalog) -> Option<Self> {
        let (kind, source, recipient, skill, value) = match record.event() {
            GameEvent::Damage {
                entity_id,
                source_id,
                skill_id,
                value,
                ..
            } => (
                JournalActionKind::Damage,
                *source_id,
                Some(*entity_id),
                Some(*skill_id),
                *value,
            ),
            GameEvent::Healing {
                entity_id,
                source_id,
                skill_id,
                value,
                ..
            } => (
                JournalActionKind::Healing,
                *source_id,
                Some(*entity_id),
                Some(*skill_id),
                *value,
            ),
            GameEvent::SkillUsed {
                entity_id,
                source_id,
                skill_id,
            } => (
                if catalog
                    .skill(skill_id.get())
                    .is_some_and(|skill| skill.automatic())
                {
                    JournalActionKind::Trigger
                } else {
                    JournalActionKind::Cast
                },
                *source_id,
                *entity_id,
                Some(*skill_id),
                0,
            ),
            GameEvent::Died { entity_id } => (
                JournalActionKind::Death,
                *entity_id,
                Some(*entity_id),
                None,
                0,
            ),
            GameEvent::CastStarted {
                source_id,
                preparation,
            } => (
                JournalActionKind::CastStart,
                *source_id,
                preparation.target_id(),
                Some(preparation.skill_id()),
                0,
            ),
            GameEvent::Miss {
                entity_id,
                source_id,
                skill_id,
            } => (
                JournalActionKind::Miss,
                *source_id,
                Some(*entity_id),
                Some(*skill_id),
                0,
            ),
            GameEvent::Resist {
                entity_id,
                source_id,
                skill_id,
                ..
            } => (
                JournalActionKind::Resist,
                *source_id,
                Some(*entity_id),
                Some(*skill_id),
                0,
            ),
            _ => return None,
        };

        Some(Self {
            sequence: record.sequence(),
            at_us: record.at_us(),
            kind,
            source_id: source.get(),
            recipient_id: recipient.map(|id| id.get()),
            skill_id: skill.map(|id| id.get()),
            value,
            duration_ms: match record.event() {
                GameEvent::CastStarted { preparation, .. } => Some(preparation.duration_ms()),
                _ => None,
            },
            skill_effect_id: match record.event() {
                GameEvent::Damage { hit, .. } => hit.skill_effect_id(),
                GameEvent::Healing { hit, .. } => {
                    hit.as_ref().and_then(|hit| hit.skill_effect_id())
                }
                _ => None,
            },
        })
    }
}
