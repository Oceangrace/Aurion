use super::live_inputs::LiveInputs;
use super::trigger_match::TriggerMatch;
use super::trigger_result::TriggerResult;
use super::trigger_state::TriggerState;
use crate::integrations::aion2::WorldBossObservation;
use crate::integrations::aion2::catalog::FieldBossCatalog;
use crate::modules::interfaces::rules::BossEvent;
use crate::modules::interfaces::rules::BossTrigger;
use serde_json::json;
use std::collections::HashMap;

pub(super) struct BossMatcher;

impl BossMatcher {
    pub(super) fn evaluate(trigger: &BossTrigger, inputs: &LiveInputs<'_>) -> TriggerResult {
        let Some(definition) = FieldBossCatalog::find(trigger.boss_id) else {
            return TriggerResult::new(TriggerState::Unsupported, Vec::new());
        };

        let game = inputs.game();

        let now = game.header().at_us() as f64 / 1000.0;

        let list = game
            .world()
            .boss_lists()
            .iter()
            .find(|list| list.map_id().get() == definition.map_id);

        let observation = WorldBossObservation::new(list, trigger.boss_id, game.header().at_us());

        let deadline = observation.respawns_at();

        let known = observation.known();

        let starts = deadline.map(|time| {
            time as f64
                - if trigger.event == BossEvent::BeforeSpawn {
                    trigger.before_seconds * 1000.0
                } else {
                    0.0
                }
        });

        let elapsed = starts.map(|start| now - start);

        let active = match trigger.event {
            BossEvent::Unknown => Some(!known),
            BossEvent::Alive => known.then_some(observation.compatible_alive()),
            BossEvent::BeforeSpawn | BossEvent::Spawned => deadline.map(|_| {
                elapsed.is_some_and(|elapsed| elapsed >= 0.0 && elapsed < trigger.duration_ms)
                    && observation.observed_alive() == Some(false)
            }),
        };

        let mut fields = HashMap::from([
            ("bossId".into(), json!(trigger.boss_id)),
            ("mapId".into(), json!(definition.map_id)),
            (
                "name".into(),
                json!(
                    inputs
                        .resolver()
                        .npc_name(definition.npc_id, inputs.text().locale())
                ),
            ),
            ("known".into(), json!(known)),
            ("alive".into(), json!(observation.compatible_alive())),
            ("observedAlive".into(), json!(observation.observed_alive())),
            ("restored".into(), json!(observation.restored())),
            ("expected".into(), json!(observation.expected())),
            ("respawnsAt".into(), json!(deadline)),
            (
                "countdown".into(),
                json!(deadline.map(|time| (time as f64 - now).max(0.0))),
            ),
            ("duration".into(), json!(trigger.duration_ms)),
            (
                "remaining".into(),
                json!(elapsed.map(|time| (trigger.duration_ms - time.max(0.0)).max(0.0))),
            ),
            ("elapsed".into(), json!(elapsed.map(|time| time.max(0.0)))),
        ]);

        if let Some(deadline) = deadline {
            fields.insert(
                "id".into(),
                json!(format!("{}:{deadline}", trigger.boss_id)),
            );
        }

        TriggerResult::new(
            TriggerState::from_active(active, false),
            vec![TriggerMatch::Boss { fields }],
        )
    }
}
