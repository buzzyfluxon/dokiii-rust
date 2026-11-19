use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::{lnk, shellpaths};

#[derive(Serialize)]
pub struct RecentItem {
    name: String,
    path: String,
    modified: f64,
}

#[derive(Serialize)]
pub struct DownloadFile {
    name: String,
    path: String,
    size: u64,
    modified: f64,
}

#[derive(Serialize)]
pub struct ScreenshotFile {
    name: String,
    path: String,
    modified: f64,
}

#[derive(Serialize)]
pub struct FileSearchResult {
    name: String,
    path: String,
    #[serde(rename = "type")]
    kind: String,
}

fn modified_ms(meta: &fs::Metadata) -> f64 {
    meta.modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as f64)
        .unwrap_or(0.0)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn newest_files(dir: &Path, limit: usize) -> Vec<(String, PathBuf, u64, f64)> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut files: Vec<(String, PathBuf, u64, f64)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let meta = entry.metadata().ok()?;
            if !meta.is_file() {
                return None;
            }
            Some((
                entry.file_name().to_string_lossy().to_string(),
                entry.path(),
                meta.len(),
                modified_ms(&meta),
            ))
        })
        .collect();
    files.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    files.truncate(limit);
    files
}

#[tauri::command]
pub async fn get_recent_items() -> Vec<RecentItem> {
    tauri::async_runtime::spawn_blocking(|| {
        let dir = shellpaths::recent_dir();
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };
        let mut items: Vec<RecentItem> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.to_lowercase().ends_with(".lnk") {
                    return None;
                }
                let meta = entry.metadata().ok()?;
                let lnk_path = path_string(&entry.path());
                let target = lnk::target_of(&lnk_path).unwrap_or_else(|| lnk_path.clone());
                Some(RecentItem {
                    name: name[..name.len() - 4].to_string(),
                    path: target,
                    modified: modified_ms(&meta),
                })
            })
            .collect();
        items.sort_by(|a, b| b.modified.partial_cmp(&a.modified).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(20);
        items
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn get_downloads() -> Vec<DownloadFile> {
    tauri::async_runtime::spawn_blocking(|| {
        let dir = match shellpaths::downloads() {
            Some(dir) => dir,
            None => return Vec::new(),
        };
        newest_files(&dir, 20)
            .into_iter()
            .map(|(name, path, size, modified)| DownloadFile {
                name,
                path: path_string(&path),
                size,
                modified,
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

fn screenshots_dir() -> Option<PathBuf> {
    shellpaths::pictures().map(|dir| dir.join("Screenshots"))
}

#[tauri::command]
pub async fn get_recent_screenshots() -> Vec<ScreenshotFile> {
    tauri::async_runtime::spawn_blocking(|| {
        let dir = match screenshots_dir() {
            Some(dir) => dir,
            None => return Vec::new(),
        };
        newest_files(&dir, 10)
            .into_iter()
            .map(|(name, path, _, modified)| ScreenshotFile {
                name,
                path: path_string(&path),
                modified,
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub fn get_screenshots_dir() -> String {
    screenshots_dir().map(|dir| path_string(&dir)).unwrap_or_default()
}

fn search_dir(dir: &Path, query: &str, depth: u32, results: &mut Vec<FileSearchResult>) {
    if depth > 2 || results.len() >= 20 {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if results.len() >= 20 {
            return;
        }
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        if meta.is_dir() {
            search_dir(&path, query, depth + 1, results);
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_lowercase().contains(query) {
            let is_shortcut = path
                .extension()
                .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("lnk"))
                .unwrap_or(false);
            results.push(FileSearchResult {
                name,
                path: path_string(&path),
                kind: if is_shortcut { "app" } else { "file" }.to_string(),
            });
        }
    }
}

#[tauri::command]
pub async fn search_files(query: String) -> Vec<FileSearchResult> {
    if query.is_empty() {
        return Vec::new();
    }
    tauri::async_runtime::spawn_blocking(move || {
        let q = query.to_lowercase();
        let mut dirs: Vec<PathBuf> = Vec::new();
        dirs.extend(shellpaths::downloads());
        dirs.extend(shellpaths::desktop());
        dirs.extend(shellpaths::documents());
        dirs.push(shellpaths::start_menu_common());
        let mut results = Vec::new();
        for dir in dirs {
            search_dir(&dir, &q, 0, &mut results);
            if results.len() >= 20 {
                break;
            }
        }
        results
    })
    .await
    .unwrap_or_default()
}
