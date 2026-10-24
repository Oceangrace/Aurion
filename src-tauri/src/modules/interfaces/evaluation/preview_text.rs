use super::WorldBossStatus;
use crate::core::localization::LocaleRegistry;
use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) struct PreviewText {
    true_value: String,
    false_value: String,
    locale: String,
    world_bosses: BTreeMap<WorldBossStatus, String>,
}

impl PreviewText {
    pub(crate) fn new(true_value: String, false_value: String, locale: String) -> Self {
        Self {
            true_value,
            false_value,
            locale,
            world_bosses: BTreeMap::new(),
        }
    }

    pub(crate) fn locale(&self) -> &str {
        &self.locale
    }

    pub(crate) fn with_world_bosses(mut self, locales: &LocaleRegistry) -> Self {
        self.world_bosses = WorldBossStatus::ALL
            .into_iter()
            .map(|status| {
                (
                    status,
                    locales.message(
                        &self.locale,
                        &["modules", "interfaces", "bossStates", status.as_str()],
                    ),
                )
            })
            .collect();
        self
    }

    pub(super) fn world_boss(&self, status: WorldBossStatus) -> &str {
        self.world_bosses
            .get(&status)
            .map_or_else(|| status.as_str(), String::as_str)
    }

    pub(crate) fn boolean(&self, value: bool) -> &str {
        if value {
            return &self.true_value;
        }

        &self.false_value
    }
}

impl Default for PreviewText {
    fn default() -> Self {
        Self::new(true.to_string(), false.to_string(), String::new())
    }
}
