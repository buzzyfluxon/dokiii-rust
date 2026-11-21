use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static FRONT: AtomicBool = AtomicBool::new(false);

pub fn front() -> bool {
    FRONT.load(Ordering::Relaxed)
}

#[derive(Serialize, Clone)]
pub struct WindowEdge {
    id: String,
    x: i32,
    y: i32,
    w: i32,
    spans: Vec<[i32; 2]>,
}

#[derive(Serialize, Clone)]
pub struct DesktopState {
    visible: bool,
    windows: Vec<WindowEdge>,
}

#[derive(Clone, Copy)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

struct Tracked {
    id: String,
    rect: Rect,
    zoomed: bool,
}

#[cfg(windows)]
fn list_windows() -> Vec<Tracked> {
    use std::ffi::c_void;
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetTopWindow, GetWindow, GetWindowLongW, GetWindowRect, GetWindowTextLengthW,
        GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed, GWL_EXSTYLE, GW_HWNDNEXT,
    };

    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
    const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
    const IGNORED: [&str; 9] = [
        "Progman",
        "WorkerW",
        "Shell_TrayWnd",
        "Shell_SecondaryTrayWnd",
        "NotifyIconOverflowWindow",
        "TopLevelWindowForOverflowXamlIsland",
        "XamlExplorerHostIslandWindow",
        "Windows.UI.Core.CoreWindow",
        "ApplicationFrameInputSinkWindow",
    ];

    let own_pid = std::process::id();
    let mut out = Vec::new();
    let mut current = unsafe { GetTopWindow(HWND::default()) }.ok();
    let mut guard = 0;

    while let Some(hwnd) = current {
        guard += 1;
        if guard > 600 {
            break;
        }
        current = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok();

        unsafe {
            if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                continue;
            }
            let mut cloaked = 0u32;
            let cloak_ok = DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut u32 as *mut c_void,
                std::mem::size_of::<u32>() as u32,
            )
            .is_ok();
            if cloak_ok && cloaked != 0 {
                continue;
            }
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            if ex_style & (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT) != 0 {
                continue;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == own_pid {
                continue;
            }
            if GetWindowTextLengthW(hwnd) <= 0 {
                continue;
            }
            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(hwnd, &mut class_buf);
            if len > 0 {
                let class = String::from_utf16_lossy(&class_buf[..len as usize]);
                if IGNORED.contains(&class.as_str()) {
                    continue;
                }
            }
            let mut rect = RECT::default();
            let framed = DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut rect as *mut RECT as *mut c_void,
                std::mem::size_of::<RECT>() as u32,
            )
            .is_ok();
            if !framed && GetWindowRect(hwnd, &mut rect).is_err() {
                continue;
            }
            if rect.right - rect.left < 120 || rect.bottom - rect.top < 60 {
                continue;
            }
            out.push(Tracked {
                id: (hwnd.0 as isize).to_string(),
                rect: Rect {
                    left: rect.left as f64,
                    top: rect.top as f64,
                    right: rect.right as f64,
                    bottom: rect.bottom as f64,
                },
                zoomed: IsZoomed(hwnd).as_bool(),
            });
        }
    }
    out
}

#[cfg(not(windows))]
fn list_windows() -> Vec<Tracked> {
    Vec::new()
}

fn subtract(spans: Vec<(f64, f64)>, from: f64, to: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for (a, b) in spans {
        if to <= a || from >= b {
            out.push((a, b));
            continue;
        }
        if from > a {
            out.push((a, from));
        }
        if to < b {
            out.push((to, b));
        }
    }
    out
}

fn compute(app: &AppHandle) -> DesktopState {
    let hidden = DesktopState {
        visible: false,
        windows: Vec::new(),
    };
    let window = match app.get_webview_window("main") {
        Some(window) => window,
        None => return hidden,
    };
    if !window.is_visible().unwrap_or(false) || window.is_minimized().unwrap_or(false) {
        return hidden;
    }
    let (position, size, scale) = match (window.outer_position(), window.inner_size(), window.scale_factor()) {
        (Ok(position), Ok(size), Ok(scale)) => (position, size, scale),
        _ => {
            return DesktopState {
                visible: true,
                windows: Vec::new(),
            }
        }
    };

    let width = size.width as f64 / scale;
    let height = size.height as f64 / scale;
    let area = width * height;
    let mut local: Vec<(String, Rect)> = Vec::new();

    for tracked in list_windows() {
        let r = Rect {
            left: (tracked.rect.left - position.x as f64) / scale,
            top: (tracked.rect.top - position.y as f64) / scale,
            right: (tracked.rect.right - position.x as f64) / scale,
            bottom: (tracked.rect.bottom - position.y as f64) / scale,
        };
        let ix = (r.right.min(width) - r.left.max(0.0)).max(0.0);
        let iy = (r.bottom.min(height) - r.top.max(0.0)).max(0.0);
        if ix <= 0.0 || iy <= 0.0 {
            continue;
        }
        let threshold = if tracked.zoomed { 0.55 } else { 0.9 };
        if ix * iy >= area * threshold {
            return hidden;
        }
        local.push((tracked.id, r));
    }

    let mut edges = Vec::new();
    for i in 0..local.len() {
        let r = local[i].1;
        if r.top < 24.0 || r.top > height - 60.0 {
            continue;
        }
        let mut spans = vec![(r.left.max(0.0), r.right.min(width))];
        for j in 0..i {
            let above = local[j].1;
            if above.top <= r.top && above.bottom > r.top {
                spans = subtract(spans, above.left, above.right);
            }
            if spans.is_empty() {
                break;
            }
        }
        let spans: Vec<[i32; 2]> = spans
            .into_iter()
            .filter(|(a, b)| b - a >= 40.0)
            .map(|(a, b)| [a.round() as i32, b.round() as i32])
            .collect();
        if spans.is_empty() {
            continue;
        }
        edges.push(WindowEdge {
            id: local[i].0.clone(),
            x: r.left.round() as i32,
            y: r.top.round() as i32,
            w: (r.right - r.left).round() as i32,
            spans,
        });
    }

    DesktopState {
        visible: true,
        windows: edges,
    }
}

fn run(app: AppHandle, generation: u64) {
    let mut last_payload = String::new();
    let mut last_change = Instant::now();
    while ACTIVE.load(Ordering::Relaxed) && GENERATION.load(Ordering::Relaxed) == generation {
        let state = compute(&app);
        if let Ok(payload) = serde_json::to_string(&state) {
            if payload != last_payload {
                last_payload = payload;
                last_change = Instant::now();
                let _ = app.emit("snow:desktopState", state);
            }
        }
        let delay = if last_change.elapsed() < Duration::from_millis(1500) { 16 } else { 250 };
        std::thread::sleep(Duration::from_millis(delay));
    }
}

#[tauri::command]
pub fn set_snow_active(app: AppHandle, active: bool) {
    if active == ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    ACTIVE.store(active, Ordering::Relaxed);
    if !active {
        GENERATION.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = std::thread::Builder::new()
        .name("dokiii-snow".into())
        .spawn(move || run(app, generation));
}

#[tauri::command]
pub fn set_snow_front(app: AppHandle, front: bool) {
    FRONT.store(front, Ordering::Relaxed);
    let modal = app.state::<crate::state::AppState>().modal.load(Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(front || modal);
    }
}

#[tauri::command]
pub fn clear_snow(app: AppHandle) {
    let _ = app.emit_to("main", "snow:clear", ());
}
