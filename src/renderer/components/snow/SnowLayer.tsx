import React, { useEffect, useRef } from 'react';
import { useConfigStore } from '../../store/configStore';
import { DEFAULT_SNOW_CONFIG } from '../../../shared/constants';
import { getSnowEngine } from './snowEngine';
import { resolveExtras } from '../../../shared/performance';
import './snow.css';

const SnowLayer: React.FC = () => {
  const snow = useConfigStore((s) => s.dock.snow);
  const initialized = useConfigStore((s) => s.initialized);
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const enabled = snow?.enabled ?? DEFAULT_SNOW_CONFIG.enabled;
  const intensity = snow?.intensity ?? DEFAULT_SNOW_CONFIG.intensity;
  const snowballs = snow?.snowballs ?? DEFAULT_SNOW_CONFIG.snowballs;
  const onWindows = snow?.onWindows ?? DEFAULT_SNOW_CONFIG.onWindows;
  const inFront = snow?.inFront ?? DEFAULT_SNOW_CONFIG.inFront;
  const fps = useConfigStore((s) => resolveExtras(s.dock).frameRate);

  useEffect(() => {
    const container = containerRef.current;
    const canvas = canvasRef.current;
    if (!container || !canvas) return;
    const engine = getSnowEngine();
    engine.attach(container, canvas);
    return () => engine.detach();
  }, []);

  useEffect(() => {
    if (!initialized) return;
    getSnowEngine().configure({ enabled, intensity, snowballs, onWindows, inFront, fps });
  }, [initialized, enabled, intensity, snowballs, onWindows, inFront, fps]);

  useEffect(() => {
    const off = window.electronAPI?.onSnowClear?.(() => getSnowEngine().clearSnow());
    return () => off?.();
  }, []);

  return (
    <div ref={containerRef} className="snow-layer" style={{ display: 'none' }}>
      <canvas ref={canvasRef} className="snow-canvas" />
    </div>
  );
};

export default SnowLayer;
