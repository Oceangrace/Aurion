use crate::core::preferences::OverlayPreferences;
use crate::core::surface::Surface;
use crate::runtime::PreferencesRepository;
use crate::runtime::Tray;
use std::sync::Mutex;
use tauri::{Manager, State, WebviewWindow};

#[tauri::command]
pub(crate) fn preferences_read(
    window: WebviewWindow,
    repository: State<'_, Mutex<PreferencesRepository>>,
) -> Result<OverlayPreferences, String> {
    Surface::Editor.require(window.label())?;

    repository
        .lock()
        .map_err(|_| "Preferences unavailable")?
        .read()
}

#[tauri::command]
pub(crate) fn preferences_save(
    window: WebviewWindow,
    repository: State<'_, Mutex<PreferencesRepository>>,
    preferences: OverlayPreferences,
) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    let repository = repository.lock().map_err(|_| "Preferences unavailable")?;

    let previous = repository.read()?;

    let locale = preferences.locale.clone();

    repository.save(preferences)?;

    if previous.locale != locale {
        Tray::localize(window.app_handle(), &locale).map_err(|error| error.to_string())?;
    }

    Ok(())
}
