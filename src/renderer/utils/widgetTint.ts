import type { DesktopWidgetItem, WidgetTint } from '../../shared/constants';

export const clamp = (v: number, min: number, max: number) => Math.min(max, Math.max(min, v));

export const normalizeHex = (input: string): string | null => {
  let v = input.trim().replace(/^#/, '');
  if (/^[0-9a-fA-F]{3}$/.test(v)) v = v.split('').map((c) => c + c).join('');
  return /^[0-9a-fA-F]{6}$/.test(v) ? `#${v.toLowerCase()}` : null;
};

export const hexToRgb = (hex: string): [number, number, number] => {
  const n = normalizeHex(hex) ?? '#000000';
  return [parseInt(n.slice(1, 3), 16), parseInt(n.slice(3, 5), 16), parseInt(n.slice(5, 7), 16)];
};

export const rgbToHex = (r: number, g: number, b: number): string =>
  '#' + [r, g, b].map((v) => clamp(Math.round(v), 0, 255).toString(16).padStart(2, '0')).join('');

export interface Hsv {
  h: number;
  s: number;
  v: number;
}

export const rgbToHsv = (r: number, g: number, b: number): Hsv => {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const d = max - min;
  let h = 0;
  if (d !== 0) {
    if (max === rn) h = ((gn - bn) / d) % 6;
    else if (max === gn) h = (bn - rn) / d + 2;
    else h = (rn - gn) / d + 4;
    h *= 60;
    if (h < 0) h += 360;
  }
  return { h, s: max === 0 ? 0 : d / max, v: max };
};

export const hsvToRgb = ({ h, s, v }: Hsv): [number, number, number] => {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;
  let r = 0;
  let g = 0;
  let b = 0;
  if (h < 60) [r, g, b] = [c, x, 0];
  else if (h < 120) [r, g, b] = [x, c, 0];
  else if (h < 180) [r, g, b] = [0, c, x];
  else if (h < 240) [r, g, b] = [0, x, c];
  else if (h < 300) [r, g, b] = [x, 0, c];
  else [r, g, b] = [c, 0, x];
  return [(r + m) * 255, (g + m) * 255, (b + m) * 255];
};

export const hexToHsv = (hex: string): Hsv => {
  const [r, g, b] = hexToRgb(hex);
  return rgbToHsv(r, g, b);
};

export const hsvToHex = (hsv: Hsv): string => {
  const [r, g, b] = hsvToRgb(hsv);
  return rgbToHex(r, g, b);
};

// Strength 0-100 maps to an overlay opacity. The overlay sits on top of the glass surface,
// so the blur, border and highlights of liquid glass stay intact underneath it.
export const tintCss = (t?: WidgetTint | null): string | undefined => {
  if (!t || t.strength <= 0) return undefined;
  const a = clamp(t.strength / 100, 0, 1) * 0.92;
  const [r, g, b] = hexToRgb(t.hex);
  const c1 = `rgba(${r}, ${g}, ${b}, ${a.toFixed(3)})`;
  if (!t.gradient) return `linear-gradient(${c1}, ${c1})`;
  const [r2, g2, b2] = hexToRgb(t.hex2 || t.hex);
  const c2 = `rgba(${r2}, ${g2}, ${b2}, ${a.toFixed(3)})`;
  return `linear-gradient(${t.angle ?? 135}deg, ${c1}, ${c2})`;
};

export const effectiveTint = (item: DesktopWidgetItem, global?: WidgetTint | null): WidgetTint | null =>
  item.tint ? item.tint : global ?? null;
