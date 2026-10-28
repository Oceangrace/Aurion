use super::binding_key::BindingKey;
use super::boss_matcher::BossMatcher;
use super::effect_matcher::EffectMatcher;
use super::event_cache::EventCache;
use super::event_matcher::EventMatcher;
use super::live_frame::LiveFrame;
use super::live_inputs::LiveInputs;
use super::selection_resolver::SelectionResolver;
use super::skill_history::SkillHistory;
use super::trigger_result::TriggerResult;
use super::trigger_state::TriggerState;
use crate::core::game::EntityId;
use crate::core::game::GameEvent;
use crate::core::game::GameEventType;
use crate::core::game::GameSnapshot;
use crate::core::game::SourceFrame;
use crate::core::game::SourceStatus;
use crate::integrations::aion2::GameId;
use crate::integrations::aion2::catalog::CatalogStore;
use crate::integrations::aion2::catalog::ZoneCatalog;
use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::document::WidgetNode;
use crate::modules::interfaces::evaluation::DocumentLocalizer;
use crate::modules::interfaces::evaluation::PreviewContext;
use crate::modules::interfaces::evaluation::PreviewEvaluator;
use crate::modules::interfaces::evaluation::PreviewText;
use crate::modules::interfaces::evaluation::PreviewVisual;
use crate::modules::interfaces::evaluation::RuleEvaluator;
use crate::modules::interfaces::layout::GroupDataSource;
use crate::modules::interfaces::rules::EffectFilter;
use crate::modules::interfaces::rules::EffectOrigin;
use crate::modules::interfaces::rules::EffectPresence;
use crate::modules::interfaces::rules::EffectTrigger;
use crate::modules::interfaces::rules::ObservationKind;
use crate::modules::interfaces::rules::TriggerDefinition;
use crate::modules::interfaces::rules::TriggerRule;
use crate::modules::interfaces::rules::TriggerSubject;
use crate::modules::interfaces::rules::TriggerUnit;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LiveEvaluator {
    events: EventCache,
    skills: SkillHistory,
    healing: super::healing_forecast::HealingForecast,
}

impl LiveEvaluator {
    pub(crate) fn evaluate(
        &mut self,
        workspace: &InterfaceWorkspace,
        source: &SourceFrame,
        catalog: &CatalogStore,
        zones: &ZoneCatalog,
        text: PreviewText,
    ) -> Result<LiveFrame, String> {
        let game = source.snapshot();

        let mut frame = LiveFrame::new(game.header().status(), source.error().map(str::to_owned));

        frame.observe_world(game);
        self.healing.update(game);
        self.events.begin(game);
        self.skills.update(game);

        let frame_inputs = LiveInputs::new(game, catalog, zones, text.clone(), &[]);

        for document in workspace
            .documents
            .iter()
            .filter(|document| document.enabled)
        {
            let localized = DocumentLocalizer::apply(document, text.locale());
            let document = localized.as_ref();

            let inputs = frame_inputs.with_formulas(&document.formulas);

            let mut part = LiveFrame::new(game.header().status(), None);

            let result = document
                .roots
                .iter()
                .try_for_each(|node| self.node(node, &inputs, &mut part))
                .map(|()| part);

            frame.absorb(&document.name, result);
        }

        self.events.finish();

        Ok(frame)
    }

