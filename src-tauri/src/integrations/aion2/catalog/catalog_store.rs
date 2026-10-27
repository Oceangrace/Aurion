use super::CatalogManifest;
use super::ZoneCatalog;
use super::field_boss_catalog::FieldBossCatalog;
use super::source::CatalogEntry;
use super::source::EnemyCatalog;
use super::source::PlayerCatalog;
use crate::config::CatalogConfiguration;
use crate::core::localization::LocaleRegistry;
use crate::core::localization::TextMatcher;
use crate::integrations::aion2::catalog::CatalogItem;
use crate::integrations::aion2::catalog::CatalogLabels;
use crate::integrations::aion2::catalog::CatalogLocalization;
use crate::integrations::aion2::catalog::CatalogPage;
use crate::integrations::aion2::catalog::CatalogScope;
use crate::integrations::aion2::catalog::CatalogSearch;
use crate::integrations::aion2::catalog::SkillNameCatalog;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) struct CatalogStore {
    runtime: Arc<super::RuntimeCatalog>,
    icons: super::catalog_icons::CatalogIcons,
    version: String,
    matcher: Arc<dyn TextMatcher>,
    localization: CatalogLocalization,
    skills: Vec<CatalogItem>,
    effects: Vec<CatalogItem>,
    enemies: Vec<CatalogItem>,
    enemy_skills: HashMap<String, Vec<CatalogItem>>,
    enemy_effects: HashMap<String, Vec<CatalogItem>>,
}

impl CatalogStore {
    pub(crate) fn load(
        manifest_bytes: &[u8],
        player_bytes: &[u8],
        enemy_bytes: &[u8],
        configuration: &CatalogConfiguration,
        locales: Arc<LocaleRegistry>,
        matcher: Arc<dyn TextMatcher>,
        runtime: Arc<super::RuntimeCatalog>,
    ) -> Result<Self, String> {
        let manifest = CatalogManifest::parse(manifest_bytes)?;

        let version = manifest.version();

        locales.validate_catalogs(version)?;
        let mut store = Self {
            runtime,
            icons: super::catalog_icons::CatalogIcons::default(),
            matcher,
            localization: CatalogLocalization::new(locales, configuration.source_locale().into()),
            version: version.to_owned(),
            skills: Vec::new(),
            effects: Vec::new(),
            enemies: Vec::new(),
            enemy_skills: HashMap::new(),
            enemy_effects: HashMap::new(),
        };

        manifest.verify(configuration.player(), player_bytes)?;
        manifest.verify(configuration.enemy(), enemy_bytes)?;
        let player: PlayerCatalog =
            serde_json::from_slice(player_bytes).map_err(|error| error.to_string())?;

        let enemy: EnemyCatalog =
            serde_json::from_slice(enemy_bytes).map_err(|error| error.to_string())?;

        if player.schema_version != 1
            || enemy.schema_version != 1
            || [
                manifest.client_build(),
                player.client_build,
                enemy.client_build,
            ]
            .iter()
            .any(|build| *build != configuration.client_build())
            || player.catalog_version != store.version
            || enemy.catalog_version != store.version
        {
            return Err("Catalog build or version mismatch".into());
        }

        store.skills = store.project_list(&player.skills, "")?;
        store.effects = store.project_list(&player.effects, "")?;

        for row in &enemy.entries {
            let key = row.entry.key();

            let item = row.entry.item(&row.context)?;

            let child_context = format!("{} · {}", item.name, row.context);

            store.enemy_skills.insert(
                key.to_owned(),
                store.project_list(&row.skills, &child_context)?,
            );
            store.enemy_effects.insert(
                key.to_owned(),
                store.project_list(&row.effects, &child_context)?,
            );
            store.enemies.push(item);
        }

        store.icons = super::catalog_icons::CatalogIcons::new(
            store
                .skills
                .iter()
                .chain(store.enemy_skills.values().flatten()),
            store
                .effects
                .iter()
                .chain(store.enemy_effects.values().flatten()),
        );

        Ok(store)
    }

    fn project_list(
        &self,
        data: &[CatalogEntry],
        context: &str,
    ) -> Result<Vec<CatalogItem>, String> {
        data.iter().map(|row| row.item(context)).collect()
    }

