use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Map, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::config;
use crate::state::AppState;

static ENABLED: AtomicBool = AtomicBool::new(false);
static FULLSCREEN: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TaskbarState {
    enabled: bool,
    fullscreen: bool,
    desktop_visible: bool,
}

fn marker_path() -> PathBuf {
    config::config_path().with_file_name("taskbar.json")
}

fn write_marker(original: u32) {
    let path = marker_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, json!({ "state": original }).to_string());
}

fn read_marker() -> Option<u32> {
    let raw = fs::read_to_string(marker_path()).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value.get("state")?.as_u64().map(|v| v as u32)
}

#[cfg(windows)]
mod sys {
    use windows::Win32::Foundation::{HWND, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::Shell::{SHAppBarMessage, APPBARDATA};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetTopWindow, GetWindow, GetWindowLongW, GetWindowRect,
        GetWindowThreadProcessId, IsWindowVisible, ShowWindow, GWL_STYLE, GW_HWNDNEXT, SW_HIDE,
        SW_SHOWNA,
    };

    const ABM_GETSTATE: u32 = 4;
    const ABM_SETSTATE: u32 = 10;
    pub const ABS_AUTOHIDE: u32 = 1;
    const WS_CAPTION: u32 = 0x00C0_0000;

    fn class_of(hwnd: HWND) -> String {
        let mut buf = [0u16; 128];
        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }

    fn tray_windows() -> Vec<(HWND, bool)> {
        let mut out = Vec::new();
        let mut current = unsafe { GetTopWindow(HWND::default()) }.ok();
        let mut guard = 0;
        while let Some(hwnd) = current {
            guard += 1;
            if guard > 1500 {
                break;
            }
            current = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok();
            let class = class_of(hwnd);
            if class == "Shell_TrayWnd" {
                out.push((hwnd, true));
            } else if class == "Shell_SecondaryTrayWnd" {
                out.push((hwnd, false));
            }
        }
        out
    }

    pub fn primary_tray() -> Option<HWND> {
        tray_windows()
            .into_iter()
            .find(|(_, primary)| *primary)
            .map(|(hwnd, _)| hwnd)
    }

    pub fn appbar_state() -> Option<(HWND, u32)> {
        let hwnd = primary_tray()?;
        let mut data = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            ..Default::default()
        };
        let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) } as u32;
        Some((hwnd, state))
    }

    pub fn set_appbar_state(hwnd: HWND, state: u32) {
        let mut data = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            lParam: LPARAM(state as isize),
            ..Default::default()
        };
        let _ = unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
    }

    pub fn hide_trays() {
        for (hwnd, _) in tray_windows() {
            unsafe {
                if IsWindowVisible(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
            }
        }
    }

    pub fn show_trays() {
        for (hwnd, _) in tray_windows() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOWNA);
            }
        }
    }

    pub fn foreground_fullscreen() -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == HWND::default() || !IsWindowVisible(hwnd).as_bool() {
                return false;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == std::process::id() {
                return false;
            }
            let class = class_of(hwnd);
            if matches!(
                class.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            ) {
                return false;
            }
            if (GetWindowLongW(hwnd, GWL_STYLE) as u32) & WS_CAPTION == WS_CAPTION {
                return false;
            }
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return false;
            }
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if !GetMonitorInfoW(monitor, &mut info).as_bool() {
                return false;
            }
            let m = info.rcMonitor;
            rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
        }
    }
}

#[cfg(not(windows))]
mod sys {
    pub const ABS_AUTOHIDE: u32 = 1;
    pub fn primary_tray() -> Option<()> {
        None
    }
    pub fn appbar_state() -> Option<((), u32)> {
        None
    }
    pub fn set_appbar_state(_hwnd: (), _state: u32) {}
    pub fn hide_trays() {}
    pub fn show_trays() {}
    pub fn foreground_fullscreen() -> bool {
        false
    }
}

pub fn wants_top() -> bool {
    ENABLED.load(Ordering::Relaxed) && !FULLSCREEN.load(Ordering::Relaxed)
}

