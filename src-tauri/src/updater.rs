use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

const OWNER: &str = "buzzyfluxon";
const REPO: &str = "dokiii-rust";
const CHECK_DELAY: Duration = Duration::from_secs(10);
const CHECK_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
const MAX_DOWNLOAD_RETRIES: u32 = 2;
const DOWNLOAD_RETRY_DELAY: Duration = Duration::from_secs(4);
const OFFLINE_RETRY_DELAY: Duration = Duration::from_secs(60);
const MAX_OFFLINE_RETRIES: u32 = 8;
const OFFLINE_MESSAGE: &str = "GitHub can't be reached right now. Check your internet connection and try again.";
const INSTALLER_ASSET: &str = "DOKIII-Setup.exe";

struct State {
    status: Value,
    busy: bool,
    downloading: bool,
    version: String,
    download_url: String,
    asset_name: String,
    installer: Option<PathBuf>,
}

static STATE: Mutex<State> = Mutex::new(State {
    status: Value::Null,
    busy: false,
    downloading: false,
    version: String::new(),
    download_url: String::new(),
    asset_name: String::new(),
    installer: None,
});

fn lock() -> std::sync::MutexGuard<'static, State> {
    match STATE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn broadcast(app: &AppHandle, status: Value) {
    lock().status = status.clone();
    let _ = app.emit("updater:status", status);
}

fn status_name(value: &Value) -> String {
    value
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("idle")
        .to_string()
}

fn store_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("dokiii").join("dokiii-updater.json")
}

fn dismissed_version() -> String {
    fs::read_to_string(store_path())
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|value| value.get("dismissedVersion").and_then(|v| v.as_str().map(String::from)))
        .unwrap_or_default()
}

fn parse_version(raw: &str) -> Vec<u64> {
    raw.trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

fn is_newer(remote: &str, local: &str) -> bool {
    let (mut a, mut b) = (parse_version(remote), parse_version(local));
    let len = a.len().max(b.len());
    a.resize(len, 0);
    b.resize(len, 0);
    a > b
}

fn agent(timeout_secs: u64) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("DOKIII-updater")
        .build()
}

