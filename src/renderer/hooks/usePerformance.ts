import { useEffect } from 'react';
import { useConfigStore } from '../store/configStore';
import { refreshMultiplier, resolveExtras } from '../../shared/performance';

export function useRefreshMultiplier(): number {
  const rate = useConfigStore((s) => s.dock.performance?.refreshRate);
  return refreshMultiplier(rate);
}

export function getRefreshMultiplier(): number {
  return refreshMultiplier(useConfigStore.getState().dock.performance?.refreshRate);
}

export function useApplyPerformance() {
  const glassBlur = useConfigStore((s) => resolveExtras(s.dock).glassBlur);

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle('perf-no-blur', !glassBlur);
    return () => root.classList.remove('perf-no-blur');
  }, [glassBlur]);
}
