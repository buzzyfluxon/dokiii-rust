use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use serde_json::{Map, Value};

use crate::config;

pub struct AppState {
    pub js_hit: AtomicBool,
    pub modal: AtomicBool,
    pub popover: AtomicBool,
    pub config: Mutex<Map<String, Value>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            js_hit: AtomicBool::new(false),
            modal: AtomicBool::new(false),
            popover: AtomicBool::new(false),
            config: Mutex::new(config::load()),
        }
    }

    pub fn default_monitor(&self) -> usize {
        let cfg = match self.config.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cfg.get("dock")
            .and_then(|dock| dock.get("defaultMonitor"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as usize
    }
}
