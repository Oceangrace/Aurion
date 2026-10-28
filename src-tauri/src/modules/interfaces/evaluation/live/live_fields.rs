use super::selection_resolver::SelectionResolver;
use super::stamina_fields::StaminaFields;
use crate::core::game::GameSnapshot;
use crate::core::game::SourceStatus;
use crate::core::game::TargetObservation;
use crate::core::game::UnitSnapshot;
use crate::integrations::aion2::catalog::CatalogScope;
use crate::integrations::aion2::catalog::ZoneCatalog;
use crate::modules::interfaces::document::WidgetNode;
use crate::modules::interfaces::rules::TriggerSubject;
use serde_json::{Value, json};
use std::collections::HashMap;

pub(super) struct LiveFields;

impl LiveFields {
    /// A target's last skill older than this no longer says whom it is fighting.
    const TARGET_OF_TARGET_US: u64 = 6_000_000;

    pub(super) fn collect(game: &GameSnapshot, zones: &ZoneCatalog) -> HashMap<String, Value> {
        let mut fields = HashMap::new();

        fields.insert("source.status".into(), json!(game.header().status()));

        if !matches!(
            game.header().status(),
            SourceStatus::Live | SourceStatus::Replay
        ) {
            return fields;
        }

        if let Some(player) = game.player_id().and_then(|id| game.entities().get(&id)) {
            Self::unit(&mut fields, "player", player, game.header().at_us());
            StaminaFields::collect(&mut fields, player);

            if let Some(statistics) =
                super::combat_projection::CombatProjection::actor(game, player.id())
            {
                let duration = super::combat_projection::CombatProjection::duration(game);

                super::combat_projection::CombatProjection::fields(
                    &mut fields,
                    "player",
                    &statistics,
                    duration,
                );
                super::combat_projection::CombatProjection::fields(
                    &mut fields,
                    "combat",
                    &statistics,
                    duration,
                );
            }
        }

        match game.target() {
            TargetObservation::Cleared => {
                fields.insert("target.exists".into(), json!(false));
            }

            TargetObservation::Selected(_) | TargetObservation::LastSkillRecipient(_) => {
                fields.insert("target.exists".into(), json!(true));
            }

            TargetObservation::Unknown => {}
        }

        if let Some(target) = game
            .target()
            .entity_id()
            .and_then(|id| game.entities().get(&id))
        {
            Self::unit(&mut fields, "target", target, game.header().at_us());
            Self::target_of_target(&mut fields, game, target);
        }

        Self::party(&mut fields, game);
        super::world_fields::WorldFields::collect(&mut fields, game);

        if let Some(combat) = game.combat() {
            if let Some(active) = combat.active() {
                fields.insert("combat.inCombat".into(), json!(active));
            }

            if let Some(elapsed) = combat.elapsed_ms() {
                fields.insert("combat.elapsed".into(), json!(elapsed));
                fields.insert("combat.duration".into(), json!(elapsed));
            }

            if let Some(encounter) = combat.encounter() {
                fields.insert("encounter.npcId".into(), json!(encounter.npc_id().get()));
                fields.insert("encounter.isBoss".into(), json!(encounter.boss()));
                fields.insert(
                    "encounter.elapsed".into(),
                    json!(
                        game.header()
                            .at_us()
                            .saturating_sub(encounter.started_at_us())
                            / 1000
                    ),
                );

                if let Some(unit) = game.entities().get(&encounter.entity_id()) {
                    Self::unit(&mut fields, "encounter", unit, game.header().at_us());
                }
            }
        }

        if let Some(map_id) = game.map_id() {
            fields.insert("zone.id".into(), json!(map_id.get()));

            if let Some(name) = zones.name(map_id.get()) {
                fields.insert("zone.name".into(), json!(name));
            }
        }

        fields
    }

    pub(super) fn selected_skill(
        fields: &mut HashMap<String, Value>,
        game: &GameSnapshot,
        node: &WidgetNode,
        resolver: &SelectionResolver<'_>,
    ) {
        let selection = match &node.trigger {
            Some(trigger) if trigger.subject != TriggerSubject::SelfPlayer => None,
            Some(trigger) if trigger.selection.scope == CatalogScope::Skills => {
                Some(&trigger.selection)
            }
            _ => node
                .image_selection
                .as_ref()
                .filter(|selection| selection.scope == CatalogScope::Skills),
        };

        if let Some(ids) = selection.and_then(|selection| resolver.ids(selection)) {
            Self::skill(fields, game, &ids, resolver);
        }
    }

