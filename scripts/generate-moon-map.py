#!/usr/bin/env python3
"""
Generates src/renderer/assets/moon-map.png, the surface map used by the Moon Phase widget.

Channels of the PNG:
  R = surface brightness of the near side of the Moon (maria, highlands, craters, bright rays)
  G = relief slope in x   (128 = flat)
  B = relief slope in y   (128 = flat)
The widget lights this map with the real Sun direction for the current moon phase, and the
relief channels make craters cast soft shadows near the terminator.

Maria and the main craters (Tycho, Copernicus, Kepler, Aristarchus, Plato, ...) are placed at
their real selenographic latitude and longitude. Smaller craters are random.

Usage: python3 scripts/generate-moon-map.py   (needs numpy and Pillow)
"""
from pathlib import Path

import numpy as np
from PIL import Image

N = 512
SEED = 20261127
rng = np.random.default_rng(SEED)

yy, xx = np.mgrid[0:N, 0:N].astype(np.float32)
X = (xx + 0.5 - N / 2) / (N / 2)
Y = (yy + 0.5 - N / 2) / (N / 2)
R2 = X * X + Y * Y
INSIDE = R2 <= 1.0
Z = np.sqrt(np.clip(1.0 - R2, 0.0, 1.0))


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def value_noise(cells, seed):
    grid = np.random.default_rng(seed).random((cells, cells)).astype(np.float32)
    img = Image.fromarray(grid).resize((N, N), Image.BICUBIC)
    return np.asarray(img, dtype=np.float32)


def fbm(base_cells, octaves, seed, gain=0.5):
    total = np.zeros((N, N), np.float32)
    amp, norm = 1.0, 0.0
    for o in range(octaves):
        total += amp * value_noise(base_cells * 2 ** o, seed + o * 17)
        norm += amp
        amp *= gain
    return total / norm


def zs(a):
    return (a - a.mean()) / (a.std() + 1e-6)


def sph(lat, lon):
    """Selenographic latitude/longitude (degrees) -> disc coordinates (x right, y down)."""
    la, lo = np.radians(lat), np.radians(lon)
    return float(np.cos(la) * np.sin(lo)), float(-np.sin(la))


# ---------------------------------------------------------------- maria
# name, lat, lon, half-width, half-height (disc units)
MARIA = [
    ("imbrium", 32.8, -15.6, 0.30, 0.25),
    ("serenitatis", 28.0, 17.5, 0.19, 0.19),
    ("tranquillitatis", 8.5, 31.4, 0.25, 0.20),
    ("crisium", 17.0, 59.1, 0.075, 0.13),
    ("fecunditatis", -7.8, 51.3, 0.075, 0.20),
    ("nectaris", -15.2, 34.6, 0.085, 0.10),
    ("nubium", -21.3, -16.6, 0.19, 0.16),
    ("humorum", -24.4, -38.6, 0.10, 0.10),
    ("procellarum", 12.0, -52.0, 0.26, 0.52),
    ("insularum", 7.5, -30.9, 0.13, 0.12),
    ("cognitum", -10.0, -23.0, 0.08, 0.07),
    ("vaporum", 13.3, 3.6, 0.09, 0.09),
    ("medii", 0.5, 0.0, 0.07, 0.06),
    ("frigoris", 56.0, 1.0, 0.38, 0.035),
    ("somniorum", 38.0, 29.0, 0.12, 0.06),
    ("roris", 54.0, -45.0, 0.10, 0.09),
]

warp = zs(fbm(6, 5, 91))
warp_fine = zs(fbm(24, 4, 93))
mare = np.zeros((N, N), np.float32)
for _, lat, lon, ax, ay in MARIA:
    cx, cy = sph(lat, lon)
    d = np.sqrt(((X - cx) / ax) ** 2 + ((Y - cy) / ay) ** 2)
    d = d * (1.0 + 0.34 * warp + 0.10 * warp_fine)
    mare = np.maximum(mare, 1.0 - smoothstep(0.55, 1.10, d))
mare *= INSIDE

# ---------------------------------------------------------------- albedo
hi = 0.78 + 0.05 * zs(fbm(3, 7, 101)) + 0.025 * zs(fbm(48, 2, 103))
ma = 0.45 + 0.04 * zs(fbm(5, 4, 201)) + 0.03 * zs(fbm(64, 3, 203))
albedo = hi * (1.0 - mare) + ma * mare

H = 0.003 * zs(fbm(30, 4, 401)) - 0.012 * mare
A = np.zeros((N, N), np.float32)


def add_crater(cx, cy, r, depth=1.0, fresh=0.0, dark_floor=0.0):
    cz = float(np.sqrt(max(0.0, 1.0 - cx * cx - cy * cy)))
    reach = 2.6 * r
    px, py = (cx + 1) * N / 2, (cy + 1) * N / 2
    w = reach * N / 2 + 2
    x0, x1 = int(max(0, px - w)), int(min(N, px + w + 1))
    y0, y1 = int(max(0, py - w)), int(min(N, py + w + 1))
    if x1 <= x0 or y1 <= y0:
        return
    sx, sy, sz = X[y0:y1, x0:x1], Y[y0:y1, x0:x1], Z[y0:y1, x0:x1]
    d = np.sqrt((sx - cx) ** 2 + (sy - cy) ** 2 + (sz - cz) ** 2) / r
    bowl = np.where(d < 1.0, -(1.0 - d ** 2), 0.0)
    rim = np.exp(-(((d - 1.0) / 0.16) ** 2))
    eject = np.where(d > 1.0, np.power(np.maximum(d, 1.0), -3.0), 0.0)
    H[y0:y1, x0:x1] += 0.16 * r * depth * (bowl + 0.55 * rim + 0.25 * eject)
    floor = 1.0 - smoothstep(0.5, 0.95, d)
    a = -0.04 * floor * (1.0 - fresh) + 0.04 * rim
    a += fresh * (0.30 * np.exp(-((d / 1.3) ** 2)) + 0.10 * np.exp(-((d / 2.6) ** 2)))
    a -= dark_floor * 0.22 * floor
    A[y0:y1, x0:x1] += a


