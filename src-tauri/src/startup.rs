#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "DOKIII";

#[cfg(windows)]
fn reg(args: &[&str]) -> bool {
    Command::new("reg")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn reg(_args: &[&str]) -> bool {
    false
}

fn remove_legacy_shortcut() {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let lnk = std::path::PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join("DOKIII.lnk");
        let _ = std::fs::remove_file(lnk);
    }
}

#[tauri::command]
pub fn get_launch_on_startup() -> bool {
    reg(&["query", RUN_KEY, "/v", VALUE_NAME])
}

#[tauri::command]
pub fn set_launch_on_startup(enabled: bool) -> bool {
    remove_legacy_shortcut();
    if enabled {
        let exe = match std::env::current_exe() {
            Ok(path) => path,
            Err(_) => return false,
        };
        let value = format!("\"{}\" --startup", exe.display());
        reg(&["add", RUN_KEY, "/v", VALUE_NAME, "/t", "REG_SZ", "/d", &value, "/f"]);
    } else {
        reg(&["delete", RUN_KEY, "/v", VALUE_NAME, "/f"]);
    }
    get_launch_on_startup()
}
