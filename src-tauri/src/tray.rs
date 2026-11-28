use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::commands;
use crate::state::AppState;

fn toggle_dock(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
        }
    }
    let _ = app.emit("dock:toggleVisibility", ());
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let auto_hide_now = {
        let state = app.state::<AppState>();
        let cfg = match state.config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cfg.get("dock")
            .and_then(|dock| dock.get("autoHide"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    };

    let open_app = MenuItem::with_id(app, "open_app", "Open DOKIII App", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle_dock", "Show / Hide Dock", true, None::<&str>)?;
    let library = MenuItem::with_id(app, "widget_library", "Widget Library", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let auto_hide = CheckMenuItem::with_id(app, "auto_hide", "Auto Hide Dock", true, auto_hide_now, None::<&str>)?;
    let restore_taskbar = MenuItem::with_id(app, "restore_taskbar", "Show Windows Taskbar", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit DOKIII", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&open_app, &toggle, &library, &settings, &auto_hide, &restore_taskbar, &separator, &quit],
    )?;

    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
    let auto_hide_item = auto_hide.clone();

    TrayIconBuilder::with_id("dokiii-tray")
        .icon(icon)
        .tooltip("DOKIII")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open_app" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                }
                let _ = app.emit("dock:showApp", ());
            }
            "toggle_dock" => toggle_dock(app),
            "widget_library" => {
                let _ = app.emit("dock:showWidgetLibrary", ());
            }
            "settings" => {
                let _ = app.emit("dock:showSettings", ());
            }
            "auto_hide" => {
                let checked = auto_hide_item.is_checked().unwrap_or(false);
                let _ = commands::set_auto_hide(app.clone(), app.state::<AppState>(), checked);
            }
            "quit" => {
                crate::taskbar::restore_on_exit();
                app.exit(0);
            }
            "restore_taskbar" => crate::taskbar::disable_and_persist(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_dock(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
