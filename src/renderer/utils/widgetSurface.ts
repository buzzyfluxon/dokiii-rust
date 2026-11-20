import { DESKTOP_WIDGET_SIZES, DesktopWidgetSize } from '../../shared/constants';

export const BASE_SIZE: Record<DesktopWidgetSize, { width: number; height: number; padding: number }> = {
  small: { ...DESKTOP_WIDGET_SIZES.small, padding: 14 },
  medium: { ...DESKTOP_WIDGET_SIZES.medium, padding: 16 },
  large: { ...DESKTOP_WIDGET_SIZES.large, padding: 18 },
};

export const FILL_TYPES = new Set<string>(['weather', 'big-date', 'digital-clock', 'photo-card', 'music-card', 'app-icon']);
export const BARE_TYPES = new Set<string>(['app-icon']);

export const MAX_TRANSPARENCY = 80;

const DARK_FILL_TYPES = new Set<string>(['weather', 'big-date', 'digital-clock', 'music-card']);

const parseRgba = (value: string): [number, number, number, number] | null => {
  const match = value.match(/rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)/);
  if (!match) return null;
  return [Number(match[1]), Number(match[2]), Number(match[3]), match[4] === undefined ? 1 : Number(match[4])];
};

const baseColor = (type: string): [number, number, number, number] => {
  if (type === 'photo-card') return [22, 22, 26, 1];
  if (DARK_FILL_TYPES.has(type)) return [14, 14, 17, 0.86];
  return [22, 22, 26, 0.72];
};

export const widgetBackground = (type: string, transparency: number | undefined, glassRgba?: string): string | undefined => {
  if (!transparency || transparency <= 0) return undefined;
  const clamped = Math.min(MAX_TRANSPARENCY, Math.max(0, transparency));
  const source = (glassRgba && parseRgba(glassRgba)) || baseColor(type);
  const alpha = Math.max(0, source[3] * (1 - clamped / 100));
  return `rgba(${Math.round(source[0])}, ${Math.round(source[1])}, ${Math.round(source[2])}, ${alpha.toFixed(3)})`;
};
