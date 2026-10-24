use serde::Serialize;

#[derive(Clone, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UnavailableProfile {
    pub(in crate::modules::interfaces) folder: String,
    pub(in crate::modules::interfaces) reason: String,
    #[serde(skip)]
    pub(in crate::modules::interfaces) fingerprint: Option<String>,
}

impl UnavailableProfile {
    pub(in crate::modules::interfaces) fn same(&self, other: &Self) -> bool {
        self.folder == other.folder && self.fingerprint == other.fingerprint
    }
}
