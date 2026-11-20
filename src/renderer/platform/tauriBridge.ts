import type { ElectronAPI, UpdaterStatus, WallpaperPalette } from '../types/electron';

interface TauriGlobal {
  core: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
    convertFileSrc(path: string, protocol?: string): string;
  };
  event: {
    listen<T>(name: string, handler: (event: { payload: T }) => void): Promise<() => void>;
  };
}

declare global {
  interface Window {
    __TAURI__?: TauriGlobal;
  }
}

interface CursorPayload {
  x: number;
  y: number;
}

export function installTauriBridge(): boolean {
  const tauri = window.__TAURI__;
  if (!tauri || window.electronAPI) return false;

  const call = <T>(command: string, args?: Record<string, unknown>) => tauri.core.invoke<T>(command, args);

  const subscribe = <T>(name: string, callback: (payload: T) => void) => {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    tauri.event
      .listen<T>(name, (event) => callback(event.payload))
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  };

  const api: ElectronAPI = {
    getStorageInfo: () => call('get_storage_info'),
    getRecycleBinCount: () => call<number>('recycle_bin_count'),
    emptyRecycleBin: () => call<void>('recycle_bin_empty'),
    openRecycleBin: () => call<void>('open_target', { target: 'shell:RecycleBinFolder' }),
    getInstalledApps: () => call('get_installed_apps'),
    getSystemMetrics: () => call('get_system_metrics'),
    getBatteryStatus: () => call('get_battery_status'),
    getAudioVolume: () => call('get_audio_volume'),
    setAudioVolume: (volume) => call('set_audio_volume', { volume }),
    toggleAudioMute: () => call<boolean>('toggle_audio_mute'),
    selectAppFile: () => call('select_app_file'),
    getAppIcon: (filePath) => call('get_app_icon', { filePath }),
    selectImageFile: async () => {
      const picked = await call<string | null>('select_image_file');
      return picked ? tauri.core.convertFileSrc(picked, 'asset') : null;
    },
    cacheImage: (url) => call('cache_image', { url }),
    fetchFavicon: (url) => call('fetch_favicon', { url }),
    pickIconImage: () => call('pick_icon_image'),
    getRecentItems: () => call('get_recent_items'),
    getDownloads: () => call('get_downloads'),
    getRecentScreenshots: () => call('get_recent_screenshots'),
    searchFiles: (query) => call('search_files', { query }),
    launchApp: (appPath) => call<void>('open_target', { target: appPath }),
    openPath: (filePath) => call<void>('open_target', { target: filePath }),
    openUrl: (url) => call<void>('open_target', { target: url }),
    runCommand: (command) => call<string>('run_command', { command }),
    getMediaInfo: () => call('get_media_info'),
    mediaControl: (action) => call<void>('media_control', { action }),
    mediaSeek: (seconds) => call<void>('media_seek', { seconds }),
    captureScreenshot: (mode) => call<string>('capture_screenshot', { mode }),
    getConfig: () => call('config_get'),
    saveConfig: (config) => call<void>('config_save', { config }),
    setAutoHide: (value) => call<void>('set_auto_hide', { value }),
    getDisplays: () => call('get_displays'),
    showWidgetLibrary: () => call<void>('show_widget_library'),
    showSettings: () => call<void>('show_settings'),
    setModalOpen: (isOpen) => call<void>('set_modal_open', { open: isOpen }),
    setPopoverOpen: (isOpen) => call<void>('set_popover_open', { open: isOpen }),
    setIgnoreMouseEvents: async () => {},
    hideWindow: () => call<void>('hide_window'),
    quitApp: () => call<void>('quit_app'),
    getLaunchOnStartup: () => call<boolean>('get_launch_on_startup'),
    setLaunchOnStartup: (enabled) => call<boolean>('set_launch_on_startup', { enabled }),
    onDockConfigChanged: (callback) => subscribe('dock:configChanged', callback as (payload: unknown) => void),
    onShowWidgetLibrary: (callback) => subscribe('dock:showWidgetLibrary', () => callback()),
    onShowSettings: (callback) => subscribe('dock:showSettings', () => callback()),
    onShowApp: (callback) => subscribe('dock:showApp', () => callback()),
    onCloseApp: (callback) => subscribe('dock:closeApp', () => callback()),
    onToggleDock: (callback) => subscribe('dock:toggleVisibility', () => callback()),
    showUninstall: () => call<void>('show_uninstall'),
    getScreenshotsDir: () => call<string>('get_screenshots_dir'),
    getWallpaperColors: () => call<WallpaperPalette>('get_wallpaper_colors'),
    onWallpaperColorsUpdated: (callback) => subscribe('wallpaper:colors-updated', callback),
    setSnowActive: (active) => call<void>('set_snow_active', { active }),
    onSnowDesktopState: (callback) => subscribe('snow:desktopState', callback),
    appReady: () => {
      void call<void>('app_ready');
    },
    checkForUpdates: (manual) => call<void>('check_for_updates', { manual: Boolean(manual) }),
    startUpdateDownload: () => call<void>('start_update_download'),
    installUpdate: () => call<void>('install_update'),
    dismissUpdate: (version) => call<void>('dismiss_update', { version }),
    getUpdaterStatus: () => call<UpdaterStatus>('get_updater_status'),
    onUpdaterStatus: (callback) => subscribe('updater:status', callback),
  };

  window.electronAPI = api;

  const isInteractive = (x: number, y: number) => {
    const el = document.elementFromPoint(x, y);
    if (!el || el === document.documentElement || el === document.body) return false;
    const root = document.getElementById('root');
    if (el === root || el === root?.firstElementChild) return false;
    return true;
  };

  let lastHit = false;
  subscribe<CursorPayload>('dokiii://cursor', ({ x, y }) => {
    window.dispatchEvent(new CustomEvent('dokiii:cursor', { detail: { x, y } }));
    const hit = isInteractive(x, y);
    if (hit === lastHit) return;
    lastHit = hit;
    void call<void>('report_hit', { hit });
  });

  return true;
}
