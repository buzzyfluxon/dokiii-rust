import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useConfigStore } from '../store/configStore';
import { DesktopWidgetType, WidgetTint } from '../../shared/constants';
import {
  Hsv,
  clamp,
  hexToHsv,
  hsvToHex,
  normalizeHex,
  tintCss,
} from '../utils/widgetTint';

const APPLE_COLORS = [
  '#ff453a',
  '#ff9f0a',
  '#ffd60a',
  '#30d158',
  '#66d4cf',
  '#64d2ff',
  '#0a84ff',
  '#5e5ce6',
  '#bf5af2',
  '#ff375f',
  '#ac8e68',
  '#8e8e93',
];

const TYPE_LABELS: Partial<Record<DesktopWidgetType, string>> = {
  'world-clock': 'World Clock',
  'date-day': 'Date & Day',
  'system-controls': 'System Controls',
  'system-monitor': 'System Monitor',
  battery: 'Battery',
  'analog-clock': 'Analog Clock',
  calendar: 'Calendar',
  'moon-phase': 'Moon Phase',
  weather: 'Weather',
  'big-date': 'Big Date',
  'digital-clock': 'Digital Clock',
  'music-card': 'Music',
  'quick-controls': 'Quick Controls',
  'photo-card': 'Photo Card',
  'car-card': 'Photo',
};

const DEFAULT_TINT: WidgetTint = { hex: '#0a84ff', hex2: '#bf5af2', gradient: false, angle: 135, strength: 0 };

