use std::sync::Mutex;

use serde::Serialize;

#[derive(Serialize, Default)]
pub struct SystemMetrics {
    cpu: u32,
    ram: u32,
    ssd: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryStatus {
    percent: u32,
    is_charging: bool,
    has_battery: bool,
}

impl Default for BatteryStatus {
    fn default() -> Self {
        Self {
            percent: 100,
            is_charging: false,
            has_battery: false,
        }
    }
}

#[derive(Serialize)]
pub struct StorageInfo {
    drive: String,
    total: u64,
    used: u64,
    free: u64,
}

static PREV_CPU: Mutex<Option<(u64, u64, u64)>> = Mutex::new(None);

#[cfg(windows)]
fn filetime_u64(ft: windows::Win32::Foundation::FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

#[cfg(windows)]
fn cpu_percent() -> u32 {
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Threading::GetSystemTimes;

    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }.is_err() {
        return 0;
    }
    let now = (filetime_u64(idle), filetime_u64(kernel), filetime_u64(user));
    let mut guard = match PREV_CPU.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let previous = guard.replace(now);
    let (pi, pk, pu) = match previous {
        Some(value) => value,
        None => return 0,
    };
    let idle_diff = now.0.saturating_sub(pi);
    let total_diff = now.1.saturating_sub(pk) + now.2.saturating_sub(pu);
    if total_diff == 0 {
        return 0;
    }
    let busy = total_diff.saturating_sub(idle_diff);
    ((busy as f64 / total_diff as f64) * 100.0).round().clamp(0.0, 100.0) as u32
}

#[cfg(not(windows))]
fn cpu_percent() -> u32 {
    0
}

#[cfg(windows)]
fn ram_percent() -> u32 {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) }.is_err() {
        return 0;
    }
    status.dwMemoryLoad.min(100)
}

#[cfg(not(windows))]
fn ram_percent() -> u32 {
    0
}

#[cfg(windows)]
fn disk_space(root: &str) -> Option<(u64, u64)> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
    let mut free_to_caller = 0u64;
    let mut total = 0u64;
    let mut total_free = 0u64;
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide.as_ptr()),
            Some(&mut free_to_caller),
            Some(&mut total),
            Some(&mut total_free),
        )
    }
    .ok()?;
    if total == 0 {
        None
    } else {
        Some((total, free_to_caller))
    }
}

#[cfg(not(windows))]
fn disk_space(_root: &str) -> Option<(u64, u64)> {
    None
}

#[tauri::command]
pub fn get_system_metrics() -> SystemMetrics {
    let ssd = disk_space("C:\\")
        .map(|(total, free)| (((total - free) as f64 / total as f64) * 100.0).round() as u32)
        .unwrap_or(50);
    SystemMetrics {
        cpu: cpu_percent(),
        ram: ram_percent(),
        ssd: ssd.min(100),
    }
}

#[tauri::command]
pub fn get_storage_info() -> Vec<StorageInfo> {
    ["C", "D", "E", "F", "G", "H", "Z"]
        .iter()
        .filter_map(|letter| {
            let (total, free) = disk_space(&format!("{}:\\", letter))?;
            Some(StorageInfo {
                drive: letter.to_string(),
                total,
                used: total - free,
                free,
            })
        })
        .collect()
}

#[cfg(windows)]
#[tauri::command]
pub fn get_battery_status() -> BatteryStatus {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

    let mut status = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() {
        return BatteryStatus::default();
    }
    let no_battery = status.BatteryFlag == 128 || status.BatteryFlag == 255;
    if no_battery {
        return BatteryStatus::default();
    }
    let percent = if status.BatteryLifePercent == 255 {
        100
    } else {
        status.BatteryLifePercent.min(100) as u32
    };
    BatteryStatus {
        percent,
        is_charging: status.ACLineStatus == 1 || (status.BatteryFlag & 8) != 0,
        has_battery: true,
    }
}

#[cfg(not(windows))]
#[tauri::command]
pub fn get_battery_status() -> BatteryStatus {
    BatteryStatus::default()
}
