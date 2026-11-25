import type { DockConfig } from './constants';

export type PresetId = 'potato' | 'low' | 'medium' | 'high' | 'ultra';
export type Level = 'low' | 'medium' | 'high';
export type RefreshRate = 'relaxed' | 'normal' | 'fast';

export interface PerformanceExtras {
  preset: PresetId;
  glassBlur: boolean;
  ambientMotion: boolean;
  frameRate: 30 | 60;
  refreshRate: RefreshRate;
}

export const DEFAULT_PERFORMANCE: PerformanceExtras = {
  preset: 'medium',
  glassBlur: true,
  ambientMotion: false,
  frameRate: 60,
  refreshRate: 'normal',
};

export interface PerfProfile {
  glassBlur: boolean;
  liquidGlass: boolean;
  magnification: boolean;
  liveIcons: boolean;
  ambientEnabled: boolean;
  ambientIntensity: Level;
  ambientMotion: boolean;
  snowEnabled: boolean;
  snowIntensity: Level;
  snowballs: boolean;
  snowOnWindows: boolean;
  frameRate: 30 | 60;
  haloReducedMotion: boolean;
  haloDownloads: boolean;
  haloScreenshots: boolean;
  refreshRate: RefreshRate;
}

export type PerfKey = keyof PerfProfile;

export const PRESET_ORDER: PresetId[] = ['potato', 'low', 'medium', 'high', 'ultra'];

export const PRESET_INFO: Record<PresetId, { label: string; blurb: string }> = {
  potato: { label: 'Potato', blurb: 'Lightest possible. Effects off.' },
  low: { label: 'Low', blurb: 'Light snow, no blur.' },
  medium: { label: 'Medium', blurb: 'Balanced. The default.' },
  high: { label: 'High', blurb: 'Full glass and ambient glow.' },
  ultra: { label: 'Ultra', blurb: 'Everything, fully animated.' },
};

export const PRESETS: Record<PresetId, PerfProfile> = {
  potato: {
    glassBlur: false,
    liquidGlass: false,
    magnification: false,
    liveIcons: false,
    ambientEnabled: false,
    ambientIntensity: 'low',
    ambientMotion: false,
    snowEnabled: false,
    snowIntensity: 'low',
    snowballs: false,
    snowOnWindows: false,
    frameRate: 30,
    haloReducedMotion: true,
    haloDownloads: false,
    haloScreenshots: false,
    refreshRate: 'relaxed',
  },
  low: {
    glassBlur: false,
    liquidGlass: false,
    magnification: true,
    liveIcons: false,
    ambientEnabled: false,
    ambientIntensity: 'low',
    ambientMotion: false,
    snowEnabled: true,
    snowIntensity: 'low',
    snowballs: false,
    snowOnWindows: false,
    frameRate: 30,
    haloReducedMotion: true,
    haloDownloads: false,
    haloScreenshots: false,
    refreshRate: 'relaxed',
  },
  medium: {
    glassBlur: true,
    liquidGlass: false,
    magnification: true,
    liveIcons: true,
    ambientEnabled: false,
    ambientIntensity: 'medium',
    ambientMotion: false,
    snowEnabled: true,
    snowIntensity: 'medium',
    snowballs: true,
    snowOnWindows: true,
    frameRate: 60,
    haloReducedMotion: false,
    haloDownloads: true,
    haloScreenshots: true,
    refreshRate: 'normal',
  },
  high: {
    glassBlur: true,
    liquidGlass: true,
    magnification: true,
    liveIcons: true,
    ambientEnabled: true,
    ambientIntensity: 'high',
    ambientMotion: false,
    snowEnabled: true,
    snowIntensity: 'high',
    snowballs: true,
    snowOnWindows: true,
    frameRate: 60,
    haloReducedMotion: false,
    haloDownloads: true,
    haloScreenshots: true,
    refreshRate: 'normal',
  },
  ultra: {
    glassBlur: true,
    liquidGlass: true,
    magnification: true,
    liveIcons: true,
    ambientEnabled: true,
    ambientIntensity: 'high',
    ambientMotion: true,
    snowEnabled: true,
    snowIntensity: 'high',
    snowballs: true,
    snowOnWindows: true,
    frameRate: 60,
    haloReducedMotion: false,
    haloDownloads: true,
    haloScreenshots: true,
    refreshRate: 'fast',
  },
};

const REFRESH_MULTIPLIER: Record<RefreshRate, number> = {
  relaxed: 3,
  normal: 1,
  fast: 0.6,
};

export function refreshMultiplier(rate: RefreshRate | undefined): number {
  return REFRESH_MULTIPLIER[rate || 'normal'] ?? 1;
}

export function resolveExtras(dock: Pick<DockConfig, 'performance'> | { performance?: Partial<PerformanceExtras> }): PerformanceExtras {
  return { ...DEFAULT_PERFORMANCE, ...(dock.performance || {}) };
}

