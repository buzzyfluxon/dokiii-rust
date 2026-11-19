use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{icon, lnk, shellpaths};

#[derive(Serialize)]
pub struct AppInfo {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

#[derive(Serialize)]
pub struct SelectedApp {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

fn expand_env(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                match std::env::var(name) {
                    Ok(value) => out.push_str(&value),
                    Err(_) => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn walk(dir: &Path, found: &mut Vec<(String, String, String)>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        if meta.is_dir() {
            walk(&path, found);
            continue;
        }
        let is_lnk = path
            .extension()
            .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("lnk"))
            .unwrap_or(false);
        if !is_lnk {
            continue;
        }
        let shortcut = path.to_string_lossy().to_string();
        let target = lnk::target_of(&shortcut)
            .map(|t| expand_env(&t))
            .unwrap_or_else(|| shortcut.clone());
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default();
        found.push((name, target, shortcut));
    }
}

fn excluded(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.contains("uninstall") {
        return true;
    }
    lower.contains("readme") || lower.contains("read me")
}

pub fn icon_for(path: &str) -> Option<String> {
    icon::file_icon_data_url(path, 128)
}

#[tauri::command]
pub async fn get_installed_apps() -> Vec<AppInfo> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut roots: Vec<PathBuf> = vec![shellpaths::start_menu_common()];
        if let Some(appdata) = std::env::var_os("APPDATA") {
            roots.push(
                PathBuf::from(appdata)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs"),
            );
        }
        let mut found = Vec::new();
        for root in roots {
            walk(&root, &mut found);
        }

        let mut unique: Vec<(String, String, String)> = Vec::new();
        for item in found {
            if let Some(existing) = unique.iter_mut().find(|u| u.0 == item.0) {
                *existing = item;
            } else {
                unique.push(item);
            }
        }
        unique.retain(|item| !excluded(&item.0));
        unique.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        unique
            .into_iter()
            .map(|(name, target, shortcut)| {
                let icon = icon_for(&shortcut).or_else(|| {
                    if target.is_empty() {
                        None
                    } else {
                        icon_for(&target)
                    }
                });
                AppInfo {
                    name,
                    path: target,
                    icon,
                }
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn get_app_icon(file_path: String) -> Option<String> {
    if file_path.is_empty() {
        return None;
    }
    tauri::async_runtime::spawn_blocking(move || icon_for(&file_path))
        .await
        .unwrap_or(None)
}

#[tauri::command]
pub async fn select_app_file() -> Option<SelectedApp> {
    tauri::async_runtime::spawn_blocking(|| {
        let picked = rfd::FileDialog::new()
            .set_title("Select Application to Add to Dock")
            .add_filter("Applications", &["exe", "lnk", "bat", "cmd"])
            .add_filter("All Files", &["*"])
            .pick_file()?;
        let path = picked.to_string_lossy().to_string();
        let name = picked
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default();
        let icon = icon_for(&path);
        Some(SelectedApp { name, path, icon })
    })
    .await
    .unwrap_or(None)
}

#[tauri::command]
pub async fn select_image_file() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| {
        let picked = rfd::FileDialog::new()
            .set_title("Select Image")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
            .pick_file()?;
        Some(picked.to_string_lossy().to_string())
    })
    .await
    .unwrap_or(None)
}

#[tauri::command]
pub async fn pick_icon_image() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| {
        let picked = rfd::FileDialog::new()
            .set_title("Select Icon")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp", "ico"])
            .pick_file()?;
        let bytes = fs::read(picked).ok()?;
        icon::bytes_to_icon_data_url(&bytes)
    })
    .await
    .unwrap_or(None)
}
