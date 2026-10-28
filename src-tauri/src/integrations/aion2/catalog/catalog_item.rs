use crate::integrations::aion2::catalog::CatalogForm;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogItem {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) name_known: bool,
    pub(crate) context: String,
    pub(crate) icon: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) source_classes: Vec<String>,
    pub(crate) forms: Vec<CatalogForm>,
}

impl CatalogItem {
    pub(crate) fn match_ids(&self) -> Vec<u32> {
        self.forms
            .iter()
            .flat_map(|form| form.match_ids.iter().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
