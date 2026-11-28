import React, { useCallback, useEffect, useRef, useState } from 'react';
import { FitBox } from './FitBox';

interface QuickControlsWidgetProps {
  width: number;
  height: number;
}

type RadioKind = 'wifi' | 'bluetooth';

const SunIcon: React.FC<{ color: string }> = ({ color }) => (
  <svg width="26" height="26" viewBox="0 0 26 26" fill="none">
    <circle cx="13" cy="13" r="5" fill={color} />
    <g stroke={color} strokeWidth="1.8" strokeLinecap="round">
      <line x1="13" y1="2.5" x2="13" y2="5" />
      <line x1="13" y1="21" x2="13" y2="23.5" />
      <line x1="2.5" y1="13" x2="5" y2="13" />
      <line x1="21" y1="13" x2="23.5" y2="13" />
      <line x1="5.6" y1="5.6" x2="7.4" y2="7.4" />
      <line x1="18.6" y1="18.6" x2="20.4" y2="20.4" />
      <line x1="5.6" y1="20.4" x2="7.4" y2="18.6" />
      <line x1="18.6" y1="7.4" x2="20.4" y2="5.6" />
    </g>
  </svg>
);

const WifiIcon: React.FC = () => (
  <svg width="30" height="30" viewBox="0 0 30 30" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
    <path d="M3.5 11.2a17 17 0 0 1 23 0" />
    <path d="M8 16a10.5 10.5 0 0 1 14 0" />
    <path d="M12.3 20.6a4.2 4.2 0 0 1 5.4 0" />
    <circle cx="15" cy="25" r="1.4" fill="currentColor" stroke="none" />
  </svg>
);

const BluetoothIcon: React.FC = () => (
  <svg width="26" height="30" viewBox="0 0 26 30" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M5 9.5l15 11-7 6.5V3l7 6.5-15 11" />
  </svg>
);

const GlassButton: React.FC<{
  active: boolean;
  available: boolean;
  label: string;
  onToggle: () => void;
  children: React.ReactNode;
}> = ({ active, available, label, onToggle, children }) => (
  <button
    type="button"
    aria-label={label}
    aria-pressed={active}
    title={available ? `${label} ${active ? 'on' : 'off'}` : `${label} not available`}
    disabled={!available}
    onPointerDown={(e) => e.stopPropagation()}
    onClick={(e) => {
      e.stopPropagation();
      onToggle();
    }}
    style={{
      width: '70px',
      height: '70px',
      borderRadius: '50%',
      border: 'none',
      background: active ? 'rgba(255,255,255,0.88)' : 'rgba(255,255,255,0.14)',
      color: active ? '#1c1c1e' : 'rgba(255,255,255,0.6)',
      boxShadow: 'none',
      cursor: available ? 'pointer' : 'not-allowed',
      opacity: available ? 1 : 0.4,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      padding: 0,
      outline: 'none',
      transition: 'background 0.2s ease, color 0.2s ease, transform 0.12s ease',
    }}
  >
    {children}
  </button>
);

