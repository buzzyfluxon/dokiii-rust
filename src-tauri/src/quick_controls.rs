use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::process::Command;

/// When a brightness read failed, the display has no software brightness control (most desktop
/// monitors). Spawning PowerShell again every few seconds just to fail again burns CPU and RAM, so
/// the failure is remembered and the next try waits.
static BRIGHTNESS_FAILED_AT: Mutex<Option<Instant>> = Mutex::new(None);
const BRIGHTNESS_RETRY: Duration = Duration::from_secs(600);

/// Windows only needs to be asked for radio access once per run.
static RADIO_ACCESS_ASKED: AtomicBool = AtomicBool::new(false);

fn brightness_known_unsupported() -> bool {
    let guard = match BRIGHTNESS_FAILED_AT.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match *guard {
        Some(at) => at.elapsed() < BRIGHTNESS_RETRY,
        None => false,
    }
}

fn remember_brightness(ok: bool) {
    let mut guard = match BRIGHTNESS_FAILED_AT.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = if ok { None } else { Some(Instant::now()) };
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Snapshot of the quick toggles.
/// `brightness` is -1 when the display has no software brightness control (most desktop monitors),
/// and -2 when it was not read this time.
/// `wifi` / `bluetooth` are `None` when the PC has no such radio.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct QuickState {
    brightness: i32,
    wifi: Option<bool>,
    bluetooth: Option<bool>,
}

impl Default for QuickState {
    fn default() -> Self {
        Self {
            brightness: -1,
            wifi: None,
            bluetooth: None,
        }
    }
}

#[cfg(windows)]
fn powershell(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(windows)]
fn read_brightness() -> i32 {
    if brightness_known_unsupported() {
        return -1;
    }
    let value = powershell(
        "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1).CurrentBrightness",
    )
    .and_then(|text| text.lines().next().and_then(|line| line.trim().parse::<i32>().ok()))
    .map(|value| value.clamp(0, 100));
    remember_brightness(value.is_some());
    value.unwrap_or(-1)
}

#[cfg(not(windows))]
fn read_brightness() -> i32 {
    -1
}

#[cfg(windows)]
fn write_brightness(value: i32) {
    let value = value.clamp(0, 100);
    let script = format!(
        "Invoke-CimMethod -InputObject (Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Select-Object -First 1) -MethodName WmiSetBrightness -Arguments @{{Timeout=1;Brightness={}}} | Out-Null",
        value
    );
    let _ = powershell(&script);
}

#[cfg(not(windows))]
fn write_brightness(_value: i32) {}

#[cfg(windows)]
fn find_radio(kind: windows::Devices::Radios::RadioKind) -> Option<windows::Devices::Radios::Radio> {
    use windows::Devices::Radios::Radio;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    if !RADIO_ACCESS_ASKED.swap(true, Ordering::Relaxed) {
        if let Ok(request) = Radio::RequestAccessAsync() {
            let _ = request.get();
        }
    }
    let radios = Radio::GetRadiosAsync().ok()?.get().ok()?;
    for radio in radios {
        if radio.Kind().ok() == Some(kind) {
            return Some(radio);
        }
    }
    None
}

#[cfg(windows)]
fn read_radio(kind: windows::Devices::Radios::RadioKind) -> Option<bool> {
    use windows::Devices::Radios::RadioState;
    let radio = find_radio(kind)?;
    Some(radio.State().ok()? == RadioState::On)
}

#[cfg(windows)]
fn write_radio(kind: windows::Devices::Radios::RadioKind, on: bool) {
    use windows::Devices::Radios::RadioState;
    if let Some(radio) = find_radio(kind) {
        let state = if on { RadioState::On } else { RadioState::Off };
        if let Ok(op) = radio.SetStateAsync(state) {
            let _ = op.get();
        }
    }
}

/// Brightness needs a PowerShell process, so it is only read when `with_brightness` is set.
/// The radios are read in-process through the Windows API, which is cheap.
#[cfg(windows)]
fn read_state(with_brightness: bool) -> QuickState {
    use windows::Devices::Radios::RadioKind;
    QuickState {
        brightness: if with_brightness { read_brightness() } else { -2 },
        wifi: read_radio(RadioKind::WiFi),
        bluetooth: read_radio(RadioKind::Bluetooth),
    }
}

#[cfg(not(windows))]
fn read_state(_with_brightness: bool) -> QuickState {
    QuickState::default()
}

/// `brightness` is -2 when the caller did not ask for it, so the page keeps the value it has.
#[tauri::command]
pub async fn get_quick_controls(with_brightness: Option<bool>) -> QuickState {
    let with_brightness = with_brightness.unwrap_or(true);
    tauri::async_runtime::spawn_blocking(move || read_state(with_brightness))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn set_brightness(value: f64) -> i32 {
    let clamped = value.round().clamp(0.0, 100.0) as i32;
    tauri::async_runtime::spawn_blocking(move || {
        write_brightness(clamped);
        clamped
    })
    .await
    .unwrap_or(clamped)
}

/// `kind` is "wifi" or "bluetooth". Returns the radio state after the change (`None` if there is no such radio).
#[tauri::command]
pub async fn set_radio(kind: String, on: bool) -> Option<bool> {
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        {
            use windows::Devices::Radios::RadioKind;
            let radio_kind = match kind.as_str() {
                "wifi" => RadioKind::WiFi,
                "bluetooth" => RadioKind::Bluetooth,
                _ => return None,
            };
            write_radio(radio_kind, on);
            read_radio(radio_kind)
        }
        #[cfg(not(windows))]
        {
            let _ = (kind, on);
            None
        }
    })
    .await
    .unwrap_or(None)
}
