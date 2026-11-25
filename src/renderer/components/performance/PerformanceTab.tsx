import React from 'react';
import { useConfigStore } from '../../store/configStore';
import {
  PRESETS,
  PRESET_INFO,
  PRESET_ORDER,
  buildPatch,
  differsFromPreset,
  readProfile,
  resolveExtras,
} from '../../../shared/performance';
import type { Level, PerfKey, PerfProfile, PresetId, RefreshRate } from '../../../shared/performance';
import './performance.css';

type Choice<T> = { value: T; label: string };

const LEVELS: Choice<Level>[] = [
  { value: 'low', label: 'Low' },
  { value: 'medium', label: 'Medium' },
  { value: 'high', label: 'High' },
];

const FPS: Choice<30 | 60>[] = [
  { value: 30, label: '30' },
  { value: 60, label: '60' },
];

const REFRESH: Choice<RefreshRate>[] = [
  { value: 'relaxed', label: 'Relaxed' },
  { value: 'normal', label: 'Normal' },
  { value: 'fast', label: 'Fast' },
];

interface RowProps {
  name: string;
  desc: string;
  modified: boolean;
  children: React.ReactNode;
}

const Row: React.FC<RowProps> = ({ name, desc, modified, children }) => (
  <div className={`dokiii-row perf-row${modified ? ' is-modified' : ''}`}>
    <div className="dokiii-row-label">
      <span className="dokiii-row-name">
        {name}
        {modified && <span className="perf-dot" title="Differs from the selected preset" />}
      </span>
      <span className="dokiii-row-desc">{desc}</span>
    </div>
    {children}
  </div>
);

interface SwitchProps {
  on: boolean;
  onChange: (next: boolean) => void;
}

const Switch: React.FC<SwitchProps> = ({ on, onChange }) => (
  <div className={`dokiii-toggle ${on ? 'active' : ''}`} onClick={() => onChange(!on)}>
    <div className="dokiii-toggle-handle" />
  </div>
);

interface SegmentedProps<T> {
  value: T;
  options: Choice<T>[];
  onChange: (next: T) => void;
  disabled?: boolean;
}

