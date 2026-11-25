#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

static PAYLOAD: &[u8] = include_bytes!("../payload/DOKIII-Setup.exe");

const APP_NAME: &str = "DOKIII";
const UNINSTALL_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\DOKIII";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Defaults {
    path: String,
    existing: bool,
    version: String,
}

#[cfg(windows)]
fn quiet(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000)
}

#[cfg(not(windows))]
fn quiet(command: &mut Command) -> &mut Command {
    command
}

fn local_app_data() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public\AppData\Local"))
}

fn default_install_dir() -> PathBuf {
    local_app_data().join("Programs").join(APP_NAME)
}

fn registry_value(name: &str) -> Option<String> {
    let mut command = Command::new("reg");
    command.args(["query", UNINSTALL_KEY, "/v", name]);
    let output = quiet(&mut command).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    for line in text.lines() {
        if !line.contains(name) {
            continue;
        }
        if let Some(index) = line.find("REG_SZ") {
            let value = line[index + "REG_SZ".len()..].trim().trim_matches('"').to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn existing_install() -> Option<PathBuf> {
    let location = registry_value("InstallLocation")?;
    let path = PathBuf::from(location);
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

fn find_main_exe(dir: &Path) -> Option<PathBuf> {
    let mut fallback: Option<PathBuf> = None;
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let is_exe = path
            .extension()
            .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("exe"))
            .unwrap_or(false);
        if !is_exe {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if stem.starts_with("uninstall") {
            continue;
        }
        if stem == "dokiii" {
            return Some(path);
        }
        if fallback.is_none() {
            fallback = Some(path);
        }
    }
    fallback
}

fn desktop_dir() -> Option<PathBuf> {
    let mut command = Command::new("powershell");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "[Environment]::GetFolderPath('Desktop')",
    ]);
    let output = quiet(&mut command).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(PathBuf::from(text))
    }
}

fn create_shortcut(lnk: &Path, exe: &Path, dir: &Path) -> bool {
    let script = "$s=(New-Object -ComObject WScript.Shell).CreateShortcut($env:DK_LNK);\
$s.TargetPath=$env:DK_EXE;$s.WorkingDirectory=$env:DK_DIR;$s.IconLocation=$env:DK_EXE;$s.Save()";
    let mut command = Command::new("powershell");
    command
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("DK_LNK", lnk)
        .env("DK_EXE", exe)
        .env("DK_DIR", dir);
    quiet(&mut command)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn phase(app: &AppHandle, text: &str) {
    let _ = app.emit("install:phase", text);
}

#[tauri::command]
fn get_defaults() -> Defaults {
    let existing = existing_install();
    Defaults {
        existing: existing.is_some(),
        path: existing
            .unwrap_or_else(default_install_dir)
            .to_string_lossy()
            .to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
async fn pick_folder(current: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = rfd::FileDialog::new().set_title("Choose where to install DOKIII");
        let start = PathBuf::from(&current);
        if let Some(parent) = start.parent() {
            if parent.is_dir() {
                dialog = dialog.set_directory(parent);
            }
        }
        let picked = dialog.pick_folder()?;
        let named_app = picked
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case(APP_NAME))
            .unwrap_or(false);
        let target = if named_app { picked } else { picked.join(APP_NAME) };
        Some(target.to_string_lossy().to_string())
    })
    .await
    .unwrap_or(None)
}

#[tauri::command]
async fn install(app: AppHandle, path: String, desktop_shortcut: bool) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || run_install(&app, &path, desktop_shortcut))
        .await
        .map_err(|error| error.to_string())?
}

fn run_install(app: &AppHandle, path: &str, desktop_shortcut: bool) -> Result<String, String> {
    if PAYLOAD.is_empty() {
        return Err("This installer was built without the DOKIII payload.".to_string());
    }
    let target = PathBuf::from(path.trim());
    if target.as_os_str().is_empty() {
        return Err("Choose an install folder first.".to_string());
    }

    let updating = existing_install().is_some();

    phase(app, "Preparing");
    let staging = std::env::temp_dir().join("dokiii-install");
    fs::create_dir_all(&staging).map_err(|error| format!("Could not create a temporary folder: {error}"))?;
    let setup = staging.join("DOKIII-Setup.exe");
    fs::write(&setup, PAYLOAD).map_err(|error| format!("Could not unpack the installer: {error}"))?;

    let mut kill = Command::new("taskkill");
    kill.args(["/IM", "dokiii.exe", "/F"]);
    let _ = quiet(&mut kill).output();
    std::thread::sleep(Duration::from_millis(600));

    phase(app, "Installing");
    let mut command = Command::new(&setup);
    command.arg("/S");
    if updating {
        command.arg("/UPDATE");
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.raw_arg(format!("/D={}", target.display()));
    }
    #[cfg(not(windows))]
    {
        command.arg(format!("/D={}", target.display()));
    }
    let status = quiet(&mut command)
        .status()
        .map_err(|error| format!("Could not start the installer: {error}"))?;
    if !status.success() {
        let _ = fs::remove_file(&setup);
        return Err(format!(
            "The installer stopped with code {}.",
            status.code().unwrap_or(-1)
        ));
    }

    phase(app, "Finishing up");
    let exe = find_main_exe(&target)
        .ok_or_else(|| "DOKIII was installed, but its program file could not be found.".to_string())?;

    if let Some(desktop) = desktop_dir() {
        let lnk = desktop.join(format!("{APP_NAME}.lnk"));
        if desktop_shortcut {
            if !lnk.exists() {
                create_shortcut(&lnk, &exe, &target);
            }
        } else if lnk.exists() {
            let _ = fs::remove_file(&lnk);
        }
    }

    let _ = fs::remove_file(&setup);
    Ok(exe.to_string_lossy().to_string())
}

#[tauri::command]
fn launch_app(exe: String) -> Result<(), String> {
    let path = PathBuf::from(&exe);
    let mut command = Command::new(&path);
    if let Some(dir) = path.parent() {
        command.current_dir(dir);
    }
    command.spawn().map(|_| ()).map_err(|error| error.to_string())
}

#[tauri::command]
fn close_installer(app: AppHandle) {
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_defaults,
            pick_folder,
            install,
            launch_app,
            close_installer
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the DOKIII installer");
}
