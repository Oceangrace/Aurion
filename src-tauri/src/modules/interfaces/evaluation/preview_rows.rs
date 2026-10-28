use super::preview_skill_history::PreviewSkillHistory;
use super::world_boss_preview::WorldBossPreview;
use crate::modules::interfaces::document::WidgetNode;
use crate::modules::interfaces::evaluation::PreviewContext;
use crate::modules::interfaces::evaluation::PreviewInstance;
use crate::modules::interfaces::evaluation::PreviewVisual;
use crate::modules::interfaces::evaluation::WidgetVisualResolver;
use crate::modules::interfaces::layout::GroupDataSource;
use crate::modules::interfaces::layout::GroupSort;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;

pub(crate) struct PreviewRows;

impl PreviewRows {
    pub(crate) fn populate(
        node: &WidgetNode,
        context: &PreviewContext,
        fields: &HashMap<String, Value>,
        result: &mut HashMap<String, PreviewVisual>,
        evaluate: impl Fn(
            &WidgetNode,
            &HashMap<String, Value>,
        ) -> Result<HashMap<String, PreviewVisual>, String>,
    ) -> Result<(), String> {
        let limit = node
            .binding
            .as_ref()
            .map(|value| value.limit)
            .unwrap_or(128) as usize;

        let group_limit = if node.group.limit > 0.0 {
            node.group.limit as usize
        } else {
            128
        };

        let boss_rows = node
            .binding
            .as_ref()
            .filter(|binding| binding.source == GroupDataSource::WorldBosses)
            .map(|_| WorldBossPreview::rows(node, fields, context.text()));
        let history_rows = node
            .binding
            .as_ref()
            .filter(|binding| binding.source == GroupDataSource::SkillHistory)
            .and_then(|_| PreviewSkillHistory::rows(node, fields));
        let mut rows: Vec<_> = boss_rows
            .as_ref()
            .or(history_rows.as_ref())
            .unwrap_or(&context.rows)
            .iter()
            .collect();

        if node.group.sort != GroupSort::Manual {
            rows.sort_by(|left, right| {
                let left = left.get("t1.remaining").and_then(Value::as_f64);

                let right = right.get("t1.remaining").and_then(Value::as_f64);

                match (left, right) {
                    (Some(left), Some(right)) => {
                        let order = left.total_cmp(&right);

                        if node.group.sort == GroupSort::RemainingDesc {
                            order.reverse()
                        } else {
                            order
                        }
                    }

                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }

        rows.truncate(limit.min(group_limit));

        for child in &node.children {
            let mut base = WidgetVisualResolver::resolve(child, context, fields)?;

            base.visible = Some(false);

            for (index, row) in rows.iter().enumerate() {
                let mut data = fields.clone();

                data.extend((*row).clone());
                data.insert("row.rank".into(), json!(index + 1));

                let mut row_visuals = evaluate(child, &data)?;

                if let Some(mut visual) = row_visuals.remove(&child.id) {
                    if node
                        .binding
                        .as_ref()
                        .is_some_and(|binding| binding.source == GroupDataSource::SkillHistory)
                    {
                        visual.remaining = Some(
                            node.group.span_seconds * (index + 1) as f64 / (rows.len() + 1) as f64,
                        );
                        visual.duration = Some(node.group.span_seconds);
                        visual.active = Some(true);
                        visual.visible = Some(child.enabled);

                        if history_rows.is_some() {
                            PreviewSkillHistory::apply(node, row, &mut visual);
                        }
                    }

                    visual.child_visuals = row_visuals;

                    let instances = std::mem::take(&mut visual.instances);

                    if instances.is_empty() {
                        if history_rows.is_none() {
                            super::row_layout::RowLayout::apply(
                                node,
                                rows.len(),
                                index,
                                &mut visual,
                            );
                        }
                        base.instances.push(PreviewInstance {
                            key: if history_rows.is_some() {
                                row.get("row.id")
                                    .and_then(Value::as_str)
                                    .map(str::to_owned)
                                    .unwrap_or_else(|| index.to_string())
                            } else {
                                index.to_string()
                            },
                            visual: Box::new(visual),
                        });
                    } else {
                        for instance in instances {
                            let mut item = *instance.visual;

                            item.child_visuals = visual.child_visuals.clone();
                            if history_rows.is_none() {
                                super::row_layout::RowLayout::apply(
                                    node,
                                    rows.len(),
                                    index,
                                    &mut item,
                                );
                            }
                            base.instances.push(PreviewInstance {
                                key: format!("{index}:{}", instance.key),
                                visual: Box::new(item),
                            });
                        }
                    }
                }
            }

            result.insert(child.id.clone(), base);
        }

        Ok(())
    }
}
