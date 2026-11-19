use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct AudioState {
    volume: u32,
    is_muted: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            volume: 50,
            is_muted: false,
        }
    }
}

#[cfg(windows)]
fn endpoint() -> windows::core::Result<windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume> {
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
    }
}

#[cfg(windows)]
fn read_state() -> AudioState {
    let endpoint = match endpoint() {
        Ok(endpoint) => endpoint,
        Err(_) => return AudioState::default(),
    };
    unsafe {
        let volume = endpoint.GetMasterVolumeLevelScalar().unwrap_or(0.5);
        let muted = endpoint.GetMute().map(|m| m.as_bool()).unwrap_or(false);
        AudioState {
            volume: (volume * 100.0).round().clamp(0.0, 100.0) as u32,
            is_muted: muted,
        }
    }
}

#[cfg(not(windows))]
fn read_state() -> AudioState {
    AudioState::default()
}

#[cfg(windows)]
fn write_volume(volume: u32) {
    if let Ok(endpoint) = endpoint() {
        let scalar = (volume.min(100) as f32) / 100.0;
        unsafe {
            let _ = endpoint.SetMasterVolumeLevelScalar(scalar, std::ptr::null());
        }
    }
}

#[cfg(not(windows))]
fn write_volume(_volume: u32) {}

#[cfg(windows)]
fn write_mute(muted: bool) {
    if let Ok(endpoint) = endpoint() {
        unsafe {
            let _ = endpoint.SetMute(muted, std::ptr::null());
        }
    }
}

#[cfg(not(windows))]
fn write_mute(_muted: bool) {}

#[tauri::command]
pub async fn get_audio_volume() -> AudioState {
    tauri::async_runtime::spawn_blocking(read_state)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn set_audio_volume(volume: f64) -> AudioState {
    let clamped = volume.round().clamp(0.0, 100.0) as u32;
    tauri::async_runtime::spawn_blocking(move || {
        write_volume(clamped);
        let mut state = read_state();
        state.volume = clamped;
        state
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn toggle_audio_mute() -> bool {
    tauri::async_runtime::spawn_blocking(|| {
        let next = !read_state().is_muted;
        write_mute(next);
        next
    })
    .await
    .unwrap_or(false)
}
