use crate::core::localization::LocaleRegistry;
use crate::integrations::aion2::catalog::CatalogItem;
use serde_json::Value;
use std::sync::Arc;

pub(crate) struct CatalogLocalization {
    locales: Arc<LocaleRegistry>,
    source_locale: String,
}

impl CatalogLocalization {
    pub(crate) fn new(locales: Arc<LocaleRegistry>, source_locale: String) -> Self {
        Self {
            locales,
            source_locale,
        }
    }

    pub(crate) fn translate(&self, item: &CatalogItem, locale: &str) -> CatalogItem {
        let locale = self.locales.effective(locale);

        let mut translated = item.clone();

        if locale != self.source_locale {
            self.apply(
                &mut translated,
                self.locales.pack(self.locales.fallback_locale()).catalog(),
            );
        }

        self.apply(&mut translated, self.locales.pack(locale).catalog());

        translated
    }

    pub(crate) fn child(
        &self,
        item: &CatalogItem,
        parent: &CatalogItem,
        locale: &str,
    ) -> CatalogItem {
        let mut translated = self.translate(item, locale);

        let locale = self.locales.effective(locale);

        let explicit_context = [locale, self.locales.fallback_locale()]
            .into_iter()
            .any(|locale| {
                self.locales.pack(locale).catalog()["entries"][&item.key]["context"].is_string()
            });

        if !explicit_context {
            let parent = self.translate(parent, locale);

            translated.context = format!("{} · {}", parent.name, parent.context);

            if let Some(id) = item.match_ids().first() {
                translated.context.push_str(&format!(" · ID {id}"));
            }
        }

        translated
    }

    fn apply(&self, item: &mut CatalogItem, catalog: &Value) {
        let row = &catalog["entries"][&item.key];

        if let Some(name) = row["name"].as_str() {
            item.name = name.to_owned();
        }

        if let Some(context) = row["context"].as_str() {
            item.context = context.to_owned();
        }

        for form in &mut item.forms {
            if form.key == item.key {
                form.name.clone_from(&item.name);
            }

            let text = &row["forms"][&form.key];

            if let Some(name) = text["name"].as_str() {
                form.name = name.to_owned();
            }
        }
    }

    /// A translation entry without a catalog item, such as `npc-skill:<id>`.
    pub(crate) fn entry_name(&self, key: &str, locale: &str) -> Option<String> {
        let locale = self.locales.effective(locale);

        [locale, self.locales.fallback_locale()]
            .into_iter()
            .find_map(|locale| self.locales.pack(locale).catalog()["entries"][key]["name"].as_str())
            .map(str::to_owned)
    }

    pub(crate) fn effective_locale<'a>(&'a self, locale: &'a str) -> &'a str {
        self.locales.effective(locale)
    }

    pub(crate) fn languages(&self) -> impl Iterator<Item = &str> {
        self.locales.languages()
    }

    pub(crate) fn zone(&self, name: &str, locale: &str) -> String {
        [
            self.locales.effective(locale),
            self.locales.fallback_locale(),
        ]
        .into_iter()
        .find_map(|locale| self.locales.pack(locale).zones()[name].as_str())
        .unwrap_or(name)
        .to_owned()
    }

    pub(crate) fn compare(&self, locale: &str, a: &str, b: &str) -> std::cmp::Ordering {
        self.locales.compare(locale, a, b)
    }
}
