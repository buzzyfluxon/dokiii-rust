use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const LABEL: &str = "control-center";

static PENDING_TAB: Mutex<String> = Mutex::new(String::new());

fn set_tab(tab: &str) {
    let mut guard = match PENDING_TAB.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = tab.to_string();
}

#[tauri::command]
pub async fn open_control_center(app: AppHandle, tab: Option<String>) -> Result<(), String> {
    let tab = tab.unwrap_or_else(|| "home".to_string());
    set_tab(&tab);

    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("cc:tab", tab);
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("index.html".into()))
        .title("DOKIII Control Center")
        .initialization_script("window.__DOKIII_WINDOW__ = 'control-center';")
        .inner_size(1040.0, 680.0)
        .min_inner_size(820.0, 520.0)
        .center()
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(true)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

    // Safety net: if the page never reports ready, still show the window.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(4000));
        if !window.is_visible().unwrap_or(true) {
            let _ = window.show();
            let _ = window.set_focus();
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cc_ready(window: WebviewWindow) -> String {
    let _ = window.show();
    let _ = window.set_focus();
    match PENDING_TAB.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

#[tauri::command]
pub fn cc_close(window: WebviewWindow) {
    let _ = window.close();
}

#[tauri::command]
pub fn cc_minimize(window: WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn cc_toggle_maximize(window: WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub fn cc_drag(window: WebviewWindow) {
    let _ = window.start_dragging();
}
