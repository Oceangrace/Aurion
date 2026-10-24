use crate::core::geometry::Viewport;
use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::document::UnavailableProfile;
use serde::Serialize;

#[derive(Clone, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InterfaceSnapshot {
    pub(in crate::modules::interfaces) workspace: InterfaceWorkspace,
    pub(in crate::modules::interfaces) viewport: Viewport,
    pub(in crate::modules::interfaces) load_error: Option<String>,
    pub(in crate::modules::interfaces) unavailable: Vec<UnavailableProfile>,
}
