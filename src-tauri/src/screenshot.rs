use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{process, shellpaths};

fn iso_stamp() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = now.as_secs() as i64;
    let millis = now.subsec_millis();
    let days = total_secs.div_euclid(86_400);
    let secs = total_secs.rem_euclid(86_400);

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };

    format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}-{:03}Z",
        year,
        month,
        day,
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60,
        millis
    )
}

#[cfg(windows)]
fn read_bitmap(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    bitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    width: i32,
    height: i32,
) -> Option<Vec<u8>> {
    use std::ffi::c_void;
    use windows::Win32::Graphics::Gdi::{GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS};

    let mut info = BITMAPINFO::default();
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: 0,
        ..Default::default()
    };
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let lines = unsafe {
        GetDIBits(
            hdc,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    if lines == 0 {
        return None;
    }
    for px in pixels.chunks_exact_mut(4) {
        px.swap(0, 2);
        px[3] = 255;
    }
    Some(pixels)
}

#[cfg(windows)]
enum Source {
    Screen { x: i32, y: i32 },
    Window(windows::Win32::Foundation::HWND),
}

#[cfg(windows)]
fn grab(source: Source, width: i32, height: i32) -> Option<Vec<u8>> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, ReleaseDC,
        SelectObject, CAPTUREBLT, HGDIOBJ, SRCCOPY,
    };
    use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};

    if width <= 0 || height <= 0 {
        return None;
    }
    unsafe {
        let screen = GetDC(HWND::default());
        let memory = CreateCompatibleDC(screen);
        let bitmap = CreateCompatibleBitmap(screen, width, height);
        let previous = SelectObject(memory, HGDIOBJ(bitmap.0));

        let drawn = match source {
            Source::Screen { x, y } => BitBlt(memory, 0, 0, width, height, screen, x, y, SRCCOPY | CAPTUREBLT).is_ok(),
            Source::Window(hwnd) => PrintWindow(hwnd, memory, PRINT_WINDOW_FLAGS(2)).as_bool(),
        };
        let pixels = if drawn { read_bitmap(memory, bitmap, width, height) } else { None };

        SelectObject(memory, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(memory);
        ReleaseDC(HWND::default(), screen);
        pixels
    }
}

#[cfg(windows)]
fn capture_full() -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    unsafe {
        let mut point = POINT::default();
        GetCursorPos(&mut point).ok()?;
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return None;
        }
        let rect = info.rcMonitor;
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        let pixels = grab(Source::Screen { x: rect.left, y: rect.top }, width, height)?;
        Some((pixels, width as u32, height as u32))
    }
}

#[cfg(windows)]
fn capture_window() -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == std::process::id() {
            return None;
        }
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        let pixels = grab(Source::Window(hwnd), width, height)?;
        Some((pixels, width as u32, height as u32))
    }
}

#[cfg(not(windows))]
fn capture_full() -> Option<(Vec<u8>, u32, u32)> {
    None
}

#[cfg(not(windows))]
fn capture_window() -> Option<(Vec<u8>, u32, u32)> {
    None
}

fn save_png(path: &PathBuf, pixels: Vec<u8>, width: u32, height: u32) -> bool {
    match image::RgbaImage::from_raw(width, height, pixels) {
        Some(image) => image.save_with_format(path, image::ImageFormat::Png).is_ok(),
        None => false,
    }
}

#[tauri::command]
pub async fn capture_screenshot(mode: String) -> String {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = match shellpaths::pictures() {
            Some(dir) => dir.join("Screenshots"),
            None => return String::new(),
        };
        let _ = fs::create_dir_all(&dir);

        if mode == "region" {
            let _ = process::shell_open("ms-screenclip:");
            return String::new();
        }

        let path = dir.join(format!("Screenshot-{}.png", iso_stamp()));
        let captured = if mode == "window" {
            capture_window().or_else(capture_full)
        } else {
            capture_full()
        };
        match captured {
            Some((pixels, width, height)) => {
                if save_png(&path, pixels, width, height) {
                    path.to_string_lossy().to_string()
                } else {
                    String::new()
                }
            }
            None => String::new(),
        }
    })
    .await
    .unwrap_or_default()
}
