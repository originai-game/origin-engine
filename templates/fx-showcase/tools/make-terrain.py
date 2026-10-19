#!/usr/bin/env python3
"""Regenerates the terrain station's self-made assets (CC0), byte for byte:

    python3 templates/fx-showcase/tools/make-terrain.py templates/fx-showcase/assets

- terrain/valley.png: a 257 x 257 16-bit grey heightmap of a 256 m valley
  (one sample a metre; black -10 m, white 50 m, as terrain/valley.ogt says):
  rolling ground, a mountain to the north, a lake basin and a knoll the
  station's character walks over.
- terrain/valley-control.png: its 256 x 256 RGBA control map: grass in R
  (except on the path), the path in G, rock in B and sand in A (both
  everywhere: their slope and altitude rules place them).
- textures/terrain-grass.png, terrain-path.png, terrain-rock.png,
  terrain-sand.png: 128 x 128 tiling layer textures, their alpha the
  surface's height (terrain/valley.ogt's heightBlend).

Pillow only; no randomness beyond a fixed hash, so the output is reproducible.
"""
import math, os, sys
from PIL import Image

out = sys.argv[1] if len(sys.argv) > 1 else "templates/fx-showcase/assets"
for d in ("terrain", "textures"):
    os.makedirs(os.path.join(out, d), exist_ok=True)

LOW, HIGH = -10.0, 50.0
N = 257  # samples per side, one a metre
HALF = (N - 1) / 2


def hash01(x, y, s=0):
    h = (x * 374761393 + y * 668265263 + s * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h ^ (h >> 16)) / 0xFFFFFFFF


def smooth(t):
    return t * t * (3 - 2 * t)


def value(x, y, s, period=None):
    """Value noise in 0..1 on a unit grid (tiling every `period` cells)."""
    xi, yi = math.floor(x), math.floor(y)
    fx, fy = smooth(x - xi), smooth(y - yi)

    def at(i, j):
        if period:
            i, j = i % period, j % period
        return hash01(i, j, s)

    a, b = at(xi, yi), at(xi + 1, yi)
    c, d = at(xi, yi + 1), at(xi + 1, yi + 1)
    return a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy


def fbm(x, y, s, octaves=4, period=None):
    total, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        p = period * (1 << o) if period else None
        total += amp * value(x * (1 << o), y * (1 << o), s + o * 17, p)
        norm += amp
        amp *= 0.5
    return total / norm


def gauss(x, z, cx, cz, r):
    return math.exp(-((x - cx) ** 2 + (z - cz) ** 2) / (2 * r * r))


def height(x, z):
    """Metres at local (x, z), the terrain's centre at 0."""
    h = 5.5 + 6.0 * (fbm(x / 48 + 3.1, z / 48 + 7.3, 1) - 0.5)
    # The mountain to the north (-z), its flanks broken by ridges.
    ridge = 1 - abs(2 * fbm(x / 22, z / 22, 5) - 1)
    h += 32 * gauss(x, z, -25, -88, 34) * (0.75 + 0.45 * ridge)
    h += 16 * gauss(x, z, 70, -60, 24) * (0.8 + 0.4 * ridge)
    # The lake basin, and the knoll west of it.
    h -= 11 * gauss(x, z, -2, -14, 24)
    h += 7 * gauss(x, z, -14, 34, 12)
    return h


def path(x, z):
    """How much of the path is at (x, z): a band along z = 36 west of the knoll's top."""
    d = abs(z - 36 - 2.5 * math.sin(x / 9))
    along = 1.0 if -60 <= x <= -6 else max(0.0, 1 - min(abs(x + 60), abs(x + 6)) / 4)
    return max(0.0, 1 - d / 1.6) * along


heights = Image.new("I;16", (N, N))
px = heights.load()
for r in range(N):
    for c in range(N):
        h = height(c - HALF, r - HALF)
        px[c, r] = max(0, min(65535, round((h - LOW) / (HIGH - LOW) * 65535)))
heights.save(os.path.join(out, "terrain", "valley.png"), optimize=False)

M = 256
control = Image.new("RGBA", (M, M))
cp = control.load()
for r in range(M):
    for c in range(M):
        x = (c + 0.5) / M * (N - 1) - HALF
        z = (r + 0.5) / M * (N - 1) - HALF
        p = path(x, z)
        cp[c, r] = (round(255 * (1 - p)), round(255 * p), 255, 255)
control.save(os.path.join(out, "terrain", "valley-control.png"), optimize=False)

S = 128


def layer(name, base, vary, seed, cells, streak=0.0):
    img = Image.new("RGBA", (S, S))
    p = img.load()
    for y in range(S):
        for x in range(S):
            n = fbm(x / S * cells, y / S * cells, seed, 4, cells)
            if streak:
                n = (1 - streak) * n + streak * fbm(x / S * cells * 4, y / S * 2, seed + 9, 3, 2)
            fine = hash01(x, y, seed + 3)
            k = 0.75 * n + 0.25 * fine
            rgb = tuple(max(0, min(255, round(base[i] + vary[i] * (k - 0.5) * 2))) for i in range(3))
            p[x, y] = rgb + (max(0, min(255, round(255 * n))),)
    img.save(os.path.join(out, "textures", name), optimize=False)


layer("terrain-grass.png", (88, 118, 52), (30, 34, 18), 21, 8)
layer("terrain-path.png", (122, 98, 70), (26, 22, 18), 31, 6)
layer("terrain-rock.png", (118, 114, 108), (40, 38, 36), 41, 4, streak=0.4)
layer("terrain-sand.png", (186, 168, 128), (20, 18, 14), 51, 10)
