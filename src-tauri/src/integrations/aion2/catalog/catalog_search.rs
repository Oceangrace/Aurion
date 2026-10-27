use super::catalog_source::CatalogSource;
use crate::integrations::aion2::catalog::CatalogScope;
use serde::Deserialize;

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CatalogSearch {
    #[serde(default)]
    pub(crate) offset: usize,
    #[serde(default)]
    pub(crate) source: CatalogSource,
    #[serde(default)]
    pub(crate) match_ids: Option<Vec<u32>>,
    #[serde(default)]
    pub(crate) locale: String,
    pub(crate) scope: CatalogScope,
    pub(crate) context_key: Option<String>,
    pub(crate) text: String,
    pub(crate) category: Option<String>,
}
