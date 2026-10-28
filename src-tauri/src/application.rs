use crate::config::ApplicationConfiguration;
use crate::core::diagnostics::PageHeartbeat;
use crate::core::geometry::Viewport;
use crate::core::icons::IconCache;
use crate::core::icons::IconManifest;
use crate::core::localization::LocaleRegistry;
use crate::core::surface::Surface;
use crate::core::updates::ReleaseInfo;
use crate::integrations::aion2::PacketPipeline;
use crate::integrations::aion2::catalog::CatalogLabels;
use crate::integrations::aion2::catalog::CatalogManifest;
use crate::integrations::aion2::catalog::CatalogStore;
use crate::integrations::aion2::source::GameSource;
use crate::integrations::windows::StartupDialog;
use crate::integrations::windows::WindowsIconTransport;
use crate::integrations::windows::WindowsReleases;
use crate::integrations::windows::WindowsSpeechEngine;
use crate::integrations::windows::WindowsTextMatcher;
use crate::modules::combat_journal::JournalArchive;
use crate::modules::combat_journal::JournalRepository;
use crate::modules::combat_journal::journal_commands;
use crate::modules::interfaces::InterfaceRepository;
use crate::modules::interfaces::InterfaceService;
use crate::modules::interfaces::SpeechService;
use crate::modules::interfaces::interface_commands;
use crate::modules::interfaces::speech_commands;
use crate::runtime::IconProtocol;
use crate::runtime::IconService;
use crate::runtime::LanguageRepository;
use crate::runtime::OverlayWindow;
use crate::runtime::PreferencesRepository;
use crate::runtime::QuitRequest;
use crate::runtime::Tray;
use crate::runtime::localization_commands;
use crate::runtime::preferences_commands;
use crate::runtime::window_commands;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{Manager, State, WebviewWindow};

pub(crate) struct Application;

