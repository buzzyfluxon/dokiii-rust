use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionSample {
    r: i64,
    g: i64,
    b: i64,
    luminance: i64,
    is_light: bool,
    tint_r: i64,
    tint_g: i64,
    tint_b: i64,
    alpha: f64,
    bg_rgba: String,
    border_color: String,
    box_shadow: String,
    highlight_color: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperPalette {
    dominant: RegionSample,
    dock_bottom: RegionSample,
    dock_left: RegionSample,
    dock_right: RegionSample,
    halo: RegionSample,
    desktop_widgets: RegionSample,
    modal: RegionSample,
    timestamp: u64,
}

struct Cache {
    modified: Option<SystemTime>,
    palette: Option<WallpaperPalette>,
}

static CACHE: Mutex<Cache> = Mutex::new(Cache {
    modified: None,
    palette: None,
});

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn clamp_i(value: f64, min: f64, max: f64) -> i64 {
    value.max(min).min(max).round() as i64
}

fn derive_glass_style(avg_r: f64, avg_g: f64, avg_b: f64, is_modal: bool) -> RegionSample {
    let luminance = (0.299 * avg_r + 0.587 * avg_g + 0.114 * avg_b).round() as i64;
    let is_light = luminance > 140;

    let (base_r, base_g, base_b) = if is_light { (24.0, 26.0, 32.0) } else { (14.0, 15.0, 18.0) };
    let blend = if is_light { 0.22 } else { 0.18 };
    let tint_r = clamp_i(base_r * (1.0 - blend) + avg_r * blend, 0.0, 255.0);
    let tint_g = clamp_i(base_g * (1.0 - blend) + avg_g * blend, 0.0, 255.0);
    let tint_b = clamp_i(base_b * (1.0 - blend) + avg_b * blend, 0.0, 255.0);

    let alpha = if is_modal {
        if is_light { 0.86 } else { 0.82 }
    } else if is_light {
        0.80
    } else {
        0.74
    };

    let bg_rgba = format!("rgba({}, {}, {}, {:.2})", tint_r, tint_g, tint_b, alpha);

    let border_alpha = if is_light { 0.24 } else { 0.16 };
    let lift = if is_light { 120 } else { 60 };
    let border_color = format!(
        "rgba({}, {}, {}, {:.2})",
        (tint_r + lift).min(255),
        (tint_g + lift).min(255),
        (tint_b + lift).min(255),
        border_alpha
    );

    let highlight_alpha = if is_light { 0.32 } else { 0.22 };
    let highlight_color = format!("rgba(255, 255, 255, {:.2})", highlight_alpha);

    let shadow_alpha = if is_light { 0.52 } else { 0.46 };
    let box_shadow = if is_light {
        format!(
            "0 20px 48px rgba(0, 0, 0, {:.2}), inset 0 1px 0 rgba(255, 255, 255, 0.28), inset 0 -1px 0 rgba(0, 0, 0, 0.35)",
            shadow_alpha
        )
    } else {
        format!(
            "0 16px 44px rgba(0, 0, 0, {:.2}), inset 0 1px 0 rgba(255, 255, 255, 0.20), inset 0 -1px 0 rgba(0, 0, 0, 0.25)",
            shadow_alpha
        )
    };

    RegionSample {
        r: avg_r.round() as i64,
        g: avg_g.round() as i64,
        b: avg_b.round() as i64,
        luminance,
        is_light,
        tint_r,
        tint_g,
        tint_b,
        alpha,
        bg_rgba,
        border_color,
        box_shadow,
        highlight_color,
    }
}

fn fallback_palette() -> WallpaperPalette {
    let normal = derive_glass_style(24.0, 24.0, 28.0, false);
    let modal = derive_glass_style(24.0, 24.0, 28.0, true);
    WallpaperPalette {
        dominant: normal.clone(),
        dock_bottom: normal.clone(),
        dock_left: normal.clone(),
        dock_right: normal.clone(),
        halo: normal.clone(),
        desktop_widgets: normal,
        modal,
        timestamp: now_ms(),
    }
}

fn sample_box(rgba: &[u8], width: u32, sx: u32, ex: u32, sy: u32, ey: u32) -> (f64, f64, f64) {
    let (mut r, mut g, mut b, mut count) = (0f64, 0f64, 0f64, 0f64);
    for y in sy..=ey {
        for x in sx..=ex {
            let offset = ((y * width + x) * 4) as usize;
            if offset + 2 < rgba.len() {
                r += rgba[offset] as f64;
                g += rgba[offset + 1] as f64;
                b += rgba[offset + 2] as f64;
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        (24.0, 24.0, 28.0)
    } else {
        (r / count, g / count, b / count)
    }
}

fn wallpaper_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("Microsoft")
        .join("Windows")
        .join("Themes")
        .join("TranscodedWallpaper")
}

pub fn compute() -> WallpaperPalette {
    let path = wallpaper_path();
    let modified = match fs::metadata(&path).and_then(|meta| meta.modified()) {
        Ok(modified) => modified,
        Err(_) => return fallback_palette(),
    };
    {
        let cache = match CACHE.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if cache.modified == Some(modified) {
            if let Some(palette) = cache.palette.clone() {
                return palette;
            }
        }
    }

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => return fallback_palette(),
    };
    let decoded = match image::load_from_memory(&bytes) {
        Ok(decoded) => decoded,
        Err(_) => return fallback_palette(),
    };
    let small = decoded
        .resize_exact(64, 36, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let raw = small.as_raw();

    let region = |sx, ex, sy, ey, modal| {
        let (r, g, b) = sample_box(raw, 64, sx, ex, sy, ey);
        derive_glass_style(r, g, b, modal)
    };

    let palette = WallpaperPalette {
        dominant: region(0, 63, 0, 35, false),
        dock_bottom: region(14, 49, 28, 35, false),
        dock_left: region(0, 10, 8, 27, false),
        dock_right: region(53, 63, 8, 27, false),
        halo: region(20, 43, 0, 6, false),
        desktop_widgets: region(46, 63, 2, 33, false),
        modal: region(16, 47, 8, 27, true),
        timestamp: now_ms(),
    };

    let mut cache = match CACHE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    cache.modified = Some(modified);
    cache.palette = Some(palette.clone());
    palette
}

#[tauri::command]
pub async fn get_wallpaper_colors() -> WallpaperPalette {
    tauri::async_runtime::spawn_blocking(compute)
        .await
        .unwrap_or_else(|_| fallback_palette())
}

pub fn start_watcher(app: AppHandle) {
    let _ = std::thread::Builder::new()
        .name("dokiii-wallpaper".into())
        .spawn(move || {
            let path = wallpaper_path();
            let mut last = fs::metadata(&path).and_then(|m| m.modified()).ok();
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let current = fs::metadata(&path).and_then(|m| m.modified()).ok();
                if current != last {
                    last = current;
                    std::thread::sleep(Duration::from_millis(800));
                    let _ = app.emit("wallpaper:colors-updated", compute());
                }
            }
        });
}