function Segmented<T extends string | number>({ value, options, onChange, disabled }: SegmentedProps<T>) {
  return (
    <div className={`perf-segmented${disabled ? ' is-disabled' : ''}`}>
      {options.map((option) => (
        <button
          key={String(option.value)}
          className={option.value === value ? 'active' : ''}
          disabled={disabled}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export const PerformanceTab: React.FC = () => {
  const dock = useConfigStore((s) => s.dock);
  const updateConfig = useConfigStore((s) => s.updateConfig);

  const preset = resolveExtras(dock).preset;
  const profile = readProfile(dock);
  const changed = differsFromPreset(profile, preset);
  const isChanged = (key: PerfKey) => changed.includes(key);

  const apply = (id: PresetId) => {
    updateConfig(buildPatch(dock, PRESETS[id], id));
  };

  const set = (changes: Partial<PerfProfile>) => {
    updateConfig(buildPatch(dock, changes));
  };

  return (
    <div className="dokiii-content-body">
      <div className="dokiii-card perf-presets-card">
        <span className="dokiii-card-title">Performance preset</span>
        <div className="perf-presets">
          {PRESET_ORDER.map((id) => (
            <button
              key={id}
              className={`perf-preset${id === preset ? ' active' : ''}`}
              onClick={() => apply(id)}
            >
              <span className="perf-preset-name">{PRESET_INFO[id].label}</span>
              <span className="perf-preset-blurb">{PRESET_INFO[id].blurb}</span>
            </button>
          ))}
        </div>
        <div className="perf-status">
          {changed.length === 0 ? (
            <span>
              Using the {PRESET_INFO[preset].label} preset exactly.
            </span>
          ) : (
            <>
              <span>
                {PRESET_INFO[preset].label} with {changed.length} custom {changed.length === 1 ? 'setting' : 'settings'}.
              </span>
              <button className="perf-reset" onClick={() => apply(preset)}>
                Reset to {PRESET_INFO[preset].label}
              </button>
            </>
          )}
        </div>
      </div>

      <div className="dokiii-card">
        <span className="dokiii-card-title">Look and feel</span>
        <Row
          name="Glass blur"
          desc="Frosted blur behind the dock, widgets and windows. The most expensive effect on weaker GPUs."
          modified={isChanged('glassBlur')}
        >
          <Switch on={profile.glassBlur} onChange={(v) => set({ glassBlur: v })} />
        </Row>
        <Row
          name="Liquid Glass"
          desc="Tint the glass from your wallpaper"
          modified={isChanged('liquidGlass')}
        >
          <Switch on={profile.liquidGlass} onChange={(v) => set({ liquidGlass: v })} />
        </Row>
        <Row
          name="Dock magnification"
          desc="Icons grow as the cursor passes over them"
          modified={isChanged('magnification')}
        >
          <Switch on={profile.magnification} onChange={(v) => set({ magnification: v })} />
        </Row>
        <Row
          name="Live icons"
          desc="Clock and calendar icons that update by themselves"
          modified={isChanged('liveIcons')}
        >
          <Switch on={profile.liveIcons} onChange={(v) => set({ liveIcons: v })} />
        </Row>
      </div>

      <div className="dokiii-card">
        <span className="dokiii-card-title">Music ambient</span>
        <Row
          name="Music ambient"
          desc="Soft gradients around the dock, tinted by the album art"
          modified={isChanged('ambientEnabled')}
        >
          <Switch on={profile.ambientEnabled} onChange={(v) => set({ ambientEnabled: v })} />
        </Row>
        <Row
          name="Ambient intensity"
          desc="How strong the glow feels"
          modified={isChanged('ambientIntensity')}
        >
          <Segmented
            value={profile.ambientIntensity}
            options={LEVELS}
            disabled={!profile.ambientEnabled}
            onChange={(v) => set({ ambientIntensity: v })}
          />
        </Row>
        <Row
          name="Ambient motion"
          desc="Let the gradients drift while music plays. Off keeps the glow still and uses far less CPU."
          modified={isChanged('ambientMotion')}
        >
          <Switch
            on={profile.ambientMotion}
            onChange={(v) => set({ ambientMotion: v })}
          />
        </Row>
      </div>

      <div className="dokiii-card">
        <span className="dokiii-card-title">Snowfall</span>
        <Row name="Snowfall" desc="Snow that settles on the dock, widgets and windows" modified={isChanged('snowEnabled')}>
          <Switch on={profile.snowEnabled} onChange={(v) => set({ snowEnabled: v })} />
        </Row>
        <Row name="Snow amount" desc="How many flakes fall" modified={isChanged('snowIntensity')}>
          <Segmented
            value={profile.snowIntensity}
            options={LEVELS}
            disabled={!profile.snowEnabled}
            onChange={(v) => set({ snowIntensity: v })}
          />
        </Row>
        <Row name="Snowballs" desc="Draggable snowballs and snowmen" modified={isChanged('snowballs')}>
          <Switch on={profile.snowballs} onChange={(v) => set({ snowballs: v })} />
        </Row>
        <Row name="Snow on windows" desc="Track open windows so snow can settle on their title bars" modified={isChanged('snowOnWindows')}>
          <Switch on={profile.snowOnWindows} onChange={(v) => set({ snowOnWindows: v })} />
        </Row>
        <Row name="Snow frame rate" desc="Frames per second for the snow animation" modified={isChanged('frameRate')}>
          <Segmented
            value={profile.frameRate}
            options={FPS}
            disabled={!profile.snowEnabled}
            onChange={(v) => set({ frameRate: v })}
          />
        </Row>
      </div>

      <div className="dokiii-card">
        <span className="dokiii-card-title">Background activity</span>
        <Row
          name="Refresh rate"
          desc="How often widgets, Halo and music info check for updates in the background"
          modified={isChanged('refreshRate')}
        >
          <Segmented value={profile.refreshRate} options={REFRESH} onChange={(v) => set({ refreshRate: v })} />
        </Row>
        <Row name="Halo reduced motion" desc="Skip the spring morphing animations" modified={isChanged('haloReducedMotion')}>
          <Switch on={profile.haloReducedMotion} onChange={(v) => set({ haloReducedMotion: v })} />
        </Row>
        <Row name="Halo download alerts" desc="Watch your Downloads folder for new files" modified={isChanged('haloDownloads')}>
          <Switch on={profile.haloDownloads} onChange={(v) => set({ haloDownloads: v })} />
        </Row>
        <Row name="Halo screenshot alerts" desc="Watch your Screenshots folder for new captures" modified={isChanged('haloScreenshots')}>
          <Switch on={profile.haloScreenshots} onChange={(v) => set({ haloScreenshots: v })} />
        </Row>
      </div>
    </div>
  );
};

export default PerformanceTab;
