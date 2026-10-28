use super::CatalogItem;
use serde::Deserialize;

#[derive(Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CatalogSource {
    #[default]
    All,
    ClassSkills,
}

impl CatalogSource {
    pub(super) fn matches(&self, item: &CatalogItem) -> bool {
        match self {
            Self::All => true,
            Self::ClassSkills => !item.source_classes.is_empty(),
        }
    }
}