export function readProfile(dock: DockConfig): PerfProfile {
  const extras = resolveExtras(dock);
  return {
    glassBlur: extras.glassBlur,
    liquidGlass: Boolean(dock.liquidGlassEnabled),
    magnification: Boolean(dock.magnification),
    liveIcons: dock.liveIcons !== false,
    ambientEnabled: Boolean(dock.musicAmbient?.enabled),
    ambientIntensity: dock.musicAmbient?.intensity || 'medium',
    ambientMotion: extras.ambientMotion,
    snowEnabled: dock.snow?.enabled !== false,
    snowIntensity: dock.snow?.intensity || 'medium',
    snowballs: dock.snow?.snowballs !== false,
    snowOnWindows: dock.snow?.onWindows !== false,
    frameRate: extras.frameRate,
    haloReducedMotion: Boolean(dock.halo?.reducedMotion),
    haloDownloads: dock.halo?.showDownloads !== false,
    haloScreenshots: dock.halo?.showScreenshots !== false,
    refreshRate: extras.refreshRate,
  };
}

const AMBIENT_DEFAULTS: { enabled: boolean; intensity: Level } = { enabled: false, intensity: 'medium' };

const SNOW_DEFAULTS: { enabled: boolean; intensity: Level; snowballs: boolean; onWindows: boolean; inFront: boolean } = {
  enabled: true,
  intensity: 'medium',
  snowballs: true,
  onWindows: true,
  inFront: false,
};

const HALO_DEFAULTS: {
  enabled: boolean;
  displayMode: 'always' | 'active';
  reducedMotion: boolean;
  showMusic: boolean;
  showSystemEvents: boolean;
  showDownloads: boolean;
  showScreenshots: boolean;
} = {
  enabled: true,
  displayMode: 'always',
  reducedMotion: false,
  showMusic: true,
  showSystemEvents: true,
  showDownloads: true,
  showScreenshots: true,
};

export function buildPatch(dock: DockConfig, changes: Partial<PerfProfile>, preset?: PresetId): Partial<DockConfig> {
  const patch: Partial<DockConfig> = {};
  const has = <K extends PerfKey>(key: K) => changes[key] !== undefined;

  const extras = resolveExtras(dock);
  let extrasTouched = false;
  if (preset) {
    extras.preset = preset;
    extrasTouched = true;
  }
  if (has('glassBlur')) {
    extras.glassBlur = changes.glassBlur as boolean;
    extrasTouched = true;
  }
  if (has('ambientMotion')) {
    extras.ambientMotion = changes.ambientMotion as boolean;
    extrasTouched = true;
  }
  if (has('frameRate')) {
    extras.frameRate = changes.frameRate as 30 | 60;
    extrasTouched = true;
  }
  if (has('refreshRate')) {
    extras.refreshRate = changes.refreshRate as RefreshRate;
    extrasTouched = true;
  }
  if (extrasTouched) patch.performance = extras;

  if (has('liquidGlass')) patch.liquidGlassEnabled = changes.liquidGlass as boolean;
  if (has('magnification')) patch.magnification = changes.magnification as boolean;
  if (has('liveIcons')) patch.liveIcons = changes.liveIcons as boolean;

  if (has('ambientEnabled') || has('ambientIntensity')) {
    const ambient = { ...AMBIENT_DEFAULTS, ...(dock.musicAmbient || {}) };
    if (has('ambientEnabled')) ambient.enabled = changes.ambientEnabled as boolean;
    if (has('ambientIntensity')) ambient.intensity = changes.ambientIntensity as Level;
    patch.musicAmbient = ambient;
  }

  if (has('snowEnabled') || has('snowIntensity') || has('snowballs') || has('snowOnWindows')) {
    const snow = { ...SNOW_DEFAULTS, ...(dock.snow || {}) };
    if (has('snowEnabled')) snow.enabled = changes.snowEnabled as boolean;
    if (has('snowIntensity')) snow.intensity = changes.snowIntensity as Level;
    if (has('snowballs')) snow.snowballs = changes.snowballs as boolean;
    if (has('snowOnWindows')) snow.onWindows = changes.snowOnWindows as boolean;
    patch.snow = snow;
  }

  if (has('haloReducedMotion') || has('haloDownloads') || has('haloScreenshots')) {
    const halo = { ...HALO_DEFAULTS, ...(dock.halo || {}) };
    if (has('haloReducedMotion')) halo.reducedMotion = changes.haloReducedMotion as boolean;
    if (has('haloDownloads')) halo.showDownloads = changes.haloDownloads as boolean;
    if (has('haloScreenshots')) halo.showScreenshots = changes.haloScreenshots as boolean;
    patch.halo = halo;
  }

  return patch;
}

export function differsFromPreset(profile: PerfProfile, preset: PresetId): PerfKey[] {
  const base = PRESETS[preset];
  return (Object.keys(base) as PerfKey[]).filter((key) => profile[key] !== base[key]);
}
