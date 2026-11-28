use std::sync::atomic::Ordering;

use serde::Serialize;
use serde_json::{Map, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::config;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    index: usize,
    id: usize,
    is_primary: bool,
    width: f64,
    height: f64,
}

#[tauri::command]
pub fn config_get(state: State<AppState>) -> Value {
    let cfg = match state.config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    Value::Object(cfg.clone())
}

#[tauri::command]
pub fn config_save(
    app: AppHandle,
    window: tauri::WebviewWindow,
    state: State<AppState>,
    config: Map<String, Value>,
) -> Result<(), String> {
    let mut cfg = match state.config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    for (key, value) in config {
        cfg.insert(key, value);
    }
    let result = config::write(&cfg).map_err(|e| e.to_string());
    drop(cfg);
    for (label, _other) in app.webview_windows() {
        if label != window.label() {
            let _ = app.emit_to(label.as_str(), "config:synced", ());
        }
    }
    result
}

#[derive(Serialize)]
pub struct DesktopSize {
    width: f64,
    height: f64,
}

/// Size of the transparent desktop layer in CSS pixels. The Control Center window has its own,
/// much smaller size, so it asks for this when it needs to place a widget on the real desktop.
#[tauri::command]
pub fn get_desktop_size(app: AppHandle) -> Option<DesktopSize> {
    let window = app.get_webview_window("main")?;
    let size = window.inner_size().ok()?;
    let scale = window.scale_factor().unwrap_or(1.0).max(0.1);
    Some(DesktopSize {
        width: (size.width as f64 / scale).round(),
        height: (size.height as f64 / scale).round(),
    })
}

/// Hands unused memory of the app and its WebView processes back to Windows.
#[tauri::command]
pub fn trim_memory() {
    crate::system::trim_soon(600);
}

#[tauri::command]
pub fn get_displays(app: AppHandle) -> Vec<DisplayInfo> {
    let primary = app.primary_monitor().ok().flatten();
    app.available_monitors()
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(index, monitor)| {
            let scale = monitor.scale_factor();
            let is_primary = primary
                .as_ref()
                .map(|p| p.position() == monitor.position())
                .unwrap_or(index == 0);
            DisplayInfo {
                index,
                id: index,
                is_primary,
                width: monitor.size().width as f64 / scale,
                height: monitor.size().height as f64 / scale,
            }
        })
        .collect()
}

#[tauri::command]
pub fn report_hit(state: State<AppState>, hit: bool) {
    state.js_hit.store(hit, Ordering::Relaxed);
}

#[tauri::command]
pub fn set_modal_open(app: AppHandle, state: State<AppState>, open: bool) {
    state.modal.store(open, Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(open || crate::snow::front() || crate::taskbar::wants_top());
        if open {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[tauri::command]
pub fn set_popover_open(state: State<AppState>, open: bool) {
    state.popover.store(open, Ordering::Relaxed);
}

#[tauri::command]
pub fn app_ready(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
    }
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    crate::taskbar::restore_on_exit();
    app.exit(0);
}

#[tauri::command]
pub fn show_widget_library(app: AppHandle) {
    let _ = app.emit("dock:showWidgetLibrary", ());
}

#[tauri::command]
pub fn show_settings(app: AppHandle) {
    let _ = app.emit("dock:showSettings", ());
}

#[tauri::command]
pub fn set_auto_hide(app: AppHandle, state: State<AppState>, value: bool) -> Result<(), String> {
    {
        let mut cfg = match state.config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let dock = cfg
            .entry("dock".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(map) = dock {
            map.insert("autoHide".to_string(), Value::Bool(value));
        }
        config::write(&cfg).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("dock:configChanged", serde_json::json!({ "autoHide": value }));
    Ok(())
}
