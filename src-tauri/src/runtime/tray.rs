use super::tray_action::TrayAction;
use crate::core::localization::LocaleRegistry;
use crate::core::surface::Surface;
use crate::runtime::QuitRequest;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) struct Tray {
    title: String,
    icon: &'static [u8],
}

impl Tray {
    const ID: &str = "overlay-tray";

    pub(crate) fn new(title: String, icon: &'static [u8]) -> Self {
        Self { title, icon }
    }

    pub(crate) fn start(&self, app: &AppHandle) -> tauri::Result<()> {
        let menu = Self::menu(app, app.state::<Arc<LocaleRegistry>>().default_locale())?;

        TrayIconBuilder::with_id(Self::ID)
            .icon(tauri::image::Image::from_bytes(self.icon)?)
            .tooltip(&self.title)
            .menu(&menu)
            .on_menu_event(|app, event| {
                let Some(action) = TrayAction::parse(event.id.as_ref()) else {
                    return;
                };

                if let Some(window) = app.get_webview_window(Surface::Editor.label()) {
                    let _ = window.unminimize();

                    let _ = window.show();

                    let _ = window.set_focus();
                }

                match action {
                    TrayAction::Quit => QuitRequest::send(app),
                    TrayAction::Editor => {
                        let _ =
                            app.emit_to(Surface::Editor.label(), "interfaces:open-requested", ());
                    }
                }
            })
            .build(app)?;

        Ok(())
    }

    fn menu(app: &AppHandle, locale: &str) -> tauri::Result<Menu<tauri::Wry>> {
        let messages = app.state::<Arc<LocaleRegistry>>();

        let items = TrayAction::ALL
            .into_iter()
            .map(|action| {
                MenuItem::with_id(
                    app,
                    action.id(),
                    messages.text(locale, action.id()),
                    true,
                    None::<&str>,
                )
            })
            .collect::<tauri::Result<Vec<_>>>()?;

        let items = items
            .iter()
            .map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
            .collect::<Vec<_>>();

        Menu::with_items(app, &items)
    }

    pub(crate) fn localize(app: &AppHandle, locale: &str) -> tauri::Result<()> {
        if let Some(tray) = app.tray_by_id(Self::ID) {
            tray.set_menu(Some(Self::menu(app, locale)?))?;
        }

        if let Some(window) = app.get_webview_window(Surface::Editor.label()) {
            window.set_title(
                &app.state::<Arc<LocaleRegistry>>()
                    .text(locale, "windowTitle"),
            )?;
        }

        Ok(())
    }

    pub(crate) fn dispose(&self, app: &AppHandle) {
        app.remove_tray_by_id(Self::ID);
    }
}