    fn skill(
        fields: &mut HashMap<String, Value>,
        game: &GameSnapshot,
        ids: &[u32],
        resolver: &SelectionResolver<'_>,
    ) {
        if !matches!(
            game.header().status(),
            SourceStatus::Live | SourceStatus::Replay
        ) {
            return;
        }

        if let Some(cooldown) = resolver.cooldown(ids, game) {
            fields.insert(
                "skill.cooldown.remaining".into(),
                json!(cooldown.remaining_ms()),
            );
            fields.insert(
                "skill.cooldown.group".into(),
                json!(cooldown.group_id().get()),
            );
            fields.insert("skill.cooldown.basis".into(), json!(cooldown.basis()));

            if let Some(duration) = cooldown.duration_ms() {
                fields.insert("skill.cooldown.duration".into(), json!(duration));
            }

            if let Some(ready) = cooldown.ready() {
                fields.insert("skill.ready".into(), json!(ready));
            }
        }

        let combo = resolver.combo(ids, game);

        if let Some(met) = resolver.shock_condition(ids, game) {
            fields.insert("skill.conditionMet".into(), json!(met));
        }

        if let Some(available) = resolver.combo_available(ids, game) {
            fields.insert("skill.comboAvailable".into(), json!(available));
        }

        if let Some(combo) = combo {
            fields.insert("skill.displayId".into(), json!(combo.skill_id().get()));
        }

        if let Some(learned) = game.skill_book().learned(ids) {
            fields.insert("skill.learned".into(), json!(learned.is_some()));

            if let Some(skill) = learned {
                fields.insert("skill.level".into(), json!(skill.level()));
                fields.insert("skill.baseLevel".into(), json!(skill.base_level()));
            }
        }
    }

    fn party(fields: &mut HashMap<String, Value>, game: &GameSnapshot) {
        let Some(roster) = game.party() else {
            return;
        };

        fields.insert("group.complete".into(), json!(roster.complete()));

        if roster.complete() {
            fields.insert("group.size".into(), json!(roster.members().len()));
            fields.insert("group.active".into(), json!(roster.members().len() > 1));
        } else if roster.members().len() > 1 {
            fields.insert("group.active".into(), json!(true));
        }

        if let Some(dungeon) = roster.dungeon_id() {
            fields.insert("group.dungeonId".into(), json!(dungeon));
        }

        let Some(player) = game.player_id().and_then(|id| game.entities().get(&id)) else {
            return;
        };

        let Some(name) = player.identity().name() else {
            return;
        };

        let Some(server) = player.identity().server_id() else {
            return;
        };

        let others = roster.members().iter().filter(|member| {
            member.identity().name() != Some(name) || member.identity().server_id() != Some(server)
        });

        for (index, member) in others.enumerate() {
            let prefix = format!("party{}", index + 1);

            if let Some(unit) = member.entity_id().and_then(|id| game.entities().get(&id)) {
                Self::unit(fields, &prefix, unit, game.header().at_us());

                if let Some(statistics) =
                    super::combat_projection::CombatProjection::actor(game, unit.id())
                {
                    super::combat_projection::CombatProjection::fields(
                        fields,
                        &prefix,
                        &statistics,
                        super::combat_projection::CombatProjection::duration(game),
                    );
                }
            }

            if let Some(name) = member.identity().name() {
                fields.insert(format!("{prefix}.name"), json!(name));
            }

            if let Some(server) = member.identity().server_id() {
                fields.insert(format!("{prefix}.serverId"), json!(server));
            }

            if let Some(class) = member.identity().class_name() {
                fields.insert(format!("{prefix}.class"), json!(class));
            }

            if let Some(level) = member.identity().level() {
                fields.insert(format!("{prefix}.level"), json!(level));
            }

            fields.insert(format!("{prefix}.gearScore"), json!(member.gear_score()));
            fields.insert(
                format!("{prefix}.combatPower"),
                json!(member.combat_power()),
            );
        }
    }

    /// Whom the target is acting on: the client never sends its real target, so this is the
    /// recipient of its running cast, else of its last skill if that is recent.
    fn target_of_target(
        fields: &mut HashMap<String, Value>,
        game: &GameSnapshot,
        target: &UnitSnapshot,
    ) {
        let now = game.header().at_us();

        let cast = target
            .cast()
            .filter(|cast| cast.remaining_ms(now) > 0.0)
            .and_then(|cast| cast.target_id())
            .map(|id| (id, "cast"));

        let skill = || {
            target
                .action()
                .filter(|action| now.saturating_sub(action.at_us()) <= Self::TARGET_OF_TARGET_US)
                .and_then(|action| action.recipient_id())
                .map(|id| (id, "skill"))
        };

        let Some((id, source)) = cast.or_else(skill).filter(|(id, _)| *id != target.id()) else {
            return;
        };

        fields.insert("targetTarget.exists".into(), json!(true));
        fields.insert("targetTarget.source".into(), json!(source));
        fields.insert(
            "targetTarget.isPlayer".into(),
            json!(game.player_id() == Some(id)),
        );

        if let Some(unit) = game.entities().get(&id) {
            Self::unit(fields, "targetTarget", unit, now);
        } else {
            fields.insert("targetTarget.id".into(), json!(id.get().to_string()));
        }
    }