export const QuickControlsWidget: React.FC<QuickControlsWidgetProps> = ({ width, height }) => {
  const [brightness, setBrightnessState] = useState<number>(-2);
  const [wifi, setWifi] = useState<boolean | null>(null);
  const [bluetooth, setBluetooth] = useState<boolean | null>(null);
  const [dragging, setDragging] = useState(false);

  const pillRef = useRef<HTMLDivElement>(null);
  const draggingRef = useRef(false);
  const inFlight = useRef(false);
  const queued = useRef<number | null>(null);
  const busyRadios = useRef<Set<RadioKind>>(new Set());

  const lastRead = useRef(0);
  const refresh = useCallback(async (withBrightness: boolean) => {
    lastRead.current = Date.now();
    try {
      const state = await window.electronAPI?.getQuickControls?.(withBrightness);
      if (!state) return;
      if (withBrightness && state.brightness !== -2 && !draggingRef.current && !inFlight.current) {
        setBrightnessState(state.brightness);
      }
      if (!busyRadios.current.has('wifi')) setWifi(state.wifi);
      if (!busyRadios.current.has('bluetooth')) setBluetooth(state.bluetooth);
    } catch (_) {}
  }, []);

  useEffect(() => {
    void refresh(false);
  }, [refresh]);

  const handleEnter = useCallback(() => {
    if (draggingRef.current || Date.now() - lastRead.current < 2000) return;
    void refresh(true);
  }, [refresh]);

  const sendBrightness = useCallback(async (value: number) => {
    if (inFlight.current) {
      queued.current = value;
      return;
    }
    inFlight.current = true;
    try {
      await window.electronAPI?.setBrightness?.(value);
    } catch (_) {}
    inFlight.current = false;
    if (queued.current !== null) {
      const next = queued.current;
      queued.current = null;
      sendBrightness(next);
    }
  }, []);

  const supported = brightness !== -1;
  const shown = brightness >= 0 ? brightness : 100;

  const valueFromPointer = (e: React.PointerEvent<HTMLDivElement>) => {
    const rect = pillRef.current?.getBoundingClientRect();
    if (!rect) return shown;
    const ratio = 1 - (e.clientY - rect.top) / rect.height;
    return Math.round(Math.max(0, Math.min(1, ratio)) * 100);
  };

  const handleDown = (e: React.PointerEvent<HTMLDivElement>) => {
    e.stopPropagation();
    if (!supported) return;
    if (brightness === -2) {
      void refresh(true);
      return;
    }
    draggingRef.current = true;
    setDragging(true);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    const value = valueFromPointer(e);
    setBrightnessState(value);
    sendBrightness(value);
  };

  const handleMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!draggingRef.current) return;
    const value = valueFromPointer(e);
    setBrightnessState(value);
    sendBrightness(value);
  };

  const handleUp = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!draggingRef.current) return;
    draggingRef.current = false;
    setDragging(false);
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch (_) {}
    sendBrightness(valueFromPointer(e));
  };

  const toggleRadio = async (kind: RadioKind) => {
    const current = kind === 'wifi' ? wifi : bluetooth;
    if (current === null || busyRadios.current.has(kind)) return;
    const setter = kind === 'wifi' ? setWifi : setBluetooth;
    busyRadios.current.add(kind);
    setter(!current);
    try {
      const result = await window.electronAPI?.setRadio?.(kind, !current);
      setter(result === null || result === undefined ? current : result);
    } catch (_) {
      setter(current);
    }
    busyRadios.current.delete(kind);
  };

  const iconOnFill = shown > 16;

  return (
    <FitBox width={width} height={height} designWidth={190} designHeight={190}>
      <div
        onPointerEnter={handleEnter}
        style={{
          width: '190px',
          height: '190px',
          boxSizing: 'border-box',
          padding: '20px',
          display: 'flex',
          alignItems: 'stretch',
          justifyContent: 'center',
          gap: '18px',
        }}
      >
        <div
          ref={pillRef}
          onPointerDown={handleDown}
          onPointerMove={handleMove}
          onPointerUp={handleUp}
          onPointerCancel={handleUp}
          title={supported ? `Brightness ${shown}%` : 'Brightness control is not available on this display'}
          style={{
            width: '58px',
            height: '150px',
            borderRadius: '29px',
            position: 'relative',
            overflow: 'hidden',
            cursor: supported ? 'ns-resize' : 'not-allowed',
            background: 'rgba(255,255,255,0.16)',
            boxShadow: 'inset 0 1px 2px rgba(0,0,0,0.3)',
            opacity: supported ? 1 : 0.55,
            touchAction: 'none',
          }}
        >
          <div
            style={{
              position: 'absolute',
              left: 0,
              right: 0,
              bottom: 0,
              height: `${shown}%`,
              background: 'rgba(255,255,255,0.88)',
              transition: dragging ? 'none' : 'height 0.18s ease',
              pointerEvents: 'none',
            }}
          />
          <div
            style={{
              position: 'absolute',
              left: 0,
              right: 0,
              bottom: '14px',
              display: 'flex',
              justifyContent: 'center',
              pointerEvents: 'none',
            }}
          >
            <SunIcon color={iconOnFill ? '#1c1c1e' : 'rgba(255,255,255,0.7)'} />
          </div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
          <GlassButton
            label="Wi-Fi"
            active={wifi === true}
            available={wifi !== null}
            onToggle={() => toggleRadio('wifi')}
          >
            <WifiIcon />
          </GlassButton>
          <GlassButton
            label="Bluetooth"
            active={bluetooth === true}
            available={bluetooth !== null}
            onToggle={() => toggleRadio('bluetooth')}
          >
            <BluetoothIcon />
          </GlassButton>
        </div>
      </div>
    </FitBox>
  );
};