    pub(crate) fn icon_paths(&self) -> BTreeSet<String> {
        self.skills
            .iter()
            .chain(&self.effects)
            .chain(self.enemy_skills.values().flatten())
            .chain(self.enemy_effects.values().flatten())
            .filter_map(|item| item.icon.as_deref())
            .chain(self.runtime.icon_paths())
            .filter(|path| !path.is_empty())
            .map(str::to_owned)
            .collect()
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn labels(
        &self,
        names: &SkillNameCatalog,
        icon_files: &[String],
        zones: &ZoneCatalog,
    ) -> HashMap<String, CatalogLabels> {
        self.localization
            .languages()
            .map(|locale| {
                let mut labels = self.localized_labels(names, icon_files, locale);

                labels.zones = zones
                    .maps()
                    .map(|(id, name)| (id, self.localization.zone(name, locale)))
                    .collect();

                (locale.to_owned(), labels)
            })
            .collect()
    }

    fn localized_labels(
        &self,
        names: &SkillNameCatalog,
        icon_files: &[String],
        locale: &str,
    ) -> CatalogLabels {
        let mut labels = CatalogLabels {
            field_bosses: FieldBossCatalog::all(),
            ..Default::default()
        };
        let mut icons: HashMap<String, Vec<String>> = HashMap::new();

        for (id, name) in names.names() {
            labels.skills.insert(
                id,
                self.localization
                    .entry_name(&format!("skill-id:{id}"), locale)
                    .unwrap_or_else(|| name.to_owned()),
            );
        }

        self.add_labels(&mut labels.skills, &mut icons, &self.skills, locale);
        self.add_icon_labels(&mut icons, &self.effects, locale);

        for enemy in &self.enemies {
            let name = self.localization.translate(enemy, locale).name;

            if enemy.name_known {
                for id in enemy.match_ids() {
                    labels.npcs.entry(id).or_insert_with(|| name.clone());
                }
            }

            self.add_labels(
                &mut labels.skills,
                &mut icons,
                &self.enemy_skills[&enemy.key],
                locale,
            );
            self.add_icon_labels(&mut icons, &self.enemy_effects[&enemy.key], locale);
        }

        labels.icons = icon_files
            .iter()
            .map(|file| {
                let name = icons
                    .get(file)
                    .map(|names| names.join(" · "))
                    .unwrap_or_else(|| file.trim_end_matches(".png").to_owned());

                (file.clone(), name)
            })
            .collect();

        self.add_runtime_labels(&mut labels, locale);

        labels
    }

    fn add_runtime_labels(&self, labels: &mut CatalogLabels, locale: &str) {
        labels.automatic_skills = self
            .runtime
            .skill_ids()
            .filter(|id| {
                self.runtime
                    .skill(*id)
                    .is_some_and(|skill| skill.automatic())
            })
            .collect();

        for id in self.runtime.npc_ids() {
            if let Some(name) = self.npc_name(id, locale) {
                labels.npcs.entry(id).or_insert(name);
            }
        }

        for id in self
            .runtime
            .skill_ids()
            .chain(labels.skills.keys().copied())
        {
            if let Some(icon) = self.skill_icon(id) {
                labels.skill_icons.insert(id, icon.to_owned());
            }
        }

        for id in self.runtime.effect_ids() {
            if let Some(name) = self
                .localization
                .entry_name(&format!("effect-id:{id}"), locale)
            {
                labels.effects.insert(id, name);
            }

            if let Some(effect) = self.runtime.effect(id) {
                labels.effect_kinds.insert(id, effect.disposition());

                if !effect.display().icon_visible() {
                    labels.hidden_effects.push(id);
                }
            }

            if let Some(icon) = self.runtime.effect(id).and_then(|effect| effect.icon()) {
                labels.effect_icons.insert(id, icon.to_owned());
            }
        }

        for item in self
            .effects
            .iter()
            .chain(self.enemy_effects.values().flatten())
        {
            let name = self.localization.translate(item, locale).name;

            for id in item.match_ids() {
                labels.effects.entry(id).or_insert_with(|| name.clone());

                if let Some(icon) = &item.icon {
                    labels
                        .effect_icons
                        .entry(id)
                        .or_insert_with(|| icon.clone());
                }
            }
        }
    }

    fn add_labels(
        &self,
        names: &mut BTreeMap<u32, String>,
        icons: &mut HashMap<String, Vec<String>>,
        items: &[CatalogItem],
        locale: &str,
    ) {
        for item in items.iter().filter(|item| item.name_known) {
            let name = self.localization.translate(item, locale).name;

            for id in item.match_ids() {
                names.entry(id).or_insert_with(|| name.clone());
            }
        }

        self.add_icon_labels(icons, items, locale);
    }

    fn add_icon_labels(
        &self,
        icons: &mut HashMap<String, Vec<String>>,
        items: &[CatalogItem],
        locale: &str,
    ) {
        for item in items {
            let Some(icon) = &item.icon else {
                continue;
            };
            let file = format!("{}.png", icon.replace('/', "_").to_lowercase());
            let name = self.localization.translate(item, locale).name;
            let names = icons.entry(file).or_default();

            if !names.contains(&name) {
                names.push(name);
            }
        }
    }

    pub(crate) fn skill_icon(&self, id: u32) -> Option<&str> {
        self.runtime
            .skill(id)
            .and_then(|skill| skill.icon())
            .or_else(|| self.icons.skill(id))
    }

    pub(crate) fn runtime(&self) -> &super::RuntimeCatalog {
        &self.runtime
    }

    pub(crate) fn effect_icon(&self, id: u32) -> Option<&str> {
        self.runtime
            .effect(id)
            .and_then(|effect| effect.icon())
            .or_else(|| self.icons.effect(id))
    }

    pub(crate) fn npc_name(&self, id: u32, locale: &str) -> Option<String> {
        let mut matches = self
            .enemies
            .iter()
            .filter(|item| item.name_known)
            .filter(|item| item.forms.iter().any(|form| form.match_ids.contains(&id)));

        if let Some(item) = matches.next()
            && matches.next().is_none()
        {
            return Some(self.localization.translate(item, locale).name);
        }

        self.localization.entry_name(&format!("npc:{id}"), locale)
    }

    pub(crate) fn skill_name(&self, id: u32, locale: &str) -> Option<String> {
        let mut names = self
            .skills
            .iter()
            .chain(self.enemy_skills.values().flatten())
            .filter(|item| item.name_known)
            .filter(|item| item.forms.iter().any(|form| form.match_ids.contains(&id)))
            .map(|item| self.localization.translate(item, locale).name);

        if let Some(name) = names.next() {
            if names.all(|other| other == name) {
                return Some(name);
            }

            return None;
        }

        self.localization
            .entry_name(&format!("npc-skill:{id}"), locale)
    }

    fn items(&self, scope: &CatalogScope, context: Option<&str>) -> &[CatalogItem] {
        match scope {
            CatalogScope::Skills => &self.skills,
            CatalogScope::Effects => &self.effects,
            CatalogScope::Enemies => &self.enemies,
            CatalogScope::EnemySkills => context
                .and_then(|key| self.enemy_skills.get(key))
                .map_or(&[], Vec::as_slice),
            CatalogScope::EnemyEffects => context
                .and_then(|key| self.enemy_effects.get(key))
                .map_or(&[], Vec::as_slice),
        }
    }

    pub(crate) fn find(
        &self,
        scope: &CatalogScope,
        context: Option<&str>,
        key: &str,
    ) -> Option<&CatalogItem> {
        self.items(scope, context)
            .iter()
            .find(|item| item.key == key)
    }

    pub(crate) fn localized_item(
        &self,
        scope: &CatalogScope,
        context: Option<&str>,
        key: &str,
        locale: &str,
    ) -> Option<CatalogItem> {
        self.find(scope, context, key)
            .map(|item| self.localize(item, self.parent(scope, context), locale))
    }

    fn parent(&self, scope: &CatalogScope, context: Option<&str>) -> Option<&CatalogItem> {
        if !matches!(
            scope,
            CatalogScope::EnemySkills | CatalogScope::EnemyEffects
        ) {
            return None;
        }

        self.find(&CatalogScope::Enemies, None, context?)
    }

    fn localize(
        &self,
        item: &CatalogItem,
        parent: Option<&CatalogItem>,
        locale: &str,
    ) -> CatalogItem {
        match parent {
            Some(parent) => self.localization.child(item, parent, locale),
            None => self.localization.translate(item, locale),
        }
    }

    pub(crate) fn search(&self, request: CatalogSearch) -> Result<CatalogPage, String> {
        if request
            .match_ids
            .as_ref()
            .is_some_and(|ids| ids.len() > 128 || ids.contains(&0))
        {
            return Err("Invalid catalog match IDs".into());
        }

        if request.text.len() > 512 {
            return Err("Search query is too long".into());
        }

        let mut matches = Vec::new();

        let parent = self.parent(&request.scope, request.context_key.as_deref());

        for item in self.items(&request.scope, request.context_key.as_deref()) {
            if !request.source.matches(item) {
                continue;
            }

            if request
                .match_ids
                .as_ref()
                .is_some_and(|ids| ids.iter().any(|id| !item.match_ids().contains(id)))
            {
                continue;
            }

            let item = self.localize(item, parent, &request.locale);

            if self.text_matches(&item, &request.text, &request.locale)? {
                matches.push(item);
            }
        }

        let available_categories = matches
            .iter()
            .flat_map(|item| {
                item.categories.iter().cloned().chain(
                    item.source_classes
                        .iter()
                        .map(|class| format!("class:{class}")),
                )
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        let mut matches: Vec<CatalogItem> = matches
            .into_iter()
            .filter(|item| self.category_matches(item, request.category.as_deref()))
            .collect();

        matches.sort_by(|a, b| {
            self.localization
                .compare(&request.locale, &a.name, &b.name)
                .then_with(|| a.key.cmp(&b.key))
        });

        Ok(CatalogPage {
            catalog_version: self.version.clone(),
            total: matches.len(),
            available_categories,
            items: matches.into_iter().skip(request.offset).take(80).collect(),
        })
    }

    fn category_matches(&self, item: &CatalogItem, category: Option<&str>) -> bool {
        match category {
            None | Some("") => true,
            Some(value) => match value.strip_prefix("class:") {
                Some(class) => item.source_classes.iter().any(|source| source == class),
                None => item.categories.iter().any(|entry| entry == value),
            },
        }
    }

    fn text_matches(&self, item: &CatalogItem, query: &str, locale: &str) -> Result<bool, String> {
        let text = format!(
            "{} {} {} {}",
            item.name,
            item.context,
            item.source_classes.join(" "),
            item.key
        );

        let locale = self.localization.effective_locale(locale);

        for word in query.split_whitespace() {
            if word
                .parse::<u32>()
                .is_ok_and(|id| item.match_ids().contains(&id))
            {
                continue;
            }

            if !self.matcher.contains(locale, &text, word)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
