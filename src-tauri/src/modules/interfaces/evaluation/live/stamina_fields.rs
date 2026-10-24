use crate::core::game::UnitSnapshot;
use serde_json::{Value, json};
use std::collections::HashMap;

pub(super) struct StaminaFields;

impl StaminaFields {
    pub(super) fn collect(fields: &mut HashMap<String, Value>, player: &UnitSnapshot) {
        if !player.present() {
            return;
        }

        let stamina = player.stamina();

        if let Some(current) = stamina.current() {
            fields.insert("player.stamina.value".into(), json!(current));
        }

        if let Some(maximum) = stamina.maximum() {
            fields.insert("player.stamina.max".into(), json!(maximum));
        }

        if let Some((current, maximum)) = stamina
            .current()
            .zip(stamina.maximum())
            .filter(|(_, maximum)| *maximum > 0)
        {
            fields.insert(
                "player.stamina.percent".into(),
                json!(f64::from(current.min(maximum)) / f64::from(maximum) * 100.0),
            );
        }
    }
}
