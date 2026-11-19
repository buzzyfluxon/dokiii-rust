use std::path::PathBuf;

#[cfg(windows)]
pub fn known_folder(id: &windows::core::GUID) -> Option<PathBuf> {
    use std::ffi::c_void;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT};

    unsafe {
        let raw = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, HANDLE::default()).ok()?;
        let text = raw.to_string().ok();
        CoTaskMemFree(Some(raw.0 as *const c_void));
        text.map(PathBuf::from)
    }
}

#[cfg(windows)]
pub fn downloads() -> Option<PathBuf> {
    known_folder(&windows::Win32::UI::Shell::FOLDERID_Downloads)
}

#[cfg(windows)]
pub fn pictures() -> Option<PathBuf> {
    known_folder(&windows::Win32::UI::Shell::FOLDERID_Pictures)
}

#[cfg(windows)]
pub fn desktop() -> Option<PathBuf> {
    known_folder(&windows::Win32::UI::Shell::FOLDERID_Desktop)
}

#[cfg(windows)]
pub fn documents() -> Option<PathBuf> {
    known_folder(&windows::Win32::UI::Shell::FOLDERID_Documents)
}

#[cfg(not(windows))]
pub fn downloads() -> Option<PathBuf> {
    None
}

#[cfg(not(windows))]
pub fn pictures() -> Option<PathBuf> {
    None
}

#[cfg(not(windows))]
pub fn desktop() -> Option<PathBuf> {
    None
}

#[cfg(not(windows))]
pub fn documents() -> Option<PathBuf> {
    None
}

pub fn start_menu_common() -> PathBuf {
    let base = std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    base.join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
}

pub fn recent_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("Microsoft").join("Windows").join("Recent")
}
