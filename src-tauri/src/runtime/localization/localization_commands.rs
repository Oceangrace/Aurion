use crate::core::localization::LocaleRegistry;
use crate::core::surface::Surface;
use std::sync::Arc;
use tauri::{State, WebviewWindow};

#[tauri::command]
pub(crate) fn localization_read(
    window: WebviewWindow,
    locales: State<'_, Arc<LocaleRegistry>>,
) -> Result<serde_json::Value, String> {
    Surface::Editor.require(window.label())?;

    Ok(locales.snapshot())
}