    pub(super) fn unit(
        fields: &mut HashMap<String, Value>,
        prefix: &str,
        unit: &UnitSnapshot,
        now_us: u64,
    ) {
        if let Some(battle) = unit.server_battle() {
            fields.insert(format!("{prefix}.serverInCombat"), json!(battle.active()));

            if let Some(deadline) = battle.deadline_ms() {
                fields.insert(format!("{prefix}.battleDeadline"), json!(deadline));
                fields.insert(
                    format!("{prefix}.battleRemaining"),
                    json!(deadline.saturating_sub(now_us / 1000)),
                );
            }
        }

        fields.insert(format!("{prefix}.exists"), json!(true));
        fields.insert(format!("{prefix}.present"), json!(unit.present()));
        fields.insert(
            format!("{prefix}.effectsComplete"),
            json!(unit.effects().complete()),
        );

        if let Some(level) = unit.identity().level() {
            fields.insert(format!("{prefix}.level"), json!(level));
        }

        if let Some(server) = unit.identity().server_id() {
            fields.insert(format!("{prefix}.serverId"), json!(server));
        }

        if let Some(npc) = unit.npc_id() {
            fields.insert(format!("{prefix}.npcId"), json!(npc.get()));
        }

        fields.insert(format!("{prefix}.id"), json!(unit.id().get().to_string()));

        if let Some(class) = unit.identity().class_name() {
            fields.insert(format!("{prefix}.class"), json!(class));
        }

        if let Some(name) = unit.identity().name() {
            fields.insert(format!("{prefix}.name"), json!(name));
        }

        if let Some(owner) = unit.owner_id() {
            fields.insert(format!("{prefix}.ownerId"), json!(owner.get().to_string()));
        }

        if let Some(boss) = unit.boss() {
            fields.insert(format!("{prefix}.isBoss"), json!(boss));
        }

        if let Some(summon) = unit.player_summon() {
            fields.insert(format!("{prefix}.isPlayerSummon"), json!(summon));
        }

        if !unit.present() {
            return;
        }

        super::cast_fields::CastFields::collect(fields, prefix, unit, now_us);

        if let Some(action) = unit.action() {
            fields.insert(
                format!("{prefix}.lastSkill.id"),
                json!(action.skill_id().get()),
            );
            fields.insert(
                format!("{prefix}.lastSkill.elapsed"),
                json!(now_us.saturating_sub(action.at_us()) / 1000),
            );

            if let Some(value) = action.casting_ms() {
                fields.insert(format!("{prefix}.lastSkill.baseCasting"), json!(value));
            }

            if let Some(value) = action.cooldown_ms() {
                fields.insert(format!("{prefix}.lastSkill.baseCooldown"), json!(value));
            }

            if let Some(value) = action.cooldown_group() {
                fields.insert(format!("{prefix}.lastSkill.cooldownGroup"), json!(value));
            }

            fields.insert(format!("{prefix}.lastSkill.atUs"), json!(action.at_us()));

            if let Some(recipient) = action.recipient_id() {
                fields.insert(
                    format!("{prefix}.lastSkill.targetId"),
                    json!(recipient.get().to_string()),
                );
            }
        }

        if let Some(value) = unit.mana().current() {
            fields.insert(format!("{prefix}.mp.value"), json!(value));
        }

        for (resource, restored) in [
            ("hp", unit.health().restored()),
            ("mp", unit.mana().restored()),
        ] {
            fields.insert(format!("{prefix}.{resource}.restored"), json!(restored));

            for field in ["value", "max", "percent"] {
                fields.insert(
                    format!("restored.{prefix}.{resource}.{field}"),
                    json!(restored),
                );
            }
        }

        if let Some(maximum) = unit.mana().maximum() {
            fields.insert(format!("{prefix}.mp.max"), json!(maximum));
        }

        if let Some((current, maximum)) = unit
            .mana()
            .current()
            .zip(unit.mana().maximum())
            .filter(|(_, maximum)| *maximum > 0)
        {
            fields.insert(
                format!("{prefix}.mp.percent"),
                json!(f64::from(current) / f64::from(maximum) * 100.0),
            );
        }

        if let Some(hp) = unit.health().current() {
            fields.insert(format!("{prefix}.dead"), json!(hp == 0));
            fields.insert(format!("{prefix}.hp.value"), json!(hp));
        }

        if let Some(maximum) = unit.health().maximum() {
            fields.insert(format!("{prefix}.hp.max"), json!(maximum));
        }

        if let Some((current, maximum)) = unit
            .health()
            .current()
            .zip(unit.health().maximum())
            .filter(|(_, maximum)| *maximum > 0)
        {
            fields.insert(
                format!("{prefix}.hp.percent"),
                json!(f64::from(current) / f64::from(maximum) * 100.0),
            );
        }
    }
}