def add_rays(cx, cy, count, length, strength):
    theta = np.arctan2(Y - cy, X - cx)
    dist = np.hypot(X - cx, Y - cy)
    out = np.zeros((N, N), np.float32)
    for _ in range(count):
        ang = rng.uniform(-np.pi, np.pi)
        width = rng.uniform(0.012, 0.045)
        delta = np.arctan2(np.sin(theta - ang), np.cos(theta - ang))
        out += rng.uniform(0.4, 1.0) * np.exp(-((delta / width) ** 2))
    A[:] += strength * out * np.exp(-dist / length) * smoothstep(0.02, 0.07, dist)


# random craters (power law)
placed = 0
while placed < 2400:
    r = 0.007 * (1.0 - rng.random()) ** (-1.0 / 1.6)
    if r > 0.09:
        continue
    z = max(0.08, rng.random() ** 0.8)
    phi = rng.uniform(0, 2 * np.pi)
    cx, cy = np.sqrt(1 - z * z) * np.cos(phi), np.sqrt(1 - z * z) * np.sin(phi)
    m = mare[int((cy + 1) * N / 2) % N, int((cx + 1) * N / 2) % N]
    if rng.random() > 1.0 - 0.75 * m:
        continue
    fresh = rng.uniform(0.4, 1.0) if (r > 0.011 and rng.random() < 0.06) else 0.0
    add_crater(cx, cy, r, depth=rng.uniform(0.6, 1.2), fresh=fresh)
    placed += 1

# named craters at real positions: lat, lon, radius, depth, fresh, dark floor
NAMED = [
    (-43.3, -11.4, 0.027, 1.0, 1.0, 0.0),   # Tycho
    (9.7, -20.1, 0.027, 1.0, 0.9, 0.0),     # Copernicus
    (8.1, -38.0, 0.012, 1.0, 1.0, 0.0),     # Kepler
    (23.7, -47.4, 0.012, 1.0, 1.0, 0.0),    # Aristarchus
    (51.6, -9.4, 0.055, 0.5, 0.0, 0.45),     # Plato
    (-58.8, -14.4, 0.12, 0.8, 0.0, 0.0),    # Clavius
    (-5.5, -68.5, 0.06, 0.8, 0.0, 0.5),     # Grimaldi
    (-8.9, 61.0, 0.05, 1.0, 0.1, 0.2),      # Langrenus
    (-25.0, 61.0, 0.05, 1.0, 0.0, 0.2),     # Petavius
    (-11.4, 26.4, 0.04, 1.0, 0.3, 0.0),     # Theophilus
    (-13.2, 24.0, 0.035, 1.0, 0.0, 0.0),    # Cyrillus
    (-9.3, -1.9, 0.07, 0.6, 0.0, 0.4),      # Ptolemaeus
    (-13.4, -3.2, 0.045, 0.9, 0.0, 0.3),    # Alphonsus
    (-18.2, -1.9, 0.035, 1.0, 0.0, 0.0),    # Arzachel
    (29.7, -4.0, 0.045, 0.8, 0.0, 0.35),     # Archimedes
    (14.5, -11.3, 0.026, 1.0, 0.2, 0.0),    # Eratosthenes
    (16.0, 47.0, 0.008, 1.0, 1.0, 0.0),     # Proclus
    (31.8, 30.0, 0.05, 0.8, 0.0, 0.3),      # Posidonius
    (50.2, 17.3, 0.05, 1.0, 0.0, 0.0),      # Aristoteles
    (53.6, 56.4, 0.05, 0.8, 0.0, 0.35),      # Endymion
]
for lat, lon, r, depth, fresh, dark in NAMED:
    cx, cy = sph(lat, lon)
    add_crater(cx, cy, r, depth=depth, fresh=fresh, dark_floor=dark)

add_rays(*sph(-43.3, -11.4), count=26, length=0.55, strength=0.14)   # Tycho
add_rays(*sph(9.7, -20.1), count=14, length=0.22, strength=0.10)     # Copernicus
add_rays(*sph(8.1, -38.0), count=8, length=0.15, strength=0.09)      # Kepler
add_rays(*sph(23.7, -47.4), count=8, length=0.12, strength=0.11)     # Aristarchus

albedo = albedo + A + 0.012 * zs(fbm(128, 2, 301))
albedo = np.clip(albedo, 0.08, 0.98)

# ---------------------------------------------------------------- pack
step = 2.0 / N
gy, gx = np.gradient(H, step)
SLOPE_SCALE = 0.8  # must match SLOPE_SCALE in MoonPhaseWidget.tsx
enc = lambda s: np.clip(128 + 127 * np.clip(s * SLOPE_SCALE, -1, 1), 0, 255)

rgba = np.zeros((N, N, 4), np.uint8)
rgba[..., 0] = np.where(INSIDE, albedo * 255, 0)
rgba[..., 1] = np.where(INSIDE, enc(gx), 128)
rgba[..., 2] = np.where(INSIDE, enc(gy), 128)
rgba[..., 3] = 255

out = Path(__file__).resolve().parent.parent / "src" / "renderer" / "assets" / "moon-map.png"
Image.fromarray(rgba, "RGBA").save(out, optimize=True)
print("wrote", out, out.stat().st_size // 1024, "KB")
