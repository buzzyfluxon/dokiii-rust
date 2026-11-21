import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useWidgetStore } from './store/widgetStore';
import { useConfigStore, DokiiiAppTab } from './store/configStore';
import { useLiquidGlassStore } from './store/liquidGlassStore';
import { useUpdaterStore } from './store/updaterStore';
import { DokiiiApp } from './components/DokiiiApp';
import './styles/global.css';
import './styles/dock.css';
import './styles/widgets.css';
import './styles/widget-library.css';

const ControlCenterApp: React.FC = () => {
  const initWidget = useWidgetStore((s) => s.init);
  const initConfig = useConfigStore((s) => s.init);
  const initLiquidGlass = useLiquidGlassStore((s) => s.init);
  const initialized = useConfigStore((s) => s.initialized);
  const setUpdaterStatus = useUpdaterStore((s) => s.setUpdaterStatus);
  const hydrateUpdater = useUpdaterStore((s) => s.hydrate);
  const rootRef = useRef<HTMLDivElement>(null);
  const [mounted, setMounted] = useState(false);
  const mountedRef = useRef(false);
  const pendingOpen = useRef(false);

  const playOpen = useCallback(async () => {
    if (!mountedRef.current) {
      pendingOpen.current = true;
      return;
    }
    try {
      await window.electronAPI.showControlCenter();
    } catch (_) {}
    const el = rootRef.current;
    if (!el) return;
    el.classList.remove('cc-open', 'cc-closed');
    void el.offsetWidth;
    el.classList.add('cc-open');
  }, []);

  const markClosed = useCallback(() => {
    const el = rootRef.current;
    if (!el) return;
    el.classList.remove('cc-open');
    el.classList.add('cc-closed');
  }, []);

  useEffect(() => {
    initWidget();
    initConfig();
    initLiquidGlass();
    hydrateUpdater();
    const removeUpdater = window.electronAPI?.onUpdaterStatus?.((status) => setUpdaterStatus(status));
    const removeOpen = window.electronAPI?.onControlCenterOpen?.((tab) => {
      if (tab) useConfigStore.getState().setActiveAppTab(tab as DokiiiAppTab);
      void playOpen();
    });
    const removeSynced = window.electronAPI?.onConfigSynced?.(() => {
      void useConfigStore.getState().reload();
      void useWidgetStore.getState().init();
    });
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') useConfigStore.getState().closeDokiiiApp();
    };
    window.addEventListener('keydown', onKey);
    return () => {
      removeUpdater?.();
      removeOpen?.();
      removeSynced?.();
      window.removeEventListener('keydown', onKey);
    };
  }, [initWidget, initConfig, initLiquidGlass, hydrateUpdater, setUpdaterStatus, playOpen]);

  // Build the whole UI while the window is still hidden, so opening it later is instant.
  useEffect(() => {
    if (!initialized || mountedRef.current) return;
    let cancelled = false;
    const start = (info?: { tab: string; open: boolean }) => {
      if (cancelled) return;
      if (info?.tab) useConfigStore.getState().setActiveAppTab(info.tab as DokiiiAppTab);
      if (info?.open) pendingOpen.current = true;
      mountedRef.current = true;
      setMounted(true);
    };
    try {
      window.electronAPI
        .controlCenterReady()
        .then((info) => start(info))
        .catch(() => start({ tab: 'home', open: true }));
    } catch (_) {
      start({ tab: 'home', open: true });
    }
    return () => {
      cancelled = true;
    };
  }, [initialized]);

  useEffect(() => {
    if (mounted && pendingOpen.current) {
      pendingOpen.current = false;
      void playOpen();
    }
  }, [mounted, playOpen]);

  return (
    <div ref={rootRef} className="cc-root cc-closed">
      {mounted ? <DokiiiApp onClosed={markClosed} /> : null}
    </div>
  );
};

export default ControlCenterApp;
