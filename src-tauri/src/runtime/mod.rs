//! Нативные службы окна, трея, настроек и внешних переводов.

mod icons;
mod localization;
mod preferences;
mod quit_request;
mod tray;
mod tray_action;
mod window;

pub(crate) use self::icons::IconProtocol;
pub(crate) use self::icons::IconService;

pub(crate) use self::localization::LanguageRepository;
pub(crate) use self::localization::localization_commands;
pub(crate) use self::preferences::PreferencesRepository;
pub(crate) use self::preferences::preferences_commands;
pub(crate) use self::quit_request::QuitRequest;
pub(crate) use self::tray::Tray;
pub(crate) use self::window::GameWindow;
pub(crate) use self::window::OverlayWindow;
pub(crate) use self::window::window_commands;
