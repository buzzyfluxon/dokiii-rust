use std::fs;
use std::io;
use std::path::PathBuf;

use serde_json::{Map, Value};

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("dokiii").join("config.json")
}

pub fn load() -> Map<String, Value> {
    let raw = match fs::read_to_string(config_path()) {
        Ok(raw) => raw,
        Err(_) => return Map::new(),
    };
    match serde_json::from_str::<Value>(&raw) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

pub fn write(map: &Map<String, Value>) -> io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(map)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, &path)
}