export const WidgetColorStudio: React.FC = () => {
  const dock = useConfigStore((s) => s.dock);
  const updateConfig = useConfigStore((s) => s.updateConfig);
  const updateDesktopWidgetData = useConfigStore((s) => s.updateDesktopWidgetData);
  const widgets = (dock.desktopWidgets || []).filter((w) => w.type !== 'app-icon');

  const [scope, setScope] = useState<string>('all');
  const [editEnd, setEditEnd] = useState(false);
  const stored: WidgetTint | null | undefined =
    scope === 'all' ? dock.widgetTint : widgets.find((w) => w.id === scope)?.tint;
  const [draft, setDraft] = useState<WidgetTint>(stored ? { ...DEFAULT_TINT, ...stored } : DEFAULT_TINT);
  const activeHex = editEnd && draft.gradient ? draft.hex2 || draft.hex : draft.hex;
  const [hsv, setHsv] = useState<Hsv>(hexToHsv(activeHex));
  const [hexInput, setHexInput] = useState(activeHex);
  const padRef = useRef<HTMLDivElement>(null);
  const timer = useRef<number | null>(null);
  const pending = useRef<WidgetTint | null>(null);

  const write = useCallback(
    (target: string, tint: WidgetTint | null) => {
      if (target === 'all') updateConfig({ widgetTint: tint });
      else updateDesktopWidgetData(target, { tint });
    },
    [updateConfig, updateDesktopWidgetData]
  );

  const flush = useCallback(
    (target: string) => {
      if (timer.current !== null) {
        window.clearTimeout(timer.current);
        timer.current = null;
      }
      if (pending.current) {
        write(target, pending.current);
        pending.current = null;
      }
    },
    [write]
  );

  const commit = (next: WidgetTint) => {
    setDraft(next);
    pending.current = next;
    if (timer.current !== null) window.clearTimeout(timer.current);
    const target = scope;
    timer.current = window.setTimeout(() => flush(target), 90);
  };

  const switchScope = (next: string) => {
    flush(scope);
    setScope(next);
    setEditEnd(false);
  };

  useEffect(() => {
    const t = scope === 'all' ? dock.widgetTint : widgets.find((w) => w.id === scope)?.tint;
    const next = t ? { ...DEFAULT_TINT, ...t } : DEFAULT_TINT;
    setDraft(next);
    setHsv(hexToHsv(next.hex));
    setHexInput(next.hex);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  useEffect(() => () => flush(scope), [flush, scope]);

  const setActiveColor = (nextHsv: Hsv, hex?: string) => {
    const nextHex = hex ?? hsvToHex(nextHsv);
    setHsv(nextHsv);
    setHexInput(nextHex);
    if (editEnd && draft.gradient) commit({ ...draft, hex2: nextHex });
    else commit({ ...draft, hex: nextHex, strength: draft.strength === 0 ? 55 : draft.strength });
  };

  const pickFromPad = (e: React.PointerEvent<HTMLDivElement>) => {
    const rect = padRef.current?.getBoundingClientRect();
    if (!rect) return;
    const s = clamp((e.clientX - rect.left) / rect.width, 0, 1);
    const v = 1 - clamp((e.clientY - rect.top) / rect.height, 0, 1);
    setActiveColor({ h: hsv.h, s, v });
  };

  const pickSwatch = (hex: string) => setActiveColor(hexToHsv(hex), hex);

  const onHexChange = (value: string) => {
    setHexInput(value);
    const n = normalizeHex(value);
    if (n) setActiveColor(hexToHsv(n), n);
  };

  const resetScope = () => {
    flush(scope);
    write(scope, null);
    setDraft(DEFAULT_TINT);
    setHsv(hexToHsv(DEFAULT_TINT.hex));
    setHexInput(DEFAULT_TINT.hex);
    setEditEnd(false);
  };

  const applyToAll = () => {
    flush(scope);
    updateConfig({
      widgetTint: draft,
      desktopWidgets: (dock.desktopWidgets || []).map((w) => ({ ...w, tint: null })),
    });
    setScope('all');
  };

  const counts: Record<string, number> = {};
  const options = widgets.map((w) => {
    const label = TYPE_LABELS[w.type as DesktopWidgetType] || String(w.type);
    counts[label] = (counts[label] || 0) + 1;
    return { id: w.id, label, n: counts[label] };
  });

  const previewTint = tintCss(draft);
  const hueColor = hsvToHex({ h: hsv.h, s: 1, v: 1 });
  const strengthTrack = `linear-gradient(90deg, rgba(0,0,0,0), ${activeHex})`;

  return (
    <div className="cs-root">
      <div className="cs-preview">
        <div
          className={`cs-preview-card${previewTint ? ' has-tint' : ''}`}
          style={previewTint ? ({ '--widget-tint': previewTint } as React.CSSProperties) : undefined}
        >
          <span className="cs-preview-time">12:41</span>
          <span className="cs-preview-label">Preview</span>
        </div>
      </div>

      <div className="cs-controls-row">
        <select className="dokiii-select" value={scope} onChange={(e) => switchScope(e.target.value)}>
          <option value="all">All Widgets</option>
          {options.map((o) => (
            <option key={o.id} value={o.id}>
              {o.label}
              {counts[o.label] > 1 ? ` ${o.n}` : ''}
            </option>
          ))}
        </select>

        <div className="cs-segment">
          <button
            className={!draft.gradient ? 'active' : ''}
            onClick={() => {
              setEditEnd(false);
              setHsv(hexToHsv(draft.hex));
              setHexInput(draft.hex);
              commit({ ...draft, gradient: false });
            }}
          >
            Solid
          </button>
          <button
            className={draft.gradient ? 'active' : ''}
            onClick={() => commit({ ...draft, gradient: true, hex2: draft.hex2 || '#bf5af2' })}
          >
            Gradient
          </button>
        </div>
      </div>

      {draft.gradient && (
        <div className="cs-stops">
          <button className={!editEnd ? 'active' : ''} onClick={() => { setEditEnd(false); setHsv(hexToHsv(draft.hex)); setHexInput(draft.hex); }}>
            <span style={{ background: draft.hex }} />
            Start
          </button>
          <button className={editEnd ? 'active' : ''} onClick={() => { const h = draft.hex2 || draft.hex; setEditEnd(true); setHsv(hexToHsv(h)); setHexInput(h); }}>
            <span style={{ background: draft.hex2 || draft.hex }} />
            End
          </button>
        </div>
      )}

      <div className="cs-picker">
        <div
          ref={padRef}
          className="cs-pad"
          style={{ background: `linear-gradient(to top, #000, rgba(0,0,0,0)), linear-gradient(to right, #fff, ${hueColor})` }}
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            pickFromPad(e);
          }}
          onPointerMove={(e) => {
            if (e.buttons === 1) pickFromPad(e);
          }}
        >
          <span
            className="cs-pad-thumb"
            style={{ left: `${hsv.s * 100}%`, top: `${(1 - hsv.v) * 100}%`, background: activeHex }}
          />
        </div>

        <div className="cs-sliders">
          <label className="cs-slider-row">
            <span>Hue</span>
            <input
              type="range"
              min={0}
              max={360}
              value={Math.round(hsv.h)}
              className="cs-range cs-hue"
              onChange={(e) => setActiveColor({ ...hsv, h: Number(e.target.value) })}
            />
          </label>
          <label className="cs-slider-row">
            <span>Strength</span>
            <input
              type="range"
              min={0}
              max={100}
              value={draft.strength}
              className="cs-range cs-alpha"
              style={{ ['--cs-track' as string]: strengthTrack }}
              onChange={(e) => commit({ ...draft, strength: Number(e.target.value) })}
            />
          </label>
          {draft.gradient && (
            <label className="cs-slider-row">
              <span>Angle</span>
              <input
                type="range"
                min={0}
                max={360}
                value={draft.angle ?? 135}
                className="cs-range cs-angle"
                onChange={(e) => commit({ ...draft, angle: Number(e.target.value) })}
              />
            </label>
          )}
          <label className="cs-slider-row">
            <span>Hex</span>
            <input
              className="cs-hex"
              value={hexInput}
              spellCheck={false}
              maxLength={7}
              onChange={(e) => onHexChange(e.target.value)}
              onBlur={() => setHexInput(activeHex)}
            />
          </label>
        </div>
      </div>

      <div className="cs-swatches">
        {APPLE_COLORS.map((c) => (
          <button
            key={c}
            className={`cs-swatch${activeHex.toLowerCase() === c ? ' active' : ''}`}
            style={{ background: c }}
            onClick={() => pickSwatch(c)}
            title={c}
          />
        ))}
      </div>

      <div className="cs-actions">
        <button className="dokiii-btn secondary" onClick={resetScope}>
          Reset
        </button>
        {scope !== 'all' && (
          <button className="dokiii-btn primary" onClick={applyToAll}>
            Apply to All Widgets
          </button>
        )}
      </div>
      <span className="dokiii-row-desc" style={{ marginTop: '4px', display: 'block' }}>
        The color sits on top of the glass, so the blur and shine stay. {scope === 'all' ? 'Widgets with their own color keep it.' : 'This widget uses its own color until you reset it.'}
      </span>
    </div>
  );
};