impl Application {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn start(self) {
        let result = tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .register_asynchronous_uri_scheme_protocol("game-icon", IconProtocol::handle)
            .invoke_handler(tauri::generate_handler![
                catalog_labels,
                icon_files,
                app_release_check,
                app_release_open,
                app_version,
                journal_commands::journal_frame,
                journal_commands::journal_archive_read,
                journal_commands::journal_archive_save,
                journal_commands::journal_export_file,
                journal_commands::journal_preferences_read,
                journal_commands::journal_preferences_save,
                interface_commands::interfaces_read,
                speech_commands::interfaces_speech_voices,
                speech_commands::interfaces_speech_render,
                window_commands::overlay_regions,
                window_commands::overlay_capture,
                window_commands::overlay_game_panel,
                window_commands::overlay_foreground,
                window_commands::overlay_game_attached,
                window_commands::app_minimize,
                window_commands::app_wait_for_game,
                interface_commands::interfaces_save,
                interface_commands::interfaces_remove_unavailable,
                interface_commands::interfaces_preview,
                interface_commands::interfaces_live,
                interface_commands::interfaces_import,
                interface_commands::interfaces_export,
                interface_commands::interfaces_export_file,
                interface_commands::interfaces_export_folder,
                interface_commands::interfaces_import_folder,
                interface_commands::interfaces_validate,
                interface_commands::catalog_search,
                interface_commands::catalog_item,
                window_commands::app_exit,
                localization_commands::localization_read,
                preferences_commands::preferences_read,
                preferences_commands::preferences_save,
            ])
            .setup(|app| {
                let configurations = &app.config().app.windows;

                let window_configuration = configurations
                    .iter()
                    .find(|window| window.label == Surface::Editor.label())
                    .ok_or("Missing application window")?
                    .clone();

                let monitor = app.primary_monitor()?.ok_or("Missing primary monitor")?;

                let viewport = Viewport {
                    width: f64::from(monitor.size().width) / monitor.scale_factor(),
                    height: f64::from(monitor.size().height) / monitor.scale_factor(),
                };

                let configuration =
                    ApplicationConfiguration::load().map_err(std::io::Error::other)?;

                let directory = configuration
                    .storage_directory(app.path().app_data_dir()?)
                    .map_err(std::io::Error::other)?;

                let mut packs = configuration.packs().map_err(std::io::Error::other)?;

                let manifest = CatalogManifest::parse(configuration.manifest())
                    .map_err(std::io::Error::other)?;

                let version = manifest.version();

                let warnings = LanguageRepository::load(
                    &directory.join(configuration.localization().directory()),
                    &mut packs,
                    configuration.localization().package(),
                    version,
                );

                let zones = configuration.zones().map_err(std::io::Error::other)?;

                let zone_values = zones.values();

                let locales = Arc::new(
                    LocaleRegistry::new(
                        packs,
                        configuration.localization().fallback_locale().into(),
                        configuration.localization().default_locale().into(),
                        warnings,
                        zone_values,
                    )
                    .map_err(std::io::Error::other)?,
                );

                app.manage(Arc::new(SpeechService::new(Arc::new(
                    WindowsSpeechEngine::new(),
                ))));
                app.manage(locales.clone());
                app.manage(Mutex::new(PreferencesRepository::new(
                    directory.join(configuration.storage().preferences()),
                    locales.default_locale().to_owned(),
                )));

                app.manage(Arc::new(Mutex::new(JournalRepository::new(
                    directory.join(configuration.storage().journal()),
                ))));

                app.manage(Arc::new(Mutex::new(JournalArchive::new(
                    directory.join("combat-reports.json"),
                ))));

                let repository =
                    InterfaceRepository::new(directory.join(configuration.storage().profiles()));

                let catalog = CatalogStore::load(
                    configuration.manifest(),
                    configuration.player(),
                    configuration.enemy(),
                    configuration.catalog(),
                    locales.clone(),
                    Arc::new(WindowsTextMatcher::new()),
                    configuration
                        .runtime_catalog()
                        .map_err(std::io::Error::other)?,
                )
                .map_err(std::io::Error::other)?;

                let names = configuration.skill_names().map_err(std::io::Error::other)?;
                let plan = IconManifest::plan(
                    configuration.icon_manifest(),
                    configuration.catalog().client_build(),
                    &catalog.icon_paths(),
                )
                .map_err(std::io::Error::other)?;
                let icons = plan
                    .iter()
                    .flat_map(|download| download.files.iter().cloned())
                    .collect::<Vec<_>>();
                let icon_cache = Arc::new(IconCache::new(
                    directory.join("icon-cache"),
                    configuration.catalog().client_build(),
                ));
                app.manage(icon_cache.clone());

                app.manage(catalog.labels(&names, &icons, &zones));

                let mut interfaces = InterfaceService::new(repository, catalog, viewport, zones);

                interfaces.start();

                app.manage(Mutex::new(interfaces));

                let overlay = OverlayWindow::new(
                    window_configuration,
                    configuration.capture().process_name().to_owned(),
                );

                let tray = Tray::new(
                    app.config().product_name.clone().unwrap_or_default(),
                    configuration.tray_icon(),
                );

                overlay.start(app.handle())?;

                tray.start(app.handle())?;

                let locale = app
                    .state::<Mutex<PreferencesRepository>>()
                    .lock()
                    .map_err(|_| std::io::Error::other("Preferences unavailable"))?
                    .read()
                    .map(|preferences| preferences.locale)
                    .unwrap_or_else(|_| locales.default_locale().to_owned());

                Tray::localize(app.handle(), &locale)?;

                let pipeline = PacketPipeline::new(
                    configuration.protocol().map_err(std::io::Error::other)?,
                    configuration.npc_index().map_err(std::io::Error::other)?,
                    configuration
                        .runtime_catalog()
                        .map_err(std::io::Error::other)?,
                );
                app.manage(pipeline.catalog().clone());

                let mut source = GameSource::new(
                    configuration.capture().clone(),
                    pipeline,
                    configuration.combat_idle_ms(),
                    directory.join("game-session.json"),
                );

                source.start().map_err(std::io::Error::other)?;
                app.manage(source.reader());
                app.manage(Mutex::new(source));

                app.manage(overlay);
                app.manage(tray);
                app.manage(Arc::new(PageHeartbeat::new()));
                IconService::start(
                    app.handle().clone(),
                    icon_cache,
                    plan,
                    Arc::new(WindowsIconTransport),
                    directory.join("aurion.log"),
                );

                Ok(())
            })
            .on_window_event(|window, event| {
                if window.label() == Surface::Editor.label()
                    && let tauri::WindowEvent::CloseRequested { api, .. } = event
                {
                    api.prevent_close();
                    QuitRequest::send(window.app_handle());
                }
            })
            .build(tauri::generate_context!());

        match result {
            Ok(application) => application.run(|app, event| {
                if matches!(event, tauri::RunEvent::Exit) {
                    if let Ok(mut source) = app.state::<Mutex<GameSource>>().lock() {
                        source.stop();
                    }

                    app.state::<Arc<IconCache>>().stop();
                    app.state::<Tray>().dispose(app);
                    app.state::<OverlayWindow>().dispose(app);
                }
            }),
            Err(error) => {
                eprintln!("Failed to initialize AION 2 overlay: {error}");
                StartupDialog::show(&error.to_string());
            }
        }
    }
}

#[tauri::command]
fn catalog_labels(
    window: WebviewWindow,
    labels: State<'_, HashMap<String, CatalogLabels>>,
) -> Result<HashMap<String, CatalogLabels>, String> {
    Surface::Editor.require(window.label())?;

    Ok(labels.inner().clone())
}

#[tauri::command]
fn icon_files(
    window: WebviewWindow,
    cache: State<'_, Arc<IconCache>>,
) -> Result<Vec<String>, String> {
    Surface::Editor.require(window.label())?;

    Ok(cache.files())
}

#[tauri::command]
async fn app_release_check(window: WebviewWindow) -> Result<ReleaseInfo, String> {
    Surface::Editor.require(window.label())?;

    tauri::async_runtime::spawn_blocking(WindowsReleases::check)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn app_version(window: WebviewWindow) -> Result<&'static str, String> {
    Surface::Editor.require(window.label())?;

    Ok(env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
fn app_release_open(window: WebviewWindow) -> Result<(), String> {
    Surface::Editor.require(window.label())?;

    WindowsReleases::open()
}
