use super::live_inputs::LiveInputs;
use super::live_row::LiveRow;
use crate::integrations::aion2::WorldBossObservation;
use crate::integrations::aion2::catalog::FieldBossCatalog;
use crate::modules::interfaces::document::WidgetNode;
use crate::modules::interfaces::evaluation::WorldBossStatus;
use serde_json::json;
use std::collections::HashMap;

pub(super) struct WorldBossRows;

impl WorldBossRows {
    pub(super) fn collect(node: &WidgetNode, inputs: &LiveInputs<'_>) -> Vec<LiveRow> {
        let Some(binding) = &node.binding else {
            return Vec::new();
        };

        let mut rows = binding
            .boss_ids
            .iter()
            .filter_map(|id| Self::row(*id, inputs))
            .collect::<Vec<_>>();

        rows.sort_by_key(|(rank, time, _)| (*rank, *time));

        rows.into_iter().map(|(_, _, row)| row).collect()
    }

    fn row(id: u32, inputs: &LiveInputs<'_>) -> Option<(u8, u64, LiveRow)> {
        let definition = FieldBossCatalog::find(id)?;

        let game = inputs.game();

        let now = game.header().at_us();

        let list = game
            .world()
            .boss_lists()
            .iter()
            .find(|list| list.map_id().get() == definition.map_id);

        let observation = WorldBossObservation::new(list, id, now);

        let remaining = observation.remaining();

        let status = WorldBossStatus::resolve(&observation);

        let name = inputs
            .resolver()
            .npc_name(definition.npc_id, inputs.text().locale())
            .unwrap_or_else(|| format!("#{id}"));

        let fields = HashMap::from([
            ("row.id".into(), json!(id.to_string())),
            ("row.name".into(), json!(name)),
            ("row.status".into(), json!(status.compatible())),
            ("row.state".into(), json!(status)),
            ("row.remaining".into(), json!(remaining)),
            (
                "row.timer".into(),
                json!(
                    remaining
                        .map_or_else(|| inputs.text().world_boss(status).to_owned(), Self::timer)
                ),
            ),
            (
                "row.statusText".into(),
                json!(inputs.text().world_boss(status)),
            ),
            ("row.restored".into(), json!(observation.restored())),
            ("row.observedAge".into(), json!(observation.age_ms())),
            ("row.mapId".into(), json!(definition.map_id)),
        ]);

        Some((
            status.rank(),
            remaining.unwrap_or(0),
            LiveRow::new(id.to_string(), None, fields),
        ))
    }

    fn timer(remaining_ms: u64) -> String {
        let seconds = remaining_ms.div_ceil(1000);

        if seconds >= 3600 {
            return format!(
                "{}:{:02}:{:02}",
                seconds / 3600,
                seconds / 60 % 60,
                seconds % 60
            );
        }

        format!("{}:{:02}", seconds / 60, seconds % 60)
    }
}
