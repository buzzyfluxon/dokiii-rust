import React, { useEffect, useMemo, useState } from 'react';
import { useConfigStore } from '../../store/configStore';
import { useWidgetStore } from '../../store/widgetStore';
import { useLiquidGlassStore } from '../../store/liquidGlassStore';
import {
  WIDGET_REGISTRY,
  WIDGET_CATEGORIES,
  DESKTOP_WIDGET_DEFAULT_DIMS,
  DesktopWidgetItem,
  DesktopWidgetSize,
  DesktopWidgetType,
  WidgetDefinition,
} from '../../../shared/constants';
import { PRESET_IMAGES } from '../../../shared/presetAssets';
import { renderDesktopWidgetContent } from '../desktop-widgets/renderWidgetContent';
import { BASE_SIZE, FILL_TYPES, BARE_TYPES, MAX_TRANSPARENCY, widgetBackground } from '../../utils/widgetSurface';
import {
  IconApps,
  IconRecent,
  IconTrash,
  IconSearch,
  IconCommand,
  IconDownload,
  IconCamera,
  IconCrop,
  IconStorage,
  IconMusic,
  IconClock,
  IconGlobe,
  IconProgressRing,
  IconNote,
  IconCurrency,
  IconSettings,
  IconRefresh,
} from '../Icons';
import './widget-library.css';

type Mode = 'desktop' | 'dock';

const ICONS: Record<string, React.FC<{ size?: number }>> = {
  apps: IconApps,
  recent: IconRecent,
  trash: IconTrash,
  search: IconSearch,
  command: IconCommand,
  download: IconDownload,
  camera: IconCamera,
  crop: IconCrop,
  storage: IconStorage,
  music: IconMusic,
  clock: IconClock,
  globe: IconGlobe,
  progress: IconProgressRing,
  note: IconNote,
  currency: IconCurrency,
  settings: IconSettings,
};

const SIZE_OPTIONS: { id: DesktopWidgetSize; label: string }[] = [
  { id: 'small', label: 'Small' },
  { id: 'medium', label: 'Medium' },
  { id: 'large', label: 'Large' },
];

const WIDTH_LABELS: Record<string, string> = { compact: 'Compact', normal: 'Normal', wide: 'Wide' };
const STAGE_MAX_WIDTH = 460;
const STAGE_MAX_HEIGHT = 250;

const iconFor = (name: string): React.FC<{ size?: number }> => ICONS[name] || IconApps;