fn restore_system() {
    let original = read_marker();
    if let (Some(state), Some(hwnd)) = (original, sys::primary_tray()) {
        sys::set_appbar_state(hwnd, state);
    }
    sys::show_trays();
    let _ = fs::remove_file(marker_path());
}

pub fn recover() {
    if marker_path().exists() {
        restore_system();
    }
}

pub fn restore_on_exit() {
    if ENABLED.swap(false, Ordering::Relaxed) {
        GENERATION.fetch_add(1, Ordering::Relaxed);
        FULLSCREEN.store(false, Ordering::Relaxed);
        restore_system();
    }
}

fn refit_later(app: &AppHandle) {
    let app = app.clone();
    let _ = std::thread::Builder::new()
        .name("dokiii-taskbar-fit".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            if let Some(window) = app.get_webview_window("main") {
                crate::display::fit_window_to_work_area(&app, &window);
            }
            crate::cursor::resync();
        });
}

fn apply_top(app: &AppHandle) {
    let modal = app.state::<AppState>().modal.load(Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(modal || crate::snow::front() || wants_top());
    }
    crate::cursor::resync();
}

fn run(app: AppHandle, generation: u64) {
    let mut last: Option<TaskbarState> = None;
    let mut last_top: Option<bool> = None;
    let mut streak = 0u32;
    while ENABLED.load(Ordering::Relaxed) && GENERATION.load(Ordering::Relaxed) == generation {
        sys::hide_trays();
        let seen = sys::foreground_fullscreen();
        if seen != FULLSCREEN.load(Ordering::Relaxed) {
            streak += 1;
            if streak >= 2 {
                FULLSCREEN.store(seen, Ordering::Relaxed);
                streak = 0;
            }
        } else {
            streak = 0;
        }
        let top = wants_top();
        if last_top != Some(top) {
            apply_top(&app);
            last_top = Some(top);
        }
        let state = TaskbarState {
            enabled: true,
            fullscreen: FULLSCREEN.load(Ordering::Relaxed),
            desktop_visible: crate::snow::desktop_visible(&app),
        };
        if last.as_ref() != Some(&state) {
            let _ = app.emit("taskbar:state", state.clone());
            last = Some(state);
        }
        std::thread::sleep(Duration::from_millis(400));
    }
}

pub fn set_enabled(app: &AppHandle, enabled: bool) {
    if enabled == ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if !enabled {
        ENABLED.store(false, Ordering::Relaxed);
        GENERATION.fetch_add(1, Ordering::Relaxed);
        FULLSCREEN.store(false, Ordering::Relaxed);
        restore_system();
        apply_top(app);
        let _ = app.emit(
            "taskbar:state",
            TaskbarState {
                enabled: false,
                fullscreen: false,
                desktop_visible: true,
            },
        );
        refit_later(app);
        return;
    }
    let (hwnd, original) = match sys::appbar_state() {
        Some(found) => found,
        None => return,
    };
    if read_marker().is_none() {
        write_marker(original);
    }
    sys::set_appbar_state(hwnd, original | sys::ABS_AUTOHIDE);
    sys::hide_trays();
    ENABLED.store(true, Ordering::Relaxed);
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let worker = app.clone();
    let _ = std::thread::Builder::new()
        .name("dokiii-taskbar".into())
        .spawn(move || run(worker, generation));
    refit_later(app);
}

pub fn start_from_config(app: &AppHandle) {
    recover();
    let wanted = {
        let state = app.state::<AppState>();
        let cfg = match state.config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cfg.get("dock")
            .and_then(|dock| dock.get("replaceTaskbar"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    };
    if wanted {
        set_enabled(app, true);
    }
}

pub fn disable_and_persist(app: &AppHandle) {
    set_enabled(app, false);
    {
        let state = app.state::<AppState>();
        let mut cfg = match state.config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let dock = cfg
            .entry("dock".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(map) = dock {
            map.insert("replaceTaskbar".to_string(), Value::Bool(false));
        }
        let _ = config::write(&cfg);
    }
    let _ = app.emit("dock:configChanged", json!({ "replaceTaskbar": false }));
}

#[tauri::command]
pub fn set_replace_taskbar(app: AppHandle, enabled: bool) {
    set_enabled(&app, enabled);
}
