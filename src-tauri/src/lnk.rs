#[cfg(windows)]
pub fn target_of(path: &str) -> Option<String> {
    use std::iter::once;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_RAWPATH};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        let wide: Vec<u16> = path.encode_utf16().chain(once(0)).collect();
        persist.Load(PCWSTR(wide.as_ptr()), STGM_READ).ok()?;
        let mut buffer = [0u16; 520];
        let mut data = WIN32_FIND_DATAW::default();
        link.GetPath(&mut buffer, &mut data, SLGP_RAWPATH.0 as u32).ok()?;
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        let text = String::from_utf16_lossy(&buffer[..len]);
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }
}

#[cfg(not(windows))]
pub fn target_of(_path: &str) -> Option<String> {
    None
}
