import React, { useEffect, useState } from 'react';
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
  const [shown, setShown] = useState(false);

  useEffect(() => {
    initWidget();
    initConfig();
    initLiquidGlass();
    hydrateUpdater();
    const removeUpdater = window.electronAPI?.onUpdaterStatus?.((status) => setUpdaterStatus(status));
    const removeTab = window.electronAPI?.onControlCenterTab?.((tab) => {
      useConfigStore.getState().setActiveAppTab(tab as DokiiiAppTab);
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
      removeTab?.();
      removeSynced?.();
      window.removeEventListener('keydown', onKey);
    };
  }, [initWidget, initConfig, initLiquidGlass, hydrateUpdater, setUpdaterStatus]);

  useEffect(() => {
    if (!initialized || shown) return;
    let cancelled = false;
    const reveal = (tab?: string) => {
      if (cancelled) return;
      if (tab) useConfigStore.getState().setActiveAppTab(tab as DokiiiAppTab);
      requestAnimationFrame(() => setShown(true));
    };
    try {
      window.electronAPI
        .controlCenterReady()
        .then((tab) => reveal(tab))
        .catch(() => reveal());
    } catch (_) {
      reveal();
    }
    return () => {
      cancelled = true;
    };
  }, [initialized, shown]);

  return shown ? <DokiiiApp /> : null;
};

export default ControlCenterApp;
