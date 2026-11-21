export const isControlCenterWindow = (): boolean => {
  try {
    if (new URLSearchParams(window.location.search).get('window') === 'control-center') return true;
    const label = (window as any).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
    return label === 'control-center';
  } catch (_) {
    return false;
  }
};
