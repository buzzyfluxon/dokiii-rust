import React, { useEffect, useRef, useState } from 'react';
import { DesktopWidgetSize } from '../../../shared/constants';
import { computeMoonState, MoonState } from '../../utils/moonPhase';
import moonMapUrl from '../../assets/moon-map.png';

interface MoonPhaseProps {
  size: DesktopWidgetSize;
}

// Must match SLOPE_SCALE in scripts/generate-moon-map.py
const SLOPE_SCALE = 0.8;
// How strongly crater relief shows up in the shading near the terminator
const RELIEF = 1.6;

const smooth = (a: number, b: number, x: number) => {
  const t = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

// ---------------------------------------------------------------------------------------------
// Shared texture cache
//
// The 512px moon map is decoded once and resampled once per on-screen size. Everything is reference
// counted: when the last Moon widget is removed from the desktop the decoded image and all pixel
// buffers are dropped, so a deleted widget no longer keeps memory around.
// ---------------------------------------------------------------------------------------------
let mapPromise: Promise<HTMLImageElement> | null = null;
const resampled = new Map<number, Uint8ClampedArray>();
let users = 0;

const loadMap = (): Promise<HTMLImageElement> => {
  if (!mapPromise) {
    mapPromise = new Promise((resolve, reject) => {
      const img = new Image();
      img.decoding = 'async';
      img.onload = () => resolve(img);
      img.onerror = () => {
        mapPromise = null;
        reject(new Error('moon map failed to load'));
      };
      img.src = moonMapUrl;
    });
  }
  return mapPromise;
};

const getTexture = async (px: number): Promise<Uint8ClampedArray | null> => {
  const cached = resampled.get(px);
  if (cached) return cached;
  const img = await loadMap();
  const again = resampled.get(px);
  if (again) return again;
  const work = document.createElement('canvas');
  work.width = px;
  work.height = px;
  const wctx = work.getContext('2d', { willReadFrequently: true });
  if (!wctx) return null;
  wctx.imageSmoothingEnabled = true;
  wctx.imageSmoothingQuality = 'high';
  wctx.drawImage(img, 0, 0, px, px);
  const data = wctx.getImageData(0, 0, px, px).data;
  work.width = 0;
  work.height = 0;
  resampled.set(px, data);
  return data;
};

const retainTexture = () => {
  users += 1;
};

const releaseTexture = () => {
  users = Math.max(0, users - 1);
  if (users > 0) return;
  resampled.clear();
  mapPromise = null;
  try {
    window.electronAPI?.trimMemory?.();
  } catch (_) {}
};

/** Lights the moon map with the real Sun direction for this phase and paints it onto the canvas. */
const paintMoon = (canvas: HTMLCanvasElement, tex: Uint8ClampedArray, moon: MoonState, px: number) => {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const out = ctx.createImageData(px, px);
  const dst = out.data;

  // Direction of the Sun as seen from the Moon: lit from the right while waxing, from the left while waning
  const e = (moon.elongation * Math.PI) / 180;
  const phaseAngle = Math.abs(Math.PI - e);
  const sx = (moon.waxing ? 1 : -1) * Math.sin(phaseAngle);
  const sz = Math.cos(phaseAngle);
  const k = moon.illumination / 100;
  const ambient = 0.02 + 0.05 * (1 - k) * (1 - k); // faint earthshine on the dark side

  const half = px / 2;
  for (let j = 0; j < px; j++) {
    for (let i = 0; i < px; i++) {
      const nx = (i + 0.5 - half) / half;
      const ny = (j + 0.5 - half) / half;
      const r2 = nx * nx + ny * ny;
      if (r2 >= 1) continue;

      const idx = (j * px + i) * 4;
      const nz = Math.sqrt(1 - r2);
      const edge = Math.min(1, (1 - Math.sqrt(r2)) * half);

      const albedo = tex[idx] / 255;
      const slopeX = (tex[idx + 1] - 128) / 127 / SLOPE_SCALE;
      const slopeY = (tex[idx + 2] - 128) / 127 / SLOPE_SCALE;

      // surface normal tilted by crater and mountain relief
      let qx = nx - RELIEF * slopeX * nz;
      let qy = ny - RELIEF * slopeY * nz;
      let qz = nz;
      const len = Math.sqrt(qx * qx + qy * qy + qz * qz) || 1;
      qx /= len;
      qy /= len;
      qz /= len;

      const mu0 = Math.max(0, qx * sx + qz * sz);
      const mu0Smooth = nx * sx + nz * sz;
      const mu = Math.max(nz, 0.06);
      // Lommel-Seeliger: how the dusty lunar surface scatters light (no limb darkening at full moon)
      const lit = Math.min(1.15, (2 * mu0) / (mu0 + mu)) * smooth(-0.02, 0.1, mu0Smooth);
      const v = Math.min(1, albedo * Math.max(lit, ambient) * 1.08);

      dst[idx] = v * 255;
      dst[idx + 1] = v * 251;
      dst[idx + 2] = v * 242;
      dst[idx + 3] = edge * 255;
    }
  }
  ctx.putImageData(out, 0, 0);
};

const MoonCanvas: React.FC<{ radius: number; moon: MoonState }> = ({ radius, moon }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const diameter = radius * 2;
  // One texel per device pixel is enough. The old 1.5x oversampling only made the buffers bigger.
  const px = Math.round(diameter * Math.min(2, Math.max(1, window.devicePixelRatio || 1)));

  useEffect(() => {
    retainTexture();
    return releaseTexture;
  }, []);

  useEffect(() => {
    let cancelled = false;
    getTexture(px)
      .then((tex) => {
        if (!cancelled && tex && canvasRef.current) paintMoon(canvasRef.current, tex, moon, px);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [moon.elongation, moon.illumination, px]);

  return (
    <canvas
      ref={canvasRef}
      width={px}
      height={px}
      style={{
        width: diameter,
        height: diameter,
        display: 'block',
        borderRadius: '50%',
        boxShadow: '0 4px 14px rgba(0,0,0,0.5)',
        pointerEvents: 'none',
      }}
    />
  );
};

export const MoonPhaseWidget: React.FC<MoonPhaseProps> = ({ size = 'small' }) => {
  const [moon, setMoon] = useState<MoonState>(() => computeMoonState());

  useEffect(() => {
    // The Moon moves about half a degree an hour, so checking every 5 minutes keeps it live and
    // the picture is only repainted once the phase has actually moved.
    const update = () => {
      const next = computeMoonState();
      setMoon((prev) => (Math.abs(next.elongation - prev.elongation) >= 0.05 ? next : prev));
    };
    const timer = setInterval(update, 5 * 60 * 1000);
    return () => clearInterval(timer);
  }, []);

  const illumination = Math.round(moon.illumination);
  const ageText = `${moon.waxing ? 'Waxing' : 'Waning'} \u2022 ${moon.ageDays.toFixed(1)} days old`;

  if (size === 'medium') {
    return (
      <div style={{ display: 'flex', height: '100%', alignItems: 'center', justifyContent: 'space-between', gap: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <MoonCanvas radius={46} moon={moon} />
        </div>
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: '4px' }}>
          <span style={{ fontSize: '11px', color: 'rgba(255,255,255,0.6)', textTransform: 'uppercase', letterSpacing: '0.4px' }}>
            Lunar Phase
          </span>
          <span style={{ fontSize: '18px', fontWeight: 600 }}>{moon.phaseName}</span>
          <span style={{ fontSize: '12px', color: 'rgba(255,255,255,0.5)' }}>Illumination: {illumination}%</span>
        </div>
      </div>
    );
  }

  if (size === 'large') {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', height: '100%', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', width: '100%', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: '12px', color: 'rgba(255,255,255,0.6)', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
            Moon Phase
          </span>
          <span style={{ fontSize: '12px', color: '#0a84ff', fontWeight: 500 }}>{illumination}% lit</span>
        </div>
        <div style={{ margin: 'auto 0' }}>
          <MoonCanvas radius={85} moon={moon} />
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '2px' }}>
          <span style={{ fontSize: '16px', fontWeight: 600 }}>{moon.phaseName}</span>
          <span style={{ fontSize: '12px', color: 'rgba(255,255,255,0.5)' }}>{ageText}</span>
        </div>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', height: '100%', alignItems: 'center', justifyContent: 'center' }}>
      <MoonCanvas radius={48} moon={moon} />
    </div>
  );
};
