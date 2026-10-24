use crate::core::game::GameSnapshot;
use crate::integrations::aion2::WorldBossObservation;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;

pub(super) struct WorldFields;

impl WorldFields {
    pub(super) fn collect(fields: &mut HashMap<String, Value>, game: &GameSnapshot) {
        let world = game.world();

        if let Some(energy) = world.energy() {
            fields.insert("player.odyle.basic".into(), json!(energy.basic()));
            fields.insert("player.odyle.charged".into(), json!(energy.charged()));
        }

        let Some(map_id) = game.map_id() else {
            return;
        };

        let now = game.header().at_us() / 1000;

        if let Some((timer, ends)) = world
            .timers()
            .iter()
            .filter(|timer| timer.map_id() == map_id)
            .filter_map(|timer| {
                timer
                    .ends_at_ms()
                    .filter(|ends| *ends > now)
                    .map(|ends| (timer, ends))
            })
            .min_by_key(|(_, ends)| *ends)
        {
            fields.insert("zone.timer.kind".into(), json!(timer.kind()));
            fields.insert("zone.timer.remaining".into(), json!(ends - now));
            fields.insert("zone.timer.endsAt".into(), json!(ends));
            fields.insert("zone.timer.startedAt".into(), json!(timer.started_at_ms()));
        }

        for timer in world
            .timers()
            .iter()
            .filter(|timer| timer.map_id() == map_id)
        {
            let prefix = format!("zone.timer.{}", timer.kind());

            fields.insert(format!("{prefix}.startedAt"), json!(timer.started_at_ms()));

            let Some(ends) = timer.ends_at_ms() else {
                continue;
            };

            fields.insert(format!("{prefix}.endsAt"), json!(ends));
            fields.insert(
                format!("{prefix}.remaining"),
                json!(ends.saturating_sub(now)),
            );
            fields.insert(
                format!("{prefix}.active"),
                json!(timer.started_at_ms() <= now && now < ends),
            );
        }

        let Some(list) = world
            .boss_lists()
            .iter()
            .find(|list| list.map_id() == map_id)
        else {
            return;
        };

        fields.insert(
            "zone.worldBosses.observedAt".into(),
            json!(list.observed_at_us() / 1000),
        );
        fields.insert(
            "zone.worldBosses.aliveCount".into(),
            json!(list.bosses().iter().filter(|boss| boss.alive()).count()),
        );

        if let Some((boss, time)) = list
            .bosses()
            .iter()
            .filter(|boss| !boss.alive())
            .filter_map(|boss| {
                boss.time_ms()
                    .filter(|time| *time > now)
                    .map(|time| (boss, time))
            })
            .min_by_key(|(_, time)| *time)
        {
            fields.insert("zone.nextWorldBoss.id".into(), json!(boss.world_map_id()));
            fields.insert("zone.nextWorldBoss.remaining".into(), json!(time - now));
            fields.insert("zone.nextWorldBoss.respawnsAt".into(), json!(time));
        }

        for boss in list.bosses() {
            let prefix = format!("worldBoss.{}", boss.world_map_id());

            let observation =
                WorldBossObservation::new(Some(list), boss.world_map_id(), game.header().at_us());

            fields.insert(
                format!("{prefix}.observedAlive"),
                json!(observation.observed_alive()),
            );
            fields.insert(
                format!("{prefix}.alive"),
                json!(observation.compatible_alive()),
            );
            fields.insert(format!("{prefix}.expected"), json!(observation.expected()));

            if let Some(time) = boss.time_ms() {
                let name = if boss.alive() {
                    "spawnedAt"
                } else {
                    "respawnsAt"
                };

                fields.insert(format!("{prefix}.{name}"), json!(time));

                if !boss.alive() && time > now {
                    fields.insert(format!("{prefix}.remaining"), json!(time - now));
                }
            }
        }
    }
}
