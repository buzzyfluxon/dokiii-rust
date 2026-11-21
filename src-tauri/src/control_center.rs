use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::window::{Color, Effect, EffectsBuilder};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const LABEL: &str = "control-center";
const KEEP_ALIVE_SECS: u64 = 180;

static PENDING_TAB: Mutex<String> = Mutex::new(String::new());
static WANT_OPEN: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
pub struct ReadyInfo {
    tab: String,
    open: bool,
}

fn set_tab(tab: &str) {
    let mut guard = match PENDING_TAB.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = tab.to_string();
}

fn get_tab() -> String {
    match PENDING_TAB.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

fn cancel_destroy() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

// A hidden window is kept alive for a few minutes so reopening is instant,
// then destroyed to give the memory back.
fn schedule_destroy(app: AppHandle) {
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(KEEP_ALIVE_SECS));
        if GENERATION.load(Ordering::SeqCst) == generation && !WANT_OPEN.load(Ordering::SeqCst) {
            if let Some(window) = app.get_webview_window(LABEL) {
                let _ = window.close();
            }
        }
    });
}

fn build(app: &AppHandle) -> Result<WebviewWindow, String> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("DOKIII Control Center")
        .initialization_script("window.__DOKIII_WINDOW__ = 'control-center';")
        .inner_size(1040.0, 680.0)
        .min_inner_size(820.0, 520.0)
        .center()
        .decorations(false)
        .transparent(true)
        .effects(
            EffectsBuilder::new()
                .effect(Effect::Acrylic)
                .color(Color(18, 18, 22, 150))
                .build(),
        )
        .shadow(true)
        .resizable(true)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())
}

fn safety_show(window: WebviewWindow, after_ms: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(after_ms));
        if WANT_OPEN.load(Ordering::SeqCst) && !window.is_visible().unwrap_or(true) {
            let _ = window.show();
            let _ = window.set_focus();
        }
    });
}

#[tauri::command]
pub async fn prewarm_control_center(app: AppHandle) -> Result<(), String> {
    if app.get_webview_window(LABEL).is_some() {
        if !WANT_OPEN.load(Ordering::SeqCst) {
            schedule_destroy(app);
        }
        return Ok(());
    }
    build(&app)?;
    schedule_destroy(app);
    Ok(())
}

#[tauri::command]
pub async fn open_control_center(app: AppHandle, tab: Option<String>) -> Result<(), String> {
    let tab = tab.unwrap_or_else(|| "home".to_string());
    set_tab(&tab);
    WANT_OPEN.store(true, Ordering::SeqCst);
    cancel_destroy();

    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = app.emit_to(LABEL, "cc:open", tab);
        safety_show(window, 2500);
        return Ok(());
    }

    let window = build(&app)?;
    safety_show(window, 5000);
    Ok(())
}

#[tauri::command]
pub fn cc_ready() -> ReadyInfo {
    ReadyInfo {
        tab: get_tab(),
        open: WANT_OPEN.load(Ordering::SeqCst),
    }
}

#[tauri::command]
pub fn cc_show(window: WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

#[tauri::command]
pub fn cc_close(window: WebviewWindow) {
    WANT_OPEN.store(false, Ordering::SeqCst);
    let _ = window.hide();
    schedule_destroy(window.app_handle().clone());
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
