//! Сохраняемые документы, рабочая область и дерево виджетов.

mod document_format;
mod interface_display;
mod interface_document;
mod interface_localization;
mod interface_snapshot;
mod interface_workspace;
mod unavailable_profile;
mod widget_kind;
mod widget_node;

pub(crate) use self::document_format::DocumentFormat;
pub(crate) use self::interface_display::InterfaceDisplay;
pub(crate) use self::interface_document::InterfaceDocument;
pub(crate) use self::interface_localization::InterfaceLocalization;
pub(crate) use self::interface_snapshot::InterfaceSnapshot;
pub(crate) use self::interface_workspace::InterfaceWorkspace;
pub(crate) use self::unavailable_profile::UnavailableProfile;
pub(crate) use self::widget_kind::WidgetKind;
pub(crate) use self::widget_node::WidgetNode;
