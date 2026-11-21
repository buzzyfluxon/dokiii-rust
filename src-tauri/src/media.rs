use std::sync::Mutex;
use std::time::{Duration, Instant};

use base64::Engine;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    title: String,
    artist: String,
    album_title: Option<String>,
    album_art: Option<String>,
    is_playing: bool,
    position: u64,
    duration: u64,
    app_name: String,
}

struct Cache {
    info: Option<MediaInfo>,
    fetched_at: Option<Instant>,
}

static CACHE: Mutex<Cache> = Mutex::new(Cache {
    info: None,
    fetched_at: None,
});

static FETCH_STARTED: Mutex<Option<Instant>> = Mutex::new(None);

fn lock_fetch() -> std::sync::MutexGuard<'static, Option<Instant>> {
    match FETCH_STARTED.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn lock_cache() -> std::sync::MutexGuard<'static, Cache> {
    match CACHE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn invalidate() {
    let mut cache = lock_cache();
    cache.info = None;
    cache.fetched_at = None;
}

fn app_name_for(source: &str) -> &'static str {
    let lower = source.to_lowercase();
    if lower.contains("spotify") {
        "spotify"
    } else if lower.contains("apple") || lower.contains("itunes") {
        "applemusic"
    } else if lower.contains("chrome") || lower.contains("brave") || lower.contains("edge") {
        "browser"
    } else {
        "media"
    }
}

#[cfg(windows)]
mod winrt {
    use super::*;
    use windows::core::Interface;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionMediaProperties as Props,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    use windows::Storage::Streams::{DataReader, IInputStream};
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    fn init() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    fn manager() -> Option<Manager> {
        init();
        Manager::RequestAsync().ok()?.get().ok()
    }

    fn is_playing(session: &Session) -> bool {
        session
            .GetPlaybackInfo()
            .ok()
            .and_then(|info| info.PlaybackStatus().ok())
            .map(|status| status == Status::Playing)
            .unwrap_or(false)
    }

    fn is_music(source: &str) -> bool {
        source.contains("spotify") || source.contains("applemusic") || source.contains("music")
    }

    fn pick(manager: &Manager) -> Option<Session> {
        if let Ok(current) = manager.GetCurrentSession() {
            if is_playing(&current) {
                return Some(current);
            }
        }
        let sessions = manager.GetSessions().ok()?;
        let mut music: Option<Session> = None;
        let mut any_playing: Option<Session> = None;
        let mut first: Option<Session> = None;
        for session in sessions {
            let source = session
                .SourceAppUserModelId()
                .map(|id| id.to_string().to_lowercase())
                .unwrap_or_default();
            let playing = is_playing(&session);
            if playing && is_music(&source) {
                return Some(session);
            }
            if playing && any_playing.is_none() {
                any_playing = Some(session.clone());
            }
            if is_music(&source) && music.is_none() {
                music = Some(session.clone());
            }
            if first.is_none() {
                first = Some(session);
            }
        }
        any_playing
            .or(music)
            .or_else(|| manager.GetCurrentSession().ok())
            .or(first)
    }