    pub(super) fn node(
        &mut self,
        node: &WidgetNode,
        inputs: &LiveInputs<'_>,
        frame: &mut LiveFrame,
    ) -> Result<(), String> {
        if !node.enabled || RuleEvaluator::loaded(node, inputs.fields()) == Some(false) {
            let mut visual = PreviewVisual::from_node(node);

            visual.active = Some(false);
            visual.visible = Some(false);
            frame.insert(
                node.id.clone(),
                TriggerState::Inactive,
                HashMap::from([(node.id.clone(), visual)]),
            );

            return Ok(());
        }

        let game = inputs.game();

        let mut context = PreviewContext::default();

        context.set_text(inputs.text().clone());
        context.fields = inputs.fields().clone();

        if matches!(
            game.header().status(),
            SourceStatus::Live | SourceStatus::Replay
        ) && let Some(entity) = node
            .entity_id
            .as_deref()
            .and_then(|id| id.parse::<u64>().ok())
            .and_then(|id| EntityId::new(id).ok())
            .and_then(|id| game.entities().get(&id))
        {
            super::live_fields::LiveFields::unit(
                &mut context.fields,
                "entity",
                entity,
                game.header().at_us(),
            );

            if let Some(statistics) =
                super::combat_projection::CombatProjection::actor(game, entity.id())
            {
                super::combat_projection::CombatProjection::fields(
                    &mut context.fields,
                    "entity",
                    &statistics,
                    super::combat_projection::CombatProjection::duration(game),
                );
            }
        }

        super::live_fields::LiveFields::selected_skill(
            &mut context.fields,
            game,
            node,
            inputs.resolver(),
        );

        let primary = match &node.trigger {
            Some(trigger) => self.primary(node, trigger, inputs.resolver(), game)?,
            None => TriggerResult::new(TriggerState::Active, Vec::new()),
        };

        let mut unsupported = primary.state() == TriggerState::Unsupported
            || node
                .binding
                .as_ref()
                .is_some_and(|_| !super::live_rows::LiveRows::supported(node));

        context.active = primary.state().active();
        Self::write_match(&mut context.fields, "t1.", &primary, game);

        if let Some(group) = &node.rules {
            for rule in &group.rules {
                let result = if let TriggerRule::Boss { trigger, .. } = rule {
                    if matches!(
                        game.header().status(),
                        SourceStatus::Live | SourceStatus::Replay
                    ) {
                        BossMatcher::evaluate(trigger, inputs)
                    } else {
                        TriggerResult::new(TriggerState::Unknown, Vec::new())
                    }
                } else {
                    self.rule(node, rule, game, inputs.resolver())?
                };

                let prefix = format!("rule.{}.", rule.id());

                unsupported |= result.state() == TriggerState::Unsupported;
                context
                    .fields
                    .insert(format!("{prefix}active"), json!(result.state().active()));
                Self::write_match(&mut context.fields, &prefix, &result, game);

                if let TriggerRule::Effect { trigger, .. } = rule
                    && trigger.clones
                {
                    context.instances.insert(
                        node.id.clone(),
                        result
                            .matches()
                            .iter()
                            .filter(|item| !trigger.effects.is_empty() || item.icon_visible())
                            .map(|item| {
                                item.fields(game)
                                    .into_iter()
                                    .map(|(key, value)| (format!("{prefix}{key}"), value))
                                    .collect()
                            })
                            .collect(),
                    );
                }
            }
        }

        let mut shallow = node.clone();

        shallow.children.clear();

        let mut visuals = PreviewEvaluator::evaluate(&[shallow], &context, inputs.formulas())?;

        super::live_images::LiveImages::apply(node, &context, inputs.resolver(), &mut visuals);

        if let Some(visual) = visuals.get_mut(&node.id) {
            if node.binding.as_ref().is_some_and(|binding| {
                binding.source == GroupDataSource::WorldBosses && binding.boss_ids.is_empty()
            }) {
                visual.active = Some(false);
                visual.visible = Some(false);
            }

            self.healing.apply(node, game, visual);

            if node
                .binding
                .as_ref()
                .is_some_and(|binding| binding.source == GroupDataSource::Effects)
            {
                super::effect_rows::EffectRows::apply(node, inputs, visual);
            }
        }

        let active = visuals.get(&node.id).and_then(|visual| visual.active);

        let state = if node.binding.is_some() && !super::live_rows::LiveRows::supported(node) {
            TriggerState::Unsupported
        } else {
            TriggerState::from_active(active, unsupported)
        };

        if state == TriggerState::Unsupported
            && let Some(visual) = visuals.get_mut(&node.id)
        {
            visual.active = None;
            visual.visible = None;
        }

        frame.insert(node.id.clone(), state, visuals);

        if node
            .binding
            .as_ref()
            .is_some_and(|binding| binding.source == GroupDataSource::Effects)
        {
            return Ok(());
        }

        if node
            .binding
            .as_ref()
            .is_some_and(|binding| binding.source == GroupDataSource::SkillHistory)
            && super::live_rows::LiveRows::supported(node)
        {
            self.skills.populate(node, inputs, frame);
            return Ok(());
        }

        if node.binding.is_some() && super::live_rows::LiveRows::supported(node) {
            return super::live_rows::LiveRows::populate(self, node, inputs, frame);
        }

        for child in &node.children {
            self.node(child, inputs, frame)?;
        }

        Ok(())
    }

