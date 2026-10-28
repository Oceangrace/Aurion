use super::CatalogChoice;
use crate::integrations::aion2::catalog::CatalogItem;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CatalogEntry {
    key: String,
    name: String,
    #[serde(default)]
    name_known: Option<bool>,
    icon: Option<String>,
    categories: Vec<String>,
    classes: Vec<String>,
    forms: Vec<CatalogChoice>,
}

impl CatalogEntry {
    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn item(&self, context: &str) -> Result<CatalogItem, String> {
        if self.key.trim().is_empty() || self.name.trim().is_empty() || self.forms.is_empty() {
            return Err("Invalid catalog entry".into());
        }

        let forms = self
            .forms
            .iter()
            .map(CatalogChoice::form)
            .collect::<Result<_, _>>()?;

        Ok(CatalogItem {
            key: self.key.clone(),
            name: self.name.clone(),
            name_known: self.name_known.unwrap_or(true),
            icon: self.icon.clone(),
            context: context.to_owned(),
            categories: self.categories.clone(),
            source_classes: self.classes.clone(),
            forms,
        })
    }
}
