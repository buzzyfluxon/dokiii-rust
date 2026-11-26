export interface MoonState {
  /** Elongation of the Moon from the Sun in degrees: 0 = new, 90 = first quarter, 180 = full, 270 = last quarter. */
  elongation: number;
  /** Fraction of the cycle, 0 to 1 (0 = new, 0.5 = full). */
  phaseFraction: number;
  /** Percent of the visible disc that is lit, 0 to 100. */
  illumination: number;
  waxing: boolean;
  /** Days since the last new moon. */
  ageDays: number;
  phaseName: string;
}

export const SYNODIC_MONTH = 29.530588861;

const rad = (deg: number) => (deg * Math.PI) / 180;
const norm360 = (deg: number) => ((deg % 360) + 360) % 360;

/**
 * Real moon phase for a given moment. Uses the mean elongation of the Moon plus the main periodic
 * terms from Meeus, "Astronomical Algorithms" (ch. 48). The result is within a few minutes of a
 * full ephemeris, so the lit part matches the real Moon in the sky.
 */
export const computeMoonState = (date: Date = new Date()): MoonState => {
  const jd = date.getTime() / 86400000 + 2440587.5;
  const t = (jd - 2451545.0) / 36525;

  const d = 297.8501921 + 445267.1114034 * t - 0.0018819 * t * t + (t * t * t) / 545868;
  const m = 357.5291092 + 35999.0502909 * t - 0.0001536 * t * t;
  const mp = 134.9633964 + 477198.8675055 * t + 0.0087414 * t * t + (t * t * t) / 69699;

  const phaseAngle =
    180 -
    d -
    6.289 * Math.sin(rad(mp)) +
    2.1 * Math.sin(rad(m)) -
    1.274 * Math.sin(rad(2 * d - mp)) -
    0.658 * Math.sin(rad(2 * d)) -
    0.214 * Math.sin(rad(2 * mp)) -
    0.11 * Math.sin(rad(d));

  const elongation = norm360(180 - phaseAngle);
  const phaseFraction = elongation / 360;
  const illumination = ((1 - Math.cos(rad(elongation))) / 2) * 100;

  let phaseName: string;
  if (phaseFraction < 0.03 || phaseFraction >= 0.97) phaseName = 'New Moon';
  else if (phaseFraction < 0.22) phaseName = 'Waxing Crescent';
  else if (phaseFraction < 0.28) phaseName = 'First Quarter';
  else if (phaseFraction < 0.47) phaseName = 'Waxing Gibbous';
  else if (phaseFraction < 0.53) phaseName = 'Full Moon';
  else if (phaseFraction < 0.72) phaseName = 'Waning Gibbous';
  else if (phaseFraction < 0.78) phaseName = 'Last Quarter';
  else phaseName = 'Waning Crescent';

  return {
    elongation,
    phaseFraction,
    illumination,
    waxing: elongation < 180,
    ageDays: phaseFraction * SYNODIC_MONTH,
    phaseName,
  };
};
