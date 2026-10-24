use crate::core::geometry::OverlayRegion;
use crate::core::surface::Surface;
use crate::runtime::OverlayWindow;
use tauri::{AppHandle, State, WebviewWindow};

#[tauri::command]
pub(crate) fn app_wait_for_game(
    window: WebviewWindow,
    overlay: State<'_, OverlayWindow>,
) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    overlay.wait_for_game(&window)
}

#[tauri::command]
pub(crate) fn overlay_game_attached(
    window: WebviewWindow,
    overlay: State<'_, OverlayWindow>,
) -> Result<bool, String> {
    Surface::Editor.require(window.label())?;

    Ok(overlay.game_attached())
}

#[tauri::command]
pub(crate) fn overlay_foreground(
    window: WebviewWindow,
    overlay: State<'_, OverlayWindow>,
) -> Result<bool, String> {
    Surface::Editor.require(window.label())?;

    Ok(overlay.foreground())
}

#[tauri::command]
pub(crate) fn overlay_game_panel(
    window: WebviewWindow,
    overlay: State<'_, OverlayWindow>,
) -> Result<bool, String> {
    Surface::Editor.require(window.label())?;

    Ok(overlay.game_panel())
}

#[tauri::command]
pub(crate) fn app_exit(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    app.exit(0);

    Ok(())
}

#[tauri::command]
pub(crate) fn overlay_regions(
    window: WebviewWindow,
    overlay: State<'_, OverlayWindow>,
    regions: Vec<OverlayRegion>,
) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    overlay.set_regions(regions)
}

#[tauri::command]
pub(crate) fn overlay_capture(
    window: WebviewWindow,
    overlay: State<'_, OverlayWindow>,
    captured: bool,
) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    overlay.set_dragging(captured);

    Ok(())
}

#[tauri::command]
pub(crate) fn app_minimize(window: WebviewWindow) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    window.minimize().map_err(|error| error.to_string())
}