    fn primary(
        &mut self,
        node: &WidgetNode,
        trigger: &TriggerDefinition,
        resolver: &SelectionResolver<'_>,
        game: &GameSnapshot,
    ) -> Result<TriggerResult, String> {
        let Some(ids) = resolver.ids(&trigger.selection) else {
            return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
        };

        let supported = match trigger.observation {
            ObservationKind::SkillUsed
            | ObservationKind::SkillState
            | ObservationKind::Cooldown
            | ObservationKind::SkillReady
            | ObservationKind::ComboAvailable => {
                game.capabilities().supports(GameEventType::SkillUsed)
            }

            ObservationKind::EffectPresent => game.capabilities().effects(),
        };

        if !supported {
            return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
        }

        if !matches!(
            game.header().status(),
            SourceStatus::Live | SourceStatus::Replay
        ) {
            return Ok(TriggerResult::new(TriggerState::Unknown, Vec::new()));
        }

        if matches!(
            trigger.observation,
            ObservationKind::SkillState
                | ObservationKind::Cooldown
                | ObservationKind::SkillReady
                | ObservationKind::ComboAvailable
        ) {
            if trigger.subject != TriggerSubject::SelfPlayer {
                return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
            }

            return Ok(super::cooldown_matcher::CooldownMatcher::evaluate(
                &trigger.observation,
                &ids,
                game,
                resolver,
            ));
        }

        if trigger.observation == ObservationKind::EffectPresent {
            let effects = ids
                .into_iter()
                .map(GameId::new)
                .collect::<Result<Vec<_>, _>>()?;

            let effect = EffectTrigger {
                unit: TriggerUnit::Player,
                member: String::new(),
                filter: EffectFilter::Any,
                effects,
                presence: EffectPresence::Present,
                origin: EffectOrigin::Any,
                min_stacks: 1.0,
                remaining_below_seconds: 0.0,
                clones: false,
                clone_direction: Default::default(),
            };

            return Ok(EffectMatcher::evaluate(&effect, None, game));
        }

        let actors = trigger
            .actor_key
            .as_deref()
            .and_then(|key| resolver.actor_ids(key));

        let signature = serde_json::to_string(trigger).map_err(|error| error.to_string())?;

        let duration = trigger
            .display_seconds
            .ok_or("Skill trigger duration is missing")?
            * 1000.0;

        if trigger.subject == TriggerSubject::SelfPlayer && game.player_id().is_none() {
            return Ok(TriggerResult::new(TriggerState::Unknown, Vec::new()));
        }

        Ok(self.events.evaluate(
            BindingKey::Primary(node.id.clone()),
            signature,
            game,
            (duration, 0.0),
            |event| {
                let GameEvent::SkillUsed {
                    source_id,
                    skill_id,
                    ..
                } = event
                else {
                    return Some(false);
                };

                if !resolver.matches_skill(&ids, skill_id.get()) {
                    return Some(false);
                }

                if trigger.subject == TriggerSubject::SelfPlayer {
                    return Some(game.player_id() == Some(*source_id));
                }

                let npc = game.entities().get(source_id)?.npc_id()?;

                Some(actors.as_ref()?.contains(&npc.get()))
            },
        ))
    }

