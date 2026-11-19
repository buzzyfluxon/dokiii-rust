#[cfg(windows)]
#[tauri::command]
pub async fn recycle_bin_count() -> u64 {
    tauri::async_runtime::spawn_blocking(|| {
        use windows::Win32::UI::Shell::{SHQueryRecycleBinW, SHQUERYRBINFO};

        let mut info = SHQUERYRBINFO {
            cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
            ..Default::default()
        };
        match unsafe { SHQueryRecycleBinW(windows::core::PCWSTR::null(), &mut info) } {
            Ok(()) => info.i64NumItems.max(0) as u64,
            Err(_) => 0,
        }
    })
    .await
    .unwrap_or(0)
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn recycle_bin_count() -> u64 {
    0
}

#[cfg(windows)]
#[tauri::command]
pub async fn recycle_bin_empty() {
    let _ = tauri::async_runtime::spawn_blocking(|| {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Shell::{
            SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
        };

        let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
        unsafe {
            let _ = SHEmptyRecycleBinW(HWND::default(), windows::core::PCWSTR::null(), flags);
        }
    })
    .await;
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn recycle_bin_empty() {}
