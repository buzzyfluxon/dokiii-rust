use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

#[derive(Clone, Serialize)]
struct CursorPayload {
    x: f64,
    y: f64,
    down: bool,
}

#[cfg(windows)]
fn cursor_position() -> Option<(i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok().map(|_| (point.x, point.y)) }
}

#[cfg(not(windows))]
fn cursor_position() -> Option<(i32, i32)> {
    None
}

#[cfg(windows)]
fn primary_button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};

    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

#[cfg(not(windows))]
fn primary_button_down() -> bool {
    false
}

static RESYNC: AtomicBool = AtomicBool::new(false);

pub fn resync() {
    RESYNC.store(true, Ordering::Relaxed);
}

pub fn start(app: AppHandle) {
    let _ = std::thread::Builder::new()
        .name("dokiii-cursor".into())
        .spawn(move || run(app));
}

struct Geometry {
    visible: bool,
    position: tauri::PhysicalPosition<i32>,
    size: tauri::PhysicalSize<u32>,
    scale: f64,
    read_at: Instant,
}

fn read_geometry(window: &tauri::WebviewWindow) -> Option<Geometry> {
    Some(Geometry {
        visible: window.is_visible().unwrap_or(false),
        position: window.outer_position().ok()?,
        size: window.inner_size().ok()?,
        scale: window.scale_factor().ok()?,
        read_at: Instant::now(),
    })
}

fn run(app: AppHandle) {
    let mut last = (i32::MIN, i32::MIN);
    let mut last_down = false;
    let mut last_move = Instant::now();
    let mut capturing: Option<bool> = None;
    let mut geometry: Option<Geometry> = None;

    loop {
        let idle = last_move.elapsed() > Duration::from_millis(1500);
        std::thread::sleep(Duration::from_millis(if idle { 50 } else { 16 }));

        if RESYNC.swap(false, Ordering::Relaxed) {
            capturing = None;
            geometry = None;
        }

        let window = match app.get_webview_window("main") {
            Some(window) => window,
            None => continue,
        };
        let stale = geometry
            .as_ref()
            .map(|g| g.read_at.elapsed() > Duration::from_millis(1000))
            .unwrap_or(true);
        if stale {
            geometry = read_geometry(&window);
        }
        let (position, size, scale) = match geometry.as_ref() {
            Some(g) if g.visible => (g.position, g.size, g.scale),
            _ => continue,
        };
        let (cx, cy) = match cursor_position() {
            Some(point) => point,
            None => continue,
        };

        let inside = cx >= position.x
            && cy >= position.y
            && cx < position.x + size.width as i32
            && cy < position.y + size.height as i32;
        let state = app.state::<AppState>();

        let down = primary_button_down();
        let moved = (cx, cy) != last;
        if moved || down != last_down {
            last_down = down;
            if moved {
                last = (cx, cy);
                last_move = Instant::now();
                crate::system::touch_activity();
            }
            if inside {
                let payload = CursorPayload {
                    x: (cx - position.x) as f64 / scale,
                    y: (cy - position.y) as f64 / scale,
                    down,
                };
                let _ = window.emit("dokiii://cursor", payload);
            } else if moved {
                state.js_hit.store(false, Ordering::Relaxed);
            }
        }

        let capture_all = state.modal.load(Ordering::Relaxed) || state.popover.load(Ordering::Relaxed);
        let wanted = capture_all || (inside && state.js_hit.load(Ordering::Relaxed));
        let holding = capturing == Some(true) && primary_button_down();
        let next = holding || wanted;

        if capturing != Some(next) {
            crate::debuglog::log(&format!(
                "capture={} cursor={},{} win={},{} {}x{} scale={} js_hit={} modal={} popover={} top={}",
                next,
                cx,
                cy,
                position.x,
                position.y,
                size.width,
                size.height,
                scale,
                state.js_hit.load(Ordering::Relaxed),
                state.modal.load(Ordering::Relaxed),
                state.popover.load(Ordering::Relaxed),
                crate::taskbar::wants_top()
            ));
            capturing = Some(next);
            let _ = window.set_ignore_cursor_events(!next);
        }
    }
}
