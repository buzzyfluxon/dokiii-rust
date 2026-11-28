use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

static ON: AtomicBool = AtomicBool::new(false);
static CHECKED: AtomicU64 = AtomicU64::new(0);

fn flag_path() -> PathBuf {
    config::config_path().with_file_name("debug-hit.flag")
}

fn log_path() -> PathBuf {
    config::config_path().with_file_name("debug-hit.log")
}

pub fn enabled() -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if CHECKED.swap(now, Ordering::Relaxed) != now {
        ON.store(flag_path().exists(), Ordering::Relaxed);
    }
    ON.load(Ordering::Relaxed)
}

pub fn log(line: &str) {
    if !enabled() {
        return;
    }
    let path = log_path();
    if fs::metadata(&path).map(|m| m.len() > 524_288).unwrap_or(false) {
        let _ = fs::remove_file(&path);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(file, "{} {}", ms, line);
    }
}

#[tauri::command]
pub fn get_debug_hit() -> bool {
    enabled()
}
