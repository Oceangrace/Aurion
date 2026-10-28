use super::journal_metric::JournalMetric;
use super::journal_period::JournalPeriod;
use super::journal_scope::JournalScope;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct JournalWindow {
    #[schemars(length(min = 1, max = 100))]
    id: String,
    #[schemars(length(min = 1, max = 80))]
    name: String,
    visible: bool,
    #[serde(default)]
    locked: bool,
    #[schemars(range(min = -100_000, max = 100_000))]
    x: f64,
    #[schemars(range(min = -100_000, max = 100_000))]
    y: f64,
    #[schemars(range(min = 220, max = 1920))]
    width: f64,
    #[schemars(range(min = 100, max = 1440))]
    height: f64,
    metric: JournalMetric,
    scope: JournalScope,
    period: JournalPeriod,
    #[schemars(range(min = 1, max = 40))]
    limit: u32,
    merge_summons: bool,
}

impl Default for JournalWindow {
    fn default() -> Self {
        Self {
            id: "damage".into(),
            name: "DPS".into(),
            visible: true,
            locked: false,
            x: 24.0,
            y: 180.0,
            width: 340.0,
            height: 200.0,
            metric: JournalMetric::Dps,
            scope: JournalScope::Group,
            period: JournalPeriod::Current,
            limit: 20,
            merge_summons: true,
        }
    }
}

impl JournalWindow {
    pub(super) fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 100
            || self.name.trim().is_empty()
            || self.name.chars().count() > 80
            || !(1..=40).contains(&self.limit)
        {
            return Err("Invalid journal window".into());
        }

        if [self.x, self.y, self.width, self.height]
            .iter()
            .any(|v| !v.is_finite())
            || self.x.abs() > 100_000.0
            || self.y.abs() > 100_000.0
            || !(220.0..=1920.0).contains(&self.width)
            || !(100.0..=1440.0).contains(&self.height)
        {
            return Err("Invalid journal window geometry".into());
        }

        Ok(())
    }
}