    fn thumbnail(props: &Props) -> Option<String> {
        let reference = props.Thumbnail().ok()?;
        let stream = reference.OpenReadAsync().ok()?.get().ok()?;
        let size = stream.Size().ok()? as u32;
        if size == 0 || size > 8_000_000 {
            return None;
        }
        let input: IInputStream = stream.cast().ok()?;
        let reader = DataReader::CreateDataReader(&input).ok()?;
        reader.LoadAsync(size).ok()?.get().ok()?;
        let mut bytes = vec![0u8; size as usize];
        reader.ReadBytes(&mut bytes).ok()?;
        let mime = stream
            .ContentType()
            .map(|value| value.to_string())
            .unwrap_or_else(|_| "image/jpeg".to_string());
        let mime = if mime.starts_with("image/") { mime } else { "image/jpeg".to_string() };
        Some(format!(
            "data:{};base64,{}",
            mime,
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    }

    pub fn fetch(previous: Option<&MediaInfo>) -> Option<MediaInfo> {
        let manager = manager()?;
        let session = pick(&manager)?;
        let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
        let title = props.Title().map(|v| v.to_string()).unwrap_or_default();
        let artist = props.Artist().map(|v| v.to_string()).unwrap_or_default();
        let album = props.AlbumTitle().map(|v| v.to_string()).unwrap_or_default();
        let source = session
            .SourceAppUserModelId()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let playing = session
            .GetPlaybackInfo()
            .ok()
            .and_then(|info| info.PlaybackStatus().ok())
            .map(|status| status == Status::Playing)
            .unwrap_or(false);

        let (position, duration) = match session.GetTimelineProperties() {
            Ok(timeline) => {
                let pos = timeline.Position().map(|t| t.Duration).unwrap_or(0);
                let end = timeline.EndTime().map(|t| t.Duration).unwrap_or(0);
                ((pos.max(0) / 10_000_000) as u64, (end.max(0) / 10_000_000) as u64)
            }
            Err(_) => (0, 0),
        };

        let same_track = previous
            .map(|p| {
                p.title == title && p.artist == artist && p.album_title.as_deref().unwrap_or("") == album
            })
            .unwrap_or(false);
        let reused = if same_track { previous.and_then(|p| p.album_art.clone()) } else { None };
        let album_art = match reused {
            Some(art) => Some(art),
            None => thumbnail(&props),
        };

        Some(MediaInfo {
            title,
            artist,
            album_title: if album.is_empty() { None } else { Some(album) },
            album_art,
            is_playing: playing,
            position,
            duration,
            app_name: app_name_for(&source).to_string(),
        })
    }

    pub fn seek(seconds: u64) {
        if let Some(manager) = manager() {
            if let Some(session) = pick(&manager) {
                if let Ok(op) = session.TryChangePlaybackPositionAsync((seconds as i64) * 10_000_000) {
                    let _ = op.get();
                }
            }
        }
    }
}

#[tauri::command]
pub async fn get_media_info() -> Option<MediaInfo> {
    let (cached, previous) = {
        let cache = lock_cache();
        let ttl = if cache.info.as_ref().map(|i| i.is_playing).unwrap_or(false) {
            Duration::from_millis(1000)
        } else {
            Duration::from_millis(2000)
        };
        let fresh = cache.fetched_at.map(|at| at.elapsed() < ttl).unwrap_or(false);
        (if fresh { Some(cache.info.clone()) } else { None }, cache.info.clone())
    };
    if let Some(info) = cached {
        return info;
    }

    {
        let mut started = lock_fetch();
        if let Some(at) = *started {
            if at.elapsed() < Duration::from_secs(6) {
                return previous;
            }
        }
        *started = Some(Instant::now());
    }

    let fetched = tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        {
            winrt::fetch(previous.as_ref())
        }
        #[cfg(not(windows))]
        {
            let _ = previous;
            None
        }
    })
    .await
    .unwrap_or(None);
    *lock_fetch() = None;

    let mut cache = lock_cache();
    cache.info = fetched.clone();
    cache.fetched_at = Some(Instant::now());
    fetched
}

#[tauri::command]
pub async fn media_control(action: String) {
    invalidate();
    let vk: u8 = match action.as_str() {
        "play-pause" => 179,
        "next" => 176,
        "previous" => 177,
        _ => return,
    };
    let _ = tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP};
            keybd_event(vk, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
        }
        #[cfg(not(windows))]
        {
            let _ = vk;
        }
    })
    .await;
}

#[tauri::command]
pub async fn media_seek(seconds: f64) {
    invalidate();
    let target = seconds.max(0.0).floor() as u64;
    let _ = tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        winrt::seek(target);
        #[cfg(not(windows))]
        {
            let _ = target;
        }
    })
    .await;
}
