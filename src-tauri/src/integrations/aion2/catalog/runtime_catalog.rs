use super::effect_metadata::EffectMetadata;
use super::npc_metadata::NpcMetadata;
use super::skill_metadata::SkillMetadata;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RuntimeCatalog {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u8,
    client_build: u32,
    catalog_version: String,
    effects: BTreeMap<NonZeroU32, EffectMetadata>,
    skills: BTreeMap<NonZeroU32, SkillMetadata>,
    npcs: BTreeMap<NonZeroU32, NpcMetadata>,
    #[serde(default)]
    non_health_skill_effects: BTreeSet<NonZeroU32>,
}

impl RuntimeCatalog {
    pub(crate) fn parse(bytes: &[u8], build: u32, version: &str) -> Result<Self, String> {
        let catalog: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;

        if catalog.schema_version != 1
            || catalog.client_build != build
            || catalog.catalog_version != version
            || catalog.effects.is_empty()
            || catalog.skills.is_empty()
            || catalog.npcs.is_empty()
            || catalog.effects.len() > 100000
            || catalog.skills.len() > 100000
            || catalog.npcs.len() > 100000
            || catalog.non_health_skill_effects.len() > 100000
        {
            return Err("Invalid runtime catalog".into());
        }

        Ok(catalog)
    }

    pub(crate) fn icon_paths(&self) -> impl Iterator<Item = &str> {
        self.skills
            .values()
            .filter_map(SkillMetadata::icon)
            .chain(self.effects.values().filter_map(EffectMetadata::icon))
    }

    pub(crate) fn npc_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.npcs.keys().map(|id| id.get())
    }

    pub(crate) fn skill_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.skills.keys().map(|id| id.get())
    }

    pub(crate) fn effect_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.effects.keys().map(|id| id.get())
    }

    pub(crate) fn effect(&self, id: u32) -> Option<&EffectMetadata> {
        self.effects.get(&NonZeroU32::new(id)?)
    }

    pub(crate) fn skill(&self, id: u32) -> Option<&SkillMetadata> {
        self.skills.get(&NonZeroU32::new(id)?)
    }

    pub(crate) fn non_health_skill_effect(&self, id: u32) -> bool {
        NonZeroU32::new(id).is_some_and(|id| self.non_health_skill_effects.contains(&id))
    }

    pub(crate) fn npc(&self, id: u32) -> Option<&NpcMetadata> {
        self.npcs.get(&NonZeroU32::new(id)?)
    }
}
