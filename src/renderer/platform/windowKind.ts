export const isControlCenterWindow = (): boolean => {
  try {
    if ((window as any).__DOKIII_WINDOW__ === 'control-center') return true;
    if (new URLSearchParams(window.location.search).get('window') === 'control-center') return true;
    const label = (window as any).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
    return label === 'control-center';
  } catch (_) {
    return false;
  }
};
