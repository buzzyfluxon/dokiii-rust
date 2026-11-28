use std::time::Duration;

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::state::AppState;

#[cfg(windows)]
fn work_area(x: i32, y: i32) -> Option<(i32, i32, u32, u32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };

    unsafe {
        let monitor = MonitorFromPoint(POINT { x: x + 1, y: y + 1 }, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return None;
        }
        let rect = info.rcWork;
        let width = (rect.right - rect.left).max(1) as u32;
        let height = (rect.bottom - rect.top).max(1) as u32;
        Some((rect.left, rect.top, width, height))
    }
}

#[cfg(not(windows))]
fn work_area(_x: i32, _y: i32) -> Option<(i32, i32, u32, u32)> {
    None
}

fn target_area(app: &AppHandle) -> Option<(i32, i32, u32, u32)> {
    let monitors = app.available_monitors().unwrap_or_default();
    let index = app.state::<AppState>().default_monitor();
    let monitor = monitors.get(index).or_else(|| monitors.first())?;
    let position = monitor.position();
    let size = monitor.size();
    Some(
        work_area(position.x, position.y)
            .unwrap_or((position.x, position.y, size.width, size.height)),
    )
}

pub fn fit_window_to_work_area(app: &AppHandle, window: &WebviewWindow) {
    if let Some((x, y, width, height)) = target_area(app) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
        let _ = window.set_size(PhysicalSize::new(width, height));
    }
}

pub fn start_work_area_watcher(app: AppHandle) {
    let _ = std::thread::Builder::new()
        .name("dokiii-work-area".into())
        .spawn(move || {
            let mut last = target_area(&app);
            loop {
                std::thread::sleep(Duration::from_millis(1500));
                let current = target_area(&app);
                if current != last {
                    if let Some(window) = app.get_webview_window("main") {
                        fit_window_to_work_area(&app, &window);
                    }
                    last = current;
                }
            }
        });
}
