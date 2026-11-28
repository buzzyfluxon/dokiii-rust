#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub fn shell_open(target: &str) -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let verb: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
    let file: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            HWND::default(),
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if (result.0 as isize) > 32 {
        Ok(())
    } else {
        Err(format!("could not open {}", target))
    }
}

#[cfg(not(windows))]
pub fn shell_open(_target: &str) -> Result<(), String> {
    Err("only implemented on Windows".to_string())
}

fn resolve_alias(target: &str) -> String {
    match target.to_lowercase().as_str() {
        "spotify" => "spotify:".to_string(),
        "photos" => "ms-photos:".to_string(),
        _ => target.to_string(),
    }
}

#[tauri::command]
pub fn open_target(target: String) -> Result<(), String> {
    shell_open(&resolve_alias(&target))
}

#[tauri::command]
pub fn run_command(command: String) -> String {
    let raw = command.trim().to_string();
    if raw.is_empty() {
        return "empty command".to_string();
    }
    if raw.starts_with("start ") || raw.starts_with("ms-") || raw.starts_with("shell:") {
        let target = match raw.strip_prefix("start ") {
            Some(rest) => rest.trim().to_string(),
            None => raw.clone(),
        };
        return if shell_open(&target).is_ok() { "success" } else { "error" }.to_string();
    }

    let cmd = raw.to_lowercase();
    let has_args = cmd.split_whitespace().next().map(|first| first != cmd).unwrap_or(false);

    if !has_args {
        if cmd == "lock" {
            #[cfg(windows)]
            {
                let _ = Command::new("rundll32.exe")
                    .arg("user32.dll,LockWorkStation")
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
            }
            return "success".to_string();
        }
        let target = match cmd.as_str() {
            "photos" => "ms-photos:",
            "chrome" => "chrome",
            "edge" => "msedge",
            "downloads" => "shell:downloads",
            "document" | "documents" => "shell:Personal",
            "picture" | "pictures" => "shell:My Pictures",
            "recycle" | "recycle bin" => "shell:RecycleBinFolder",
            "settings" => "ms-settings:",
            "storage" => "ms-settings:storagesense",
            "calculator" | "calc" => "calc",
            "notepad" => "notepad",
            "task manager" | "taskmgr" => "taskmgr",
            "terminal" | "powershell" => "wt",
            "cmd" => "cmd",
            "control panel" => "control",
            "explorer" => "explorer",
            "store" => "ms-windows-store:",
            _ => "",
        };
        if !target.is_empty() {
            return if shell_open(target).is_ok() { "success" } else { "error" }.to_string();
        }
    }

    #[cfg(windows)]
    {
        return match Command::new("cmd")
            .args(["/C", &raw])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(_) => "success".to_string(),
            Err(_) => "error".to_string(),
        };
    }
    #[cfg(not(windows))]
    {
        "error".to_string()
    }
}

#[tauri::command]
pub fn show_uninstall(app: tauri::AppHandle) {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["uninstall.exe", "Uninstall DOKIII.exe"] {
                let candidate = dir.join(name);
                if candidate.exists() && std::process::Command::new(&candidate).spawn().is_ok() {
                    crate::taskbar::restore_on_exit();
                    app.exit(0);
                    return;
                }
            }
        }
    }
    let _ = shell_open("ms-settings:appsfeatures");
}
