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
  down?: boolean;
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
    getQuickControls: (withBrightness = true) => call('get_quick_controls', { withBrightness }),
    setBrightness: (value) => call<number>('set_brightness', { value }),
    setRadio: (kind, on) => call<boolean | null>('set_radio', { kind, on }),
    selectAppFile: () => call('select_app_file'),
    getAppIcon: (filePath) => call('get_app_icon', { filePath }),
    selectImageFile: async () => {
      const picked = await call<string | null>('select_image_file');
      return picked ? tauri.core.convertFileSrc(picked, 'asset') : null;
    },
    savePhoto: async (data, ext) => {
      const saved = await call<string | null>('save_photo', { data, ext });
      return saved ? tauri.core.convertFileSrc(saved, 'asset') : null;
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
    setReplaceTaskbar: (enabled) => call<void>('set_replace_taskbar', { enabled }),
    onTaskbarState: (callback) =>
      subscribe<{ enabled: boolean; fullscreen: boolean; desktopVisible: boolean }>('taskbar:state', callback),
    getDisplays: () => call('get_displays'),
    getDesktopSize: () => call('get_desktop_size'),
    trimMemory: () => call<void>('trim_memory'),
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
    setSnowFront: (front) => call<void>('set_snow_front', { front }),
    clearSnow: () => call<void>('clear_snow'),
    onSnowClear: (callback) => subscribe('snow:clear', () => callback()),
    openControlCenter: (tab) => call<void>('open_control_center', { tab }),
    controlCenterReady: () => call<{ tab: string; open: boolean }>('cc_ready'),
    showControlCenter: () => call<void>('cc_show'),
    closeControlCenter: () => call<void>('cc_close'),
    minimizeControlCenter: () => call<void>('cc_minimize'),
    toggleMaximizeControlCenter: () => call<void>('cc_toggle_maximize'),
    dragControlCenter: () => call<void>('cc_drag'),
    onControlCenterOpen: (callback) => subscribe<string>('cc:open', callback),
    onConfigSynced: (callback) => subscribe('config:synced', () => callback()),
    onAudioChanged: (callback) => subscribe<{ volume: number; isMuted: boolean }>('audio:changed', callback),
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
    const root = document.getElementById('root');
    const bare =
      !el || el === document.documentElement || el === document.body || el === root || el === root?.firstElementChild;
    if (!bare) return true;
    return Boolean((window as any).dokiiiSnowHit?.(x, y));
  };

  let lastHit = false;
  let lastDown = false;
  let lastTest = 0;
  let pending: { x: number; y: number } | null = null;
  let pendingTimer: ReturnType<typeof setTimeout> | null = null;

  let debugHit = false;
  const refreshDebug = () => {
    void call<boolean>('get_debug_hit')
      .then((on) => {
        debugHit = Boolean(on);
      })
      .catch(() => {
        debugHit = false;
      });
  };
  refreshDebug();
  setInterval(refreshDebug, 5000);

  const describeHit = (x: number, y: number) => {
    const parts = document
      .elementsFromPoint(x, y)
      .slice(0, 4)
      .map((el) => {
        const r = el.getBoundingClientRect();
        const cls = typeof el.className === 'string' ? el.className : '';
        return `${el.tagName.toLowerCase()}.${cls.split(' ').join('.')}[${Math.round(r.left)},${Math.round(r.top)} ${Math.round(r.width)}x${Math.round(r.height)}]`;
      });
    return `at=${Math.round(x)},${Math.round(y)} viewport=${window.innerWidth}x${window.innerHeight} stack=${parts.join(' > ')}`;
  };

  const testHit = (x: number, y: number) => {
    lastTest = performance.now();
    const hit = isInteractive(x, y);
    if (hit === lastHit) return;
    lastHit = hit;
    void call<void>('report_hit', { hit, info: debugHit ? describeHit(x, y) : null });
  };

  subscribe<CursorPayload>('dokiii://cursor', ({ x, y, down }) => {
    const isDown = Boolean(down);
    window.dispatchEvent(new CustomEvent('dokiii:cursor', { detail: { x, y, down: isDown } }));
    const elapsed = performance.now() - lastTest;
    if (elapsed >= 33 || isDown !== lastDown) {
      lastDown = isDown;
      pending = null;
      testHit(x, y);
      return;
    }
    pending = { x, y };
    if (pendingTimer === null) {
      pendingTimer = setTimeout(() => {
        pendingTimer = null;
        if (pending) {
          const point = pending;
          pending = null;
          testHit(point.x, point.y);
        }
      }, 33 - elapsed);
    }
  });

  return true;
}
