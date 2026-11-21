use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;

fn photos_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("dokiii").join("photos")
}

#[tauri::command]
pub async fn save_photo(data: String, ext: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ext: &str = match ext.to_lowercase().as_str() {
            "png" => "png",
            "jpg" | "jpeg" => "jpg",
            "webp" => "webp",
            "gif" => "gif",
            "bmp" => "bmp",
            _ => return None,
        };
        let payload = data.rsplit(',').next().unwrap_or("");
        let bytes = base64::engine::general_purpose::STANDARD.decode(payload).ok()?;
        if bytes.is_empty() || bytes.len() > 25_000_000 {
            return None;
        }
        let dir = photos_dir();
        fs::create_dir_all(&dir).ok()?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_nanos();
        let path = dir.join(format!("photo-{:x}.{}", stamp, ext));
        fs::write(&path, bytes).ok()?;
        Some(path.to_string_lossy().to_string())
    })
    .await
    .unwrap_or(None)
}
