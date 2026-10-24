use crate::core::diagnostics::PageHeartbeat;
use crate::core::game::GameObservationSource;
use crate::core::localization::LocaleRegistry;
use crate::core::surface::Surface;
use crate::integrations::aion2::catalog::CatalogPage;
use crate::integrations::aion2::catalog::CatalogRecord;
use crate::integrations::aion2::catalog::CatalogScope;
use crate::integrations::aion2::catalog::CatalogSearch;
use crate::modules::interfaces::InterfaceService;
use crate::modules::interfaces::document::DocumentFormat;
use crate::modules::interfaces::document::InterfaceDocument;
use crate::modules::interfaces::document::InterfaceSnapshot;
use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::evaluation::LiveUpdate;
use crate::modules::interfaces::evaluation::PreviewContext;
use crate::modules::interfaces::evaluation::PreviewText;
use crate::modules::interfaces::evaluation::PreviewVisual;
use crate::modules::interfaces::exchange::ImportPreview;
use crate::modules::interfaces::exchange::ProfilePackage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

#[tauri::command]
pub(crate) fn interfaces_read(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
) -> Result<InterfaceSnapshot, String> {
    Surface::Editor.require(window.label())?;

    Ok(state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .read())
}

#[tauri::command]
pub(crate) fn interfaces_save(
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, Mutex<InterfaceService>>,
    workspace: InterfaceWorkspace,
) -> Result<InterfaceSnapshot, String> {
    Surface::Editor.require(window.label())?;

    let snapshot = state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .save(workspace)?;

    let _ = app.emit("interfaces:changed", &snapshot);

    Ok(snapshot)
}

#[tauri::command]
pub(crate) fn interfaces_remove_unavailable(
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, Mutex<InterfaceService>>,
    folder: String,
) -> Result<InterfaceSnapshot, String> {
    Surface::Editor.require(window.label())?;

    let snapshot = state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .remove_unavailable(&folder)?;

    let _ = app.emit("interfaces:changed", &snapshot);

    Ok(snapshot)
}

#[tauri::command]
pub(crate) fn catalog_search(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    request: CatalogSearch,
) -> Result<CatalogPage, String> {
    Surface::Editor.require(window.label())?;

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .search(request)
}

#[tauri::command]
pub(crate) fn catalog_item(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    scope: CatalogScope,
    context_key: Option<String>,
    key: String,
    locale: String,
) -> Result<CatalogRecord, String> {
    Surface::Editor.require(window.label())?;

    Ok(state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .catalog_item(&scope, context_key.as_deref(), &key, &locale))
}

#[tauri::command]
pub(crate) fn interfaces_preview(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    workspace: InterfaceWorkspace,
    mut context: PreviewContext,
    locale: String,
    locales: State<'_, Arc<LocaleRegistry>>,
) -> Result<HashMap<String, PreviewVisual>, String> {
    Surface::Editor.require(window.label())?;

    context.set_text(
        PreviewText::new(
            locales.message(&locale, &["modules", "interfaces", "values", "true"]),
            locales.message(&locale, &["modules", "interfaces", "values", "false"]),
            locale.clone(),
        )
        .with_world_bosses(&locales),
    );

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .preview(&workspace, &context)
}

#[tauri::command]
pub(crate) fn interfaces_import(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    text: String,
) -> Result<ImportPreview, String> {
    Surface::Editor.require(window.label())?;

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .inspect_import(&text)
}

#[tauri::command]
pub(crate) fn interfaces_validate(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    workspace: InterfaceWorkspace,
) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .validate(&workspace)
}

#[tauri::command]
pub(crate) fn interfaces_export(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    document: InterfaceDocument,
) -> Result<String, String> {
    Surface::Editor.require(window.label())?;

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .export(&document)
}

#[tauri::command]
pub(crate) async fn interfaces_export_file(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    text: String,
    locale: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    Surface::Editor.require(window.label())?;

    if !text.starts_with(DocumentFormat::PREFIX) {
        return Err("Expected an export string".into());
    }

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .inspect_import(&text)?;
    let locales = window.state::<Arc<LocaleRegistry>>().inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        let selected = window
            .dialog()
            .file()
            .set_parent(&window)
            .set_title(locales.text(&locale, "saveTitle"))
            .set_file_name("aurion-interface.txt")
            .add_filter(locales.text(&locale, "interfaceFilter"), &["txt"])
            .blocking_save_file();

        let Some(selected) = selected else {
            return Ok(None);
        };

        let path = selected.into_path().map_err(|error| error.to_string())?;

        std::fs::write(&path, text).map_err(|error| error.to_string())?;

        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub(crate) async fn interfaces_export_folder(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    document: InterfaceDocument,
    locale: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    Surface::Editor.require(window.label())?;

    let mut workspace = InterfaceWorkspace::empty();

    workspace.documents.push(document.clone());
    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .validate(&workspace)?;
    let locales = window.state::<Arc<LocaleRegistry>>().inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        let selected = window
            .dialog()
            .file()
            .set_parent(&window)
            .set_title(locales.text(&locale, "exportFolderTitle"))
            .blocking_pick_folder();

        let Some(selected) = selected else {
            return Ok(None);
        };

        let parent = selected.into_path().map_err(|error| error.to_string())?;
        let path = ProfilePackage::export(&parent, &document)?;

        Ok(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub(crate) async fn interfaces_import_folder(
    window: WebviewWindow,
    app: AppHandle,
    locale: String,
) -> Result<Option<ImportPreview>, String> {
    use tauri_plugin_dialog::DialogExt;

    Surface::Editor.require(window.label())?;

    let locales = window.state::<Arc<LocaleRegistry>>().inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        let selected = window
            .dialog()
            .file()
            .set_parent(&window)
            .set_title(locales.text(&locale, "importFolderTitle"))
            .blocking_pick_folder();

        let Some(selected) = selected else {
            return Ok(None);
        };

        let path = selected.into_path().map_err(|error| error.to_string())?;
        let document = ProfilePackage::import(&path)?;
        let mut workspace = InterfaceWorkspace::empty();

        workspace.documents.push(document.clone());
        app.state::<Mutex<InterfaceService>>()
            .lock()
            .map_err(|_| "Interface state unavailable")?
            .validate(&workspace)?;

        Ok(Some(ImportPreview { document }))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub(crate) fn interfaces_live(
    window: WebviewWindow,
    state: State<'_, Mutex<InterfaceService>>,
    source: State<'_, Arc<dyn GameObservationSource>>,
    locale: String,
    revision: Option<u64>,
    locales: State<'_, Arc<LocaleRegistry>>,
    heartbeat: State<'_, Arc<PageHeartbeat>>,
) -> Result<LiveUpdate, String> {
    Surface::Editor.require(window.label())?;

    heartbeat.beat();

    let frame = source.frame()?;
    let text = PreviewText::new(
        locales.message(&locale, &["modules", "interfaces", "values", "true"]),
        locales.message(&locale, &["modules", "interfaces", "values", "false"]),
        locale.clone(),
    )
    .with_world_bosses(&locales);

    state
        .lock()
        .map_err(|_| "Interface state unavailable")?
        .live_update(&frame, text, revision)
}
