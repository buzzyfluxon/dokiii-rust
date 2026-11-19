use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use base64::Engine;

use crate::icon;

fn fnv1a(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:016x}", hash)
}

fn cache_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("dokiii").join("image-cache")
}

fn download(url: &str, timeout_secs: u64, limit: u64) -> Option<(Vec<u8>, String)> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(timeout_secs))
        .build();
    let response = agent.get(url).call().ok()?;
    let mime = response
        .header("content-type")
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let mut bytes = Vec::new();
    response.into_reader().take(limit).read_to_end(&mut bytes).ok()?;
    Some((bytes, mime))
}

#[tauri::command]
pub async fn cache_image(url: String) -> Option<String> {
    let lower = url.to_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return None;
    }
    tauri::async_runtime::spawn_blocking(move || {
        let dir = cache_dir();
        let _ = fs::create_dir_all(&dir);
        let key = fnv1a(&url);
        let file = dir.join(&key);
        let mime_file = dir.join(format!("{}.mime", key));

        if let (Ok(bytes), Ok(mime)) = (fs::read(&file), fs::read_to_string(&mime_file)) {
            return Some(format!(
                "data:{};base64,{}",
                mime.trim(),
                base64::engine::general_purpose::STANDARD.encode(bytes)
            ));
        }

        let (bytes, mime) = download(&url, 12, 25_000_000)?;
        if !mime.starts_with("image/") || bytes.is_empty() {
            return None;
        }
        let _ = fs::write(&file, &bytes);
        let _ = fs::write(&mime_file, &mime);
        Some(format!(
            "data:{};base64,{}",
            mime,
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    })
    .await
    .unwrap_or(None)
}

#[tauri::command]
pub async fn fetch_favicon(url: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let parsed = url::Url::parse(&url).ok()?;
        let host = parsed.host_str()?.to_string();
        let origin = parsed.origin().ascii_serialization();
        let candidates = [
            format!("https://www.google.com/s2/favicons?domain={}&sz=128", host),
            format!("https://icons.duckduckgo.com/ip3/{}.ico", host),
            format!("{}/apple-touch-icon.png", origin),
            format!("{}/favicon.ico", origin),
        ];
        for candidate in candidates.iter() {
            if let Some((bytes, _)) = download(candidate, 4, 2_000_000) {
                if let Some(data_url) = icon::bytes_to_icon_data_url(&bytes) {
                    return Some(data_url);
                }
            }
        }
        None
    })
    .await
    .unwrap_or(None)
}