fn is_network_failure(message: &str) -> bool {
    let lower = message.to_lowercase();
    [
        "dns failed",
        "no such host",
        "connection failed",
        "timed out",
        "network is unreachable",
        "failed to lookup",
        "os error",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn friendly(message: String) -> String {
    if is_network_failure(&message) {
        OFFLINE_MESSAGE.to_string()
    } else {
        message
    }
}

fn fetch_latest_via_web() -> Result<Option<(String, String, String)>, String> {
    let url = format!("https://github.com/{}/{}/releases/latest", OWNER, REPO);
    let client = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout(Duration::from_secs(15))
        .user_agent("DOKIII-updater")
        .build();
    let response = client.get(&url).call().map_err(|e| e.to_string())?;
    let location = response
        .header("location")
        .ok_or("GitHub did not point to a release")?
        .to_string();
    let tag = location
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    if tag.is_empty() {
        return Err("GitHub did not point to a release".to_string());
    }
    let version = tag.trim_start_matches('v').to_string();
    if !is_newer(&version, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }
    let download = format!(
        "https://github.com/{}/{}/releases/download/{}/{}",
        OWNER, REPO, tag, INSTALLER_ASSET
    );
    Ok(Some((version, download, INSTALLER_ASSET.to_string())))
}

fn fetch_latest() -> Result<Option<(String, String, String)>, String> {
    match fetch_latest_from_api() {
        Ok(found) => Ok(found),
        Err(api_error) => match fetch_latest_via_web() {
            Ok(found) => Ok(found),
            Err(_) => Err(friendly(api_error)),
        },
    }
}

fn fetch_latest_from_api() -> Result<Option<(String, String, String)>, String> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/latest", OWNER, REPO);
    let response = agent(15)
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?;
    let mut raw = String::new();
    response
        .into_reader()
        .take(4_000_000)
        .read_to_string(&mut raw)
        .map_err(|e| e.to_string())?;
    let release: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or("release has no tag")?
        .to_string();
    let version = tag.trim_start_matches('v').to_string();
    if !is_newer(&version, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }

    let assets = release.get("assets").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    for asset in assets {
        let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let lower = name.to_lowercase();
        if lower.ends_with("-setup.exe") || (lower.ends_with(".exe") && lower.contains("setup")) {
            let download = asset
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .ok_or("asset has no download url")?;
            return Ok(Some((version, download.to_string(), name.to_string())));
        }
    }
    Err("the release has no installer".to_string())
}

fn run_check(app: AppHandle, manual: bool) -> bool {
    if cfg!(debug_assertions) {
        broadcast(&app, json!({ "status": "not-available", "isManual": manual }));
        return false;
    }
    {
        let mut state = lock();
        if state.busy || state.downloading {
            return false;
        }
        state.busy = true;
    }
    broadcast(&app, json!({ "status": "checking", "isManual": manual }));

    match fetch_latest() {
        Ok(Some((version, url, asset))) => {
            {
                let mut state = lock();
                state.version = version.clone();
                state.download_url = url;
                state.asset_name = asset;
                state.busy = false;
            }
            if !manual && dismissed_version() == version {
                lock().status = Value::Null;
                return false;
            }
            broadcast(&app, json!({ "status": "available", "version": version, "isManual": manual }));
            false
        }
        Ok(None) => {
            lock().busy = false;
            broadcast(&app, json!({ "status": "not-available", "isManual": manual }));
            false
        }
        Err(message) => {
            lock().busy = false;
            let offline = message == OFFLINE_MESSAGE;
            broadcast(
                &app,
                json!({ "status": "error", "message": message, "phase": "check", "isManual": manual }),
            );
            offline
        }
    }
}

fn download_once(app: &AppHandle, url: &str, target: &PathBuf, version: &str) -> Result<(), String> {
    let response = agent(600).get(url).call().map_err(|e| e.to_string())?;
    let total: u64 = response
        .header("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut reader = response.into_reader();
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = fs::File::create(target).map_err(|e| e.to_string())?;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut received: u64 = 0;
    let mut last_percent: i64 = -1;
    loop {
        let read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
        received += read as u64;
        if total > 0 {
            let percent = ((received as f64 / total as f64) * 100.0).round().clamp(0.0, 100.0) as i64;
            if percent != last_percent {
                last_percent = percent;
                broadcast(app, json!({ "status": "downloading", "version": version, "percent": percent }));
            }
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    if received == 0 {
        return Err("the download was empty".to_string());
    }
    Ok(())
}

fn run_download(app: AppHandle) {
    let (version, url, asset) = {
        let state = lock();
        (state.version.clone(), state.download_url.clone(), state.asset_name.clone())
    };
    let target = std::env::temp_dir().join("dokiii-update").join(&asset);
    let mut attempt = 0;
    loop {
        match download_once(&app, &url, &target, &version) {
            Ok(()) => {
                {
                    let mut state = lock();
                    state.downloading = false;
                    state.installer = Some(target);
                }
                broadcast(&app, json!({ "status": "downloaded", "version": version }));
                return;
            }
            Err(message) => {
                if attempt < MAX_DOWNLOAD_RETRIES {
                    attempt += 1;
                    broadcast(&app, json!({ "status": "downloading", "version": version, "percent": 0 }));
                    std::thread::sleep(DOWNLOAD_RETRY_DELAY);
                    continue;
                }
                lock().downloading = false;
                broadcast(
                    &app,
                    json!({ "status": "error", "message": friendly(message), "phase": "download", "isManual": false }),
                );
                return;
            }
        }
    }
}

pub fn start(app: AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("dokiii-updater".into())
        .spawn(move || {
            std::thread::sleep(CHECK_DELAY);
            let mut offline_retries: u32 = 0;
            loop {
                let offline = run_check(app.clone(), false);
                if offline && offline_retries < MAX_OFFLINE_RETRIES {
                    offline_retries += 1;
                    std::thread::sleep(OFFLINE_RETRY_DELAY);
                } else {
                    offline_retries = 0;
                    std::thread::sleep(CHECK_INTERVAL);
                }
            }
        });
}

#[tauri::command]
pub fn check_for_updates(app: AppHandle, manual: Option<bool>) {
    let manual = manual.unwrap_or(false);
    let _ = std::thread::Builder::new()
        .name("dokiii-update-check".into())
        .spawn(move || {
            run_check(app, manual);
        });
}

#[tauri::command]
pub fn start_update_download(app: AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    {
        let mut state = lock();
        if status_name(&state.status) != "available" || state.downloading {
            return;
        }
        state.downloading = true;
    }
    let _ = std::thread::Builder::new()
        .name("dokiii-update-download".into())
        .spawn(move || run_download(app));
}

#[tauri::command]
pub fn install_update(app: AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    let installer = {
        let state = lock();
        if status_name(&state.status) != "downloaded" {
            return;
        }
        state.installer.clone()
    };
    if let Some(path) = installer {
        if path.exists() && std::process::Command::new(&path).args(["/P", "/R", "/UPDATE"]).spawn().is_ok() {
            crate::taskbar::restore_on_exit();
            app.exit(0);
        }
    }
}

#[tauri::command]
pub fn dismiss_update(version: String) {
    if !cfg!(debug_assertions) && !version.is_empty() {
        let path = store_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, json!({ "dismissedVersion": version }).to_string());
    }
    lock().status = Value::Null;
}

#[tauri::command]
pub fn get_updater_status() -> Value {
    let state = lock();
    if state.status.is_null() {
        json!({ "status": "idle" })
    } else {
        state.status.clone()
    }
}