export const WidgetLibraryView: React.FC = () => {
  const {
    dock,
    addDesktopWidget,
    removeDesktopWidget,
    setDesktopWidgetSize,
    updateDesktopWidgetData,
    resetDesktopWidgetPosition,
  } = useConfigStore();
  const { enabledWidgets, toggleWidget } = useWidgetStore();
  const palette = useLiquidGlassStore((s) => s.palette);

  const [mode, setMode] = useState<Mode>('desktop');
  const [query, setQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string>(
    WIDGET_REGISTRY.find((w) => w.placement === 'desktop')?.id || ''
  );
  const [instanceId, setInstanceId] = useState<string | null>(null);
  const [draftTransparency, setDraftTransparency] = useState<number | null>(null);
  const [pendingLast, setPendingLast] = useState(false);

  const desktopWidgets: DesktopWidgetItem[] = dock.desktopWidgets || [];
  const normalizedQuery = query.trim().toLowerCase();

  const groups = useMemo(
    () =>
      WIDGET_CATEGORIES.map((cat) => ({
        cat,
        items: WIDGET_REGISTRY.filter(
          (w) =>
            w.placement === mode &&
            !w.hidden &&
            w.category === cat.id &&
            (!normalizedQuery ||
              w.name.toLowerCase().includes(normalizedQuery) ||
              w.description.toLowerCase().includes(normalizedQuery))
        ),
      })).filter((g) => g.items.length > 0),
    [mode, normalizedQuery]
  );

  const selected: WidgetDefinition | null =
    WIDGET_REGISTRY.find((w) => w.id === selectedId && w.placement === mode) || groups[0]?.items[0] || null;

  const instances =
    selected && mode === 'desktop' ? desktopWidgets.filter((w) => w.type === selected.id) : [];
  const instance = instances.find((w) => w.id === instanceId) || instances[0] || null;

  useEffect(() => {
    setInstanceId(null);
    setDraftTransparency(null);
  }, [selected?.id, mode]);

  useEffect(() => {
    setDraftTransparency(null);
  }, [instance?.id]);

  useEffect(() => {
    if (pendingLast && instances.length > 0) {
      setInstanceId(instances[instances.length - 1].id);
      setPendingLast(false);
    }
  }, [pendingLast, instances.length]);

  const countOf = (type: string) => desktopWidgets.filter((w) => w.type === type).length;

  const handleAdd = (w: WidgetDefinition) => {
    const added = addDesktopWidget(w.id as DesktopWidgetType, 'small');
    if (added) setPendingLast(true);
  };

  const handleRemove = () => {
    if (!instance) return;
    removeDesktopWidget(instance.id);
    setInstanceId(null);
  };

  const commitTransparency = (e: React.SyntheticEvent<HTMLInputElement>) => {
    if (!instance) return;
    const value = Number(e.currentTarget.value);
    if (value !== (instance.transparency || 0)) {
      updateDesktopWidgetData(instance.id, { transparency: value });
    }
  };

  const transparencyValue = draftTransparency ?? instance?.transparency ?? 0;

  const stageBackground = palette
    ? `linear-gradient(135deg, rgb(${palette.dominant.r}, ${palette.dominant.g}, ${palette.dominant.b}), rgb(${Math.round(
        palette.dominant.r * 0.4
      )}, ${Math.round(palette.dominant.g * 0.4)}, ${Math.round(palette.dominant.b * 0.4)}))`
    : 'linear-gradient(135deg, #1b2b4a, #0c1424)';

  const renderDesktopPreview = (def: WidgetDefinition) => {
    const type = def.id as DesktopWidgetType;
    const customDims = DESKTOP_WIDGET_DEFAULT_DIMS[type];
    const base: DesktopWidgetItem = instance
      ? { ...instance, transparency: transparencyValue }
      : {
          id: 'preview',
          type,
          x: 0,
          y: 0,
          size: 'medium',
          ...(type === 'car-card' ? { data: { imageUrl: PRESET_IMAGES.tower, name: 'Photo', subtitle: 'Click to change' } } : {}),
          ...(customDims ? { width: customDims.width, height: customDims.height } : {}),
        };
    const sizeBase = BASE_SIZE[base.size] || BASE_SIZE.small;
    const dims = { width: base.width ?? sizeBase.width, height: base.height ?? sizeBase.height };
    const fit = Math.min(1, STAGE_MAX_WIDTH / dims.width, STAGE_MAX_HEIGHT / dims.height);
    const isFill = FILL_TYPES.has(base.type);
    const isBare = BARE_TYPES.has(base.type);
    const uniformScale = Math.min(dims.width / sizeBase.width, dims.height / sizeBase.height);
    const glass = dock.liquidGlassEnabled && palette ? palette.desktopWidgets : null;
    const surface = isBare ? undefined : widgetBackground(base.type, base.transparency, glass?.bgRgba);

    return (
      <div className="wlv-card-box" style={{ width: dims.width * fit, height: dims.height * fit }}>
        <div
          className={`desktop-widget-container size-${base.size} type-${base.type}${isBare ? ' is-bare' : ''}`}
          style={{
            left: 0,
            top: 0,
            width: dims.width,
            height: dims.height,
            padding: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'default',
            transform: `scale(${fit})`,
            transformOrigin: 'top left',
            ...(glass && !isBare
              ? { background: glass.bgRgba, borderColor: glass.borderColor, boxShadow: glass.boxShadow }
              : {}),
            ...(surface ? { background: surface } : {}),
          }}
        >
          <div
            className="desktop-widget-content-scaler"
            style={
              isFill
                ? { width: dims.width, height: dims.height, padding: 0, flexShrink: 0 }
                : {
                    width: sizeBase.width,
                    height: sizeBase.height,
                    padding: sizeBase.padding,
                    transform: `scale(${uniformScale})`,
                    flexShrink: 0,
                  }
            }
          >
            {renderDesktopWidgetContent(base, dims, () => {})}
          </div>
        </div>
      </div>
    );
  };

  const renderDockPreview = (def: WidgetDefinition) => {
    const Icon = iconFor(def.icon);
    const neighbours = [IconApps, IconSearch, IconDownload];
    return (
      <div className="wlv-dock-mock">
        {neighbours.map((N, i) => (
          <div key={i} className="wlv-dock-tile">
            <N size={22} />
          </div>
        ))}
        <div className="wlv-dock-tile wlv-dock-tile-active">
          <Icon size={22} />
          <span className="wlv-dock-label">{def.name}</span>
        </div>
      </div>
    );
  };

  const renderDetail = () => {
    if (!selected) {
      return <div className="wlv-empty">No widgets match your search.</div>;
    }
    const isDesktop = mode === 'desktop';
    const dockEnabled = enabledWidgets.includes(selected.id);
    const supportsSizes = instance ? !FILL_TYPES.has(instance.type) : false;

    return (
      <div className="wlv-detail">
        <div className="wlv-detail-head">
          <div className="wlv-detail-titles">
            <h2 className="wlv-detail-name">{selected.name}</h2>
            <p className="wlv-detail-desc">{selected.description}</p>
          </div>
          <div className="wlv-detail-actions">
            {isDesktop ? (
              <>
                {instance && (
                  <button className="dokiii-btn danger" onClick={handleRemove}>
                    Remove
                  </button>
                )}
                <button className={`dokiii-btn ${instance ? 'secondary' : 'primary'}`} onClick={() => handleAdd(selected)}>
                  {instance ? 'Add another' : 'Add to desktop'}
                </button>
              </>
            ) : (
              <button className={`dokiii-btn ${dockEnabled ? 'danger' : 'primary'}`} onClick={() => toggleWidget(selected.id)}>
                {dockEnabled ? 'Remove from dock' : 'Add to dock'}
              </button>
            )}
          </div>
        </div>

        {isDesktop && instances.length > 1 && (
          <div className="wlv-chips">
            {instances.map((w, i) => (
              <button
                key={w.id}
                className={`wlv-chip${instance?.id === w.id ? ' on' : ''}`}
                onClick={() => setInstanceId(w.id)}
              >
                {selected.name} {i + 1}
              </button>
            ))}
          </div>
        )}

        {isDesktop && instance && (
          <div className="wlv-controls">
            <div className="wlv-control">
              <span className="wlv-label">Size</span>
              {supportsSizes ? (
                <div className="wlv-seg">
                  {SIZE_OPTIONS.map((o) => (
                    <button
                      key={o.id}
                      className={instance.size === o.id ? 'on' : ''}
                      onClick={() => setDesktopWidgetSize(instance.id, o.id)}
                    >
                      {o.label}
                    </button>
                  ))}
                </div>
              ) : (
                <span className="wlv-note">Drag the edges of this widget on the desktop to resize it.</span>
              )}
            </div>
            {instance.type !== 'app-icon' && (
              <div className="wlv-control wlv-control-grow">
                <div className="wlv-label-row">
                  <span className="wlv-label">Transparency</span>
                  <span className="wlv-label">{transparencyValue}%</span>
                </div>
                <input
                  className="wlv-slider"
                  type="range"
                  min={0}
                  max={MAX_TRANSPARENCY}
                  step={1}
                  value={transparencyValue}
                  onChange={(e) => setDraftTransparency(Number(e.target.value))}
                  onPointerUp={commitTransparency}
                  onKeyUp={commitTransparency}
                  onBlur={commitTransparency}
                />
              </div>
            )}
          </div>
        )}

        {!isDesktop && (
          <div className="wlv-controls">
            <div className="wlv-control">
              <span className="wlv-label">Width in dock</span>
              <span className="wlv-chip on">{WIDTH_LABELS[selected.defaultWidth] || 'Normal'}</span>
            </div>
          </div>
        )}

        <div className="wlv-stage" style={{ background: stageBackground }}>
          <span className="wlv-stage-tag">{isDesktop ? (instance ? 'On your desktop' : 'Preview') : 'In your dock'}</span>
          {isDesktop ? renderDesktopPreview(selected) : renderDockPreview(selected)}
        </div>

        <div className="wlv-foot">
          {isDesktop ? (
            <>
              <span>
                {instance ? 'Drag the widget on your desktop to move it.' : 'Add it to see it live on your desktop.'}
              </span>
              {instance && (
                <button className="dokiii-btn secondary" onClick={() => resetDesktopWidgetPosition(instance.id)}>
                  <IconRefresh size={12} />
                  Reset position
                </button>
              )}
            </>
          ) : (
            <span>Reorder dock widgets by dragging them in the dock.</span>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="dokiii-content-body wlv-root">
      <div className="wlv-layout">
        <div className="wlv-side">
          <div className="wlv-mode">
            <button className={mode === 'desktop' ? 'on' : ''} onClick={() => setMode('desktop')}>
              Desktop
            </button>
            <button className={mode === 'dock' ? 'on' : ''} onClick={() => setMode('dock')}>
              Dock
            </button>
          </div>
          <input
            className="wlv-search"
            type="text"
            placeholder="Search widgets"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <div className="wlv-list">
            {groups.map((g) => (
              <div key={g.cat.id} className="wlv-group">
                <div className="wlv-group-title">{g.cat.label}</div>
                {g.items.map((w) => {
                  const Icon = iconFor(w.icon);
                  const count = mode === 'desktop' ? countOf(w.id) : 0;
                  const active = mode === 'desktop' ? count > 0 : enabledWidgets.includes(w.id);
                  return (
                    <button
                      key={w.id}
                      className={`wlv-item${selected?.id === w.id ? ' sel' : ''}`}
                      onClick={() => setSelectedId(w.id)}
                    >
                      <span className="wlv-item-icon">
                        <Icon size={18} />
                      </span>
                      <span className="wlv-item-name">{w.name}</span>
                      {count > 1 && <span className="wlv-count">{count}</span>}
                      <span className={`wlv-dot${active ? ' live' : ''}`} />
                    </button>
                  );
                })}
              </div>
            ))}
            {groups.length === 0 && <div className="wlv-empty">No widgets match your search.</div>}
          </div>
        </div>
        {renderDetail()}
      </div>
    </div>
  );
};