    fn rule(
        &mut self,
        node: &WidgetNode,
        rule: &TriggerRule,
        game: &GameSnapshot,
        resolver: &SelectionResolver<'_>,
    ) -> Result<TriggerResult, String> {
        let entity = node
            .entity_id
            .as_deref()
            .and_then(|id| id.parse().ok())
            .and_then(|id| EntityId::new(id).ok());

        let supported = match rule {
            TriggerRule::Effect { trigger, .. } => EffectMatcher::supported(trigger, game),
            TriggerRule::Event { trigger, .. } => EventMatcher::supported(trigger, game),
            TriggerRule::Boss { .. } | TriggerRule::Value { .. } => true,
        };

        if !supported {
            return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
        }

        if !matches!(
            game.header().status(),
            SourceStatus::Live | SourceStatus::Replay
        ) {
            return Ok(TriggerResult::new(TriggerState::Unknown, Vec::new()));
        }

        Ok(match rule {
            TriggerRule::Boss { .. } | TriggerRule::Value { .. } => {
                TriggerResult::new(TriggerState::Unknown, Vec::new())
            }
            TriggerRule::Effect { trigger, .. } => {
                let mut actual = trigger.clone();

                if let Some(selections) = node.rule_selections.get(rule.id())
                    && !selections.is_empty()
                {
                    actual.effects.clear();

                    for selection in selections {
                        let Some(ids) = resolver.ids(selection) else {
                            return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
                        };

                        actual.effects.extend(
                            ids.into_iter()
                                .map(GameId::new)
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                    }
                }

                EffectMatcher::evaluate(&actual, entity, game)
            }
            TriggerRule::Event { trigger, .. } if EventMatcher::supported(trigger, game) => {
                if !EventMatcher::ready(trigger, entity, game) {
                    return Ok(TriggerResult::new(TriggerState::Unknown, Vec::new()));
                }

                let mut actual = trigger.clone();

                let mut selected = Vec::new();

                if trigger.use_widget_skill == Some(true) {
                    let Some(primary) = &node.trigger else {
                        return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
                    };

                    if primary.observation == ObservationKind::EffectPresent {
                        return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
                    }

                    let Some(ids) = resolver.ids(&primary.selection) else {
                        return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
                    };

                    selected = ids;
                } else if let Some(selections) = node.rule_selections.get(rule.id()) {
                    for selection in selections {
                        let Some(ids) = resolver.ids(selection) else {
                            return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
                        };

                        selected.extend(ids);
                    }
                }

                if !selected.is_empty() {
                    actual.skill_id = None;
                    actual.effect_id = None;
                }

                let actors = match trigger.source_actor_key.as_deref() {
                    Some(key) => {
                        let Some(ids) = resolver.actor_ids(key) else {
                            return Ok(TriggerResult::new(TriggerState::Unsupported, Vec::new()));
                        };

                        ids
                    }

                    None => Vec::new(),
                };

                let signature = serde_json::to_string(&(rule, &node.entity_id, &selected, &actors))
                    .map_err(|error| error.to_string())?;

                self.events.evaluate(
                    BindingKey::Rule {
                        widget: node.id.clone(),
                        rule: rule.id().to_owned(),
                    },
                    signature,
                    game,
                    (trigger.duration_ms, trigger.delay_ms.unwrap_or(0.0)),
                    |event| {
                        if !selected.is_empty() && event.event_type() == Some(actual.event) {
                            let selected_event = if matches!(
                                actual.event,
                                GameEventType::EffectApplied
                                    | GameEventType::EffectUpdated
                                    | GameEventType::EffectRemoved
                            ) {
                                event.effect_id().map(|id| selected.contains(&id.get()))
                            } else {
                                event
                                    .skill_id()
                                    .map(|id| resolver.matches_skill(&selected, id.get()))
                            }?;

                            if !selected_event {
                                return Some(false);
                            }
                        }

                        if !actors.is_empty() && event.event_type() == Some(actual.event) {
                            let npc = event
                                .source()
                                .and_then(|id| game.entities().get(&id))?
                                .npc_id()?;

                            if !actors.contains(&npc.get()) {
                                return Some(false);
                            }
                        }

                        EventMatcher::matches(event, &actual, entity, game)
                    },
                )
            }

            TriggerRule::Event { .. } => TriggerResult::new(TriggerState::Unsupported, Vec::new()),
        })
    }

    fn write_match(
        fields: &mut HashMap<String, Value>,
        prefix: &str,
        result: &TriggerResult,
        game: &GameSnapshot,
    ) {
        if let Some(first) = result.matches().first() {
            fields.extend(
                first
                    .fields(game)
                    .into_iter()
                    .map(|(key, value)| (format!("{prefix}{key}"), value)),
            );
        }
    }
}
