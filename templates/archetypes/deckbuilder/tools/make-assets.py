#!/usr/bin/env python3
"""Regenerates Ashen Crown's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/sprites/units.png + units.ogt: the flipbook sheet, 48 px cells:
  the two heroes (the Warden, the Hunter) and the five foes (cinder imp,
  ash hound, bog slime, the ember knight, the Ashen King), each standing
  side-on in two idle frames, then each as a round table token seen from
  above (the top-down-2d view),
- assets/sprites/portraits/*.png: each unit's head, for the unit frames,
- assets/sprites/battlefield.png: the side-2d backdrop, an ash plain at
  dusk under a sinking ember sun (640 x 704 px, 16 px a world unit),
- assets/sprites/table.png: the top-down-2d backdrop, a felt war table on
  a plank floor, seen from above, at the same scale,
- assets/cards/*.png: each card's art (40 x 32 px: art kind card, 5:4),
- assets/audio/*.wav: the card, the hits, block, hurt, poison, buffs,
  the draw, the turns, a fall, the win and the loss, and a battle loop,
  with the og sfx presets of the UI's sounds.

Standard library only; deterministic.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import png_bytes, write_wav, tone, noise, mix, concat  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")
C = 48  # a sheet cell


def write(path, data):
    path = os.path.join(assets, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(data, bytes) else "w") as f:
        f.write(data)


def rgb(h, a=255):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


def lerp(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(4))


CLEAR = (0, 0, 0, 0)
INK = "#120c0a"


def hashn(*v):
    h = 2166136261
    for x in v:
        h = ((h ^ (x & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    h ^= h >> 13
    h = (h * 0x5BD1E995) & 0xFFFFFFFF
    return (h ^ (h >> 15)) / 4294967296.0


class Pix:
    """Hard-edged pixels (no antialiasing): pixel art, sampled nearest."""

    def __init__(self, w, h, fill=CLEAR):
        self.w, self.h = w, h
        self.p = [fill] * (w * h)

    def get(self, x, y):
        return self.p[y * self.w + x] if 0 <= x < self.w and 0 <= y < self.h else CLEAR

    def set(self, x, y, c):
        if 0 <= x < self.w and 0 <= y < self.h:
            self.p[y * self.w + x] = rgb(c) if isinstance(c, str) else c

    def rect(self, x, y, w, h, c):
        for yy in range(y, y + h):
            for xx in range(x, x + w):
                self.set(xx, yy, c)

    def disc(self, cx, cy, r, c, ry=None):
        ry = ry if ry is not None else r
        for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
            for x in range(int(cx - r) - 1, int(cx + r) + 2):
                if ((x + 0.5 - cx) / r) ** 2 + ((y + 0.5 - cy) / ry) ** 2 <= 1:
                    self.set(x, y, c)

    def line(self, x0, y0, x1, y1, c):
        n = max(abs(x1 - x0), abs(y1 - y0), 1)
        for i in range(n + 1):
            self.set(round(x0 + (x1 - x0) * i / n), round(y0 + (y1 - y0) * i / n), c)

    def blit(self, other, ox, oy, flip=False):
        for y in range(other.h):
            for x in range(other.w):
                c = other.p[y * other.w + (other.w - 1 - x if flip else x)]
                if c[3]:
                    self.set(ox + x, oy + y, c)

    def outline(self, colour=INK):
        src = list(self.p)
        for y in range(self.h):
            for x in range(self.w):
                if src[y * self.w + x][3]:
                    continue
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    xx, yy = x + dx, y + dy
                    if 0 <= xx < self.w and 0 <= yy < self.h and src[yy * self.w + xx][3] == 255:
                        self.set(x, y, colour)
                        break

    def png(self):
        rows = []
        for y in range(self.h):
            row = bytearray()
            for x in range(self.w):
                row += bytes(self.p[y * self.w + x])
            rows.append(bytes(row))
        return png_bytes(self.w, self.h, rows)


# ── the units (drawn facing right; the foes are mirrored to face the heroes) ──

def warden(k):
    """The Warden: plate, an ember tabard, a round shield and a sword."""
    t = Pix(C, C)
    b = k  # the breath: one pixel down on the second frame
    t.rect(17, 40, 5, 6, "#3d3540"); t.rect(25, 40, 5, 6, "#3d3540")        # legs
    t.rect(16, 45, 6, 2, "#2a232c"); t.rect(25, 45, 7, 2, "#2a232c")        # boots
    t.rect(15, 26 + b, 17, 15 - b, "#8d96a6")                                # cuirass
    t.rect(16, 27 + b, 4, 12 - b, "#b9c1cf")
    t.rect(19, 29 + b, 9, 13 - b, "#c2502e")                                 # tabard
    t.rect(22, 29 + b, 3, 13 - b, "#e2793f")
    t.rect(19, 36 + b, 9, 1, "#7d2c1c")
    t.rect(17, 14 + b, 13, 12, "#8d96a6")                                    # helm
    t.rect(18, 15 + b, 4, 9, "#b9c1cf")
    t.rect(21, 19 + b, 9, 2, INK)                                            # visor
    t.rect(26, 19 + b, 2, 2, "#ffb347")
    t.rect(22, 10 + b, 3, 5, "#e2793f"); t.rect(23, 8 + b, 2, 3, "#ffb347")  # plume
    t.disc(13, 33 + b, 7, "#6b4428"); t.disc(13, 33 + b, 5, "#8a5a32")      # shield
    t.disc(13, 33 + b, 2, "#ffc857")
    for i in range(13):                                                      # sword
        t.set(33 + i // 2, 30 + b - i, "#e6ebf2"); t.set(34 + i // 2, 30 + b - i, "#a7b0bd")
    t.rect(31, 30 + b, 5, 2, "#c9a24a"); t.rect(32, 32 + b, 2, 3, "#6b4428")
    t.outline()
    return t


def hunter(k):
    """The Hunter: a teal hooded cloak, a bow on the back, a venom knife."""
    t = Pix(C, C)
    b = k
    t.rect(18, 40, 4, 6, "#3a3226"); t.rect(26, 40, 4, 6, "#3a3226")
    t.rect(17, 45, 6, 2, "#241e17"); t.rect(26, 45, 6, 2, "#241e17")
    t.rect(16, 25 + b, 16, 16 - b, "#2f6f6a")                                # cloak
    t.rect(17, 26 + b, 4, 13 - b, "#4f9a8a")
    t.rect(20, 30 + b, 8, 9 - b, "#5b4a33")                                  # jerkin
    t.rect(20, 34 + b, 8, 1, "#c9a24a")
    for i in range(18):                                                      # the bow on the back
        t.set(13 + round(3 * math.sin(i / 17 * math.pi)), 18 + i + b, "#8a5a32")
    t.line(13, 18 + b, 13, 35 + b, "#d8d2c0")
    t.rect(17, 13 + b, 14, 13, "#2f6f6a")                                    # hood
    t.rect(18, 14 + b, 4, 10, "#4f9a8a")
    t.rect(22, 17 + b, 8, 7, "#e0b48a")                                      # face
    t.rect(22, 17 + b, 8, 2, "#24504d")
    t.set(27, 20 + b, INK); t.set(28, 20 + b, INK)
    t.rect(31, 32 + b, 2, 3, "#5b4a33")                                      # knife
    t.rect(33, 31 + b, 6, 2, "#cfd6de"); t.rect(37, 31 + b, 2, 1, "#7ed957")
    t.outline()
    return t


def imp(k):
    t = Pix(C, C)
    b = k * 2
    t.disc(24, 46, 7, (0, 0, 0, 70), 1.5)
    t.rect(20, 40 - b, 3, 4, "#7d2c1c"); t.rect(26, 40 - b, 3, 4, "#7d2c1c")
    t.disc(24, 34 - b, 8, "#d9532c", 7)                                      # body
    t.disc(22, 31 - b, 3, "#ff8a3d")
    t.rect(16, 24 - b, 2, 5, "#ffc857"); t.rect(31, 24 - b, 2, 5, "#ffc857")  # horns
    t.rect(20, 31 - b, 3, 3, "#ffe28a"); t.rect(27, 31 - b, 3, 3, "#ffe28a")  # eyes
    t.set(21, 32 - b, INK); t.set(28, 32 - b, INK)
    t.rect(22, 37 - b, 6, 1, INK)
    for i in range(6):                                                       # wings
        t.set(13 - i // 2, 30 - b + i, "#8f2f1f"); t.set(14 - i // 2, 30 - b + i, "#8f2f1f")
        t.set(35 + i // 2, 30 - b + i, "#8f2f1f"); t.set(34 + i // 2, 30 - b + i, "#8f2f1f")
    t.rect(30 + k, 41 - b, 6, 1, "#7d2c1c"); t.set(36 + k, 40 - b, "#ffb347")  # tail
    t.outline()
    return t


def hound(k):
    t = Pix(C, C)
    b = k
    t.rect(10, 38, 3, 8, "#4a4048"); t.rect(16, 38, 3, 8, "#4a4048")
    t.rect(28, 38, 3, 8, "#4a4048"); t.rect(33, 38, 3, 8, "#4a4048")
    t.rect(9, 29 + b, 28, 11 - b, "#6f646c")                                 # body
    t.rect(10, 29 + b, 26, 3, "#8e8389")
    for x in range(10, 36, 4):                                               # bristles
        t.set(x, 28 + b, "#8e8389"); t.set(x + 1, 27 + b, "#ff8a3d")
    t.rect(32, 24 + b, 10, 10, "#6f646c")                                    # head
    t.rect(40, 29 + b, 5, 4, "#5a5058")
    t.rect(38, 27 + b, 2, 2, "#ffc857")                                      # eye
    t.rect(41, 33 + b, 4, 1, "#e6e0cf")
    t.rect(33, 21 + b, 3, 4, "#5a5058")                                      # ear
    t.rect(4, 30 + b - k, 6, 2, "#6f646c")                                   # tail
    t.outline()
    return t


def slime(k):
    t = Pix(C, C)
    sq = k
    for y in range(28 + sq, 46):
        w = max(6, min(18, 10 + round(8 * math.sqrt(max(0, (y - 27 - sq) / (19 - sq))))))
        t.rect(24 - w, y, 2 * w, 1, "#4f8a3a")
    for y in range(30 + sq, 40):
        t.rect(14, y, 4, 1, "#7ed957")
    t.disc(19, 35 + sq, 3, "#e6f2d0"); t.disc(29, 35 + sq, 3, "#e6f2d0")
    t.set(20, 35 + sq, INK); t.set(30, 35 + sq, INK)
    t.rect(21, 41, 6, 1, "#264d1d")
    t.disc(36, 30 + sq, 2, "#7ed957")                                        # a bubble
    t.outline()
    return t


def knight(k):
    """The ember knight (elite): black plate, a red plume, a greatsword."""
    t = Pix(C, C)
    b = k
    t.rect(16, 38, 6, 8, "#2a2230"); t.rect(26, 38, 6, 8, "#2a2230")
    t.rect(13, 22 + b, 22, 17 - b, "#3b3442")
    t.rect(14, 23 + b, 5, 14 - b, "#5d5468")
    t.rect(20, 26 + b, 9, 10 - b, "#7d2c1c")
    t.rect(23, 26 + b, 3, 10 - b, "#ff6a2b")                                 # the ember in the chest
    t.rect(16, 9 + b, 15, 14, "#3b3442")
    t.rect(17, 10 + b, 4, 11, "#5d5468")
    t.rect(21, 15 + b, 10, 2, INK)
    t.rect(26, 15 + b, 3, 2, "#ff6a2b")
    t.rect(18, 3 + b, 4, 7, "#c2302a"); t.rect(15, 5 + b, 4, 4, "#c2302a")   # plume
    for i in range(24):                                                      # greatsword
        t.set(36 + i // 6, 36 + b - i, "#d8dde6"); t.set(37 + i // 6, 36 + b - i, "#8a8f9a")
    t.rect(33, 35 + b, 8, 2, "#c9a24a")
    t.outline()
    return t


def king(k):
    """The Ashen King (the boss): a grey robed giant, a crown of embers, a staff."""
    t = Pix(C, C)
    b = k
    for y in range(18 + b, 47):                                              # robe
        w = 8 + (y - 18 - b) // 2
        t.rect(24 - w, y, 2 * w, 1, "#4a4048")
        t.rect(24 - w, y, 3, 1, "#6f646c")
    t.rect(17, 30 + b, 14, 2, "#c9a24a")
    t.rect(22, 30 + b, 4, 16, "#2e2830")
    t.rect(17, 8 + b, 14, 12, "#9a8f8a")                                     # face of ash
    t.rect(18, 9 + b, 4, 10, "#bdb2ab")
    t.rect(19, 13 + b, 4, 2, "#ff6a2b"); t.rect(26, 13 + b, 4, 2, "#ff6a2b")  # eyes
    t.rect(20, 18 + b, 9, 4, "#d8d2c0")                                      # beard
    t.rect(16, 4 + b, 16, 4, "#c9a24a")                                      # crown
    for x in (16, 21, 26, 31):
        t.rect(x, 1 + b, 2, 3, "#c9a24a")
        t.set(x, 0 + b + k, "#ffb347")
    t.rect(23, 5 + b, 2, 2, "#ff6a2b")
    t.rect(39, 6 + b, 2, 41 - b, "#6b4428")                                  # staff
    t.disc(40, 5 + b, 3, "#ff8a3d"); t.disc(40, 5 + b, 1.5, "#ffe28a")
    t.rect(32, 26 + b, 8, 3, "#4a4048")                                      # the arm to it
    t.outline()
    return t


UNITS = [("warden", warden, "#c2502e"), ("hunter", hunter, "#2f6f6a"), ("imp", imp, "#d9532c"), ("hound", hound, "#6f646c"),
         ("slime", slime, "#4f8a3a"), ("knight", knight, "#3b3442"), ("king", king, "#c9a24a")]
HEROES = 2


def token(art, ring, foe):
    """A unit as a round table token seen from above: a wooden disc, its side's ring, the figure painted on it."""
    t = Pix(C, C)
    cx, cy, r = 24, 26, 20
    t.disc(cx + 2, cy + 3, r, (0, 0, 0, 110))                                # the shadow on the felt
    t.disc(cx, cy, r, "#4a2f1e")
    t.disc(cx, cy, r - 1, ring)
    t.disc(cx, cy, r - 4, "#e9dcc0" if not foe else "#d9c6b0")
    t.disc(cx - 4, cy - 5, r - 9, "#f6eedf" if not foe else "#e6d6c2")
    # The figure's head and shoulders on the disc: its top four pixels inside the disc's top, centred across.
    ys = [y for y in range(C) for x in range(C) if art.get(x, y)[3]]
    xs = [x for y in range(C) for x in range(C) if art.get(x, y)[3]]
    dy = min(ys) - (cy - r + 7) if max(ys) - min(ys) > 2 * r - 10 else (min(ys) + max(ys)) // 2 - cy
    dx = (min(xs) + max(xs)) // 2 - cx
    for y in range(C):
        for x in range(C):
            c = art.get(x + dx, y + dy)
            if c[3] == 255 and (x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2 <= (r - 4) ** 2:
                t.set(x, y, c)
    return t


def sheet():
    s = Pix(8 * C, 4 * C)
    frames = []
    for name, fn, _ in UNITS:
        frames += [fn(0), fn(1)]
    for i, f in enumerate(frames):
        foe = i >= 2 * HEROES
        s.blit(f, (i % 8) * C, (i // 8) * C, flip=foe)
    for j, (name, fn, ring) in enumerate(UNITS):
        foe = j >= HEROES
        art = Pix(C, C)
        art.blit(fn(0), 0, 0, flip=foe)
        i = 14 + j
        s.blit(token(art, ring, foe), (i % 8) * C, (i // 8) * C)
    return s


def portrait(fn, foe):
    """The unit's head, 24 x 24 from its frame, for the unit frames (drawn larger, sampled nearest)."""
    f = Pix(C, C)
    f.blit(fn(0), 0, 0, flip=foe)
    # The head's box: the topmost opaque rows.
    top = next(y for y in range(C) if any(f.get(x, y)[3] for x in range(C)))
    xs = [x for y in range(top, top + 20) for x in range(C) if f.get(x, y)[3]]
    cx = (min(xs) + max(xs)) // 2
    p = Pix(24, 24, rgb("#2b201b"))
    for y in range(24):
        for x in range(24):
            c = f.get(cx - 12 + x, top - 2 + y)
            if c[3]:
                p.set(x, y, c)
    return p


SHEET = """type flipbook
version 1
---
# Written by tools/make-assets.py: 48 px cells, 16 px a world unit (a cell
# is 3 x 3 units). Each unit stands side-on in two idle frames (the foes
# face left, at the heroes), then each is a table token seen from above.
texture "units.png"
pivot 0.5 0
grid 48 48 8 4
"""
for j, (name, _, _) in enumerate(UNITS):
    SHEET += f"clip {name} {{\n  frames {2 * j} {2 * j + 1}\n  durations {0.6 + 0.05 * j:.2f} 0.4\n}}\n"
for j, (name, _, _) in enumerate(UNITS):
    SHEET += f"clip {name}-token {{\n  frames {14 + j}\n  fps 1\n}}\n"

PIXEL = """type import
version 1
---
# Pixel art: lossless, no mips, sampled nearest.
texture {
  kind pixel
}
"""

# A picture drawn at the look's grid already that is not a sprite cell (a
# backdrop, a card's art, a portrait): a pixel look's cook would shrink it
# to its spriteSizes (16, 32, 48) and line it; `conform ui` keeps its size
# and takes only the palette and the 1-bit alpha.
PICTURE = """type import
version 1
---
# Drawn at the look's grid already: the look's palette, never resampled.
conform ui
texture {
  kind pixel
}
"""


# ── the backdrops (640 x 704 px: world x -20..20, y -20..24, at 16 px a unit) ──

BW, BH = 640, 704
CX = BW // 2  # the column of world x = 0
BASE = 24 * 16  # the row of world y = 0, where the units stand


def battlefield():
    p = Pix(BW, BH)
    top, mid, low = rgb("#1b1224"), rgb("#5a2633"), rgb("#d8682f")
    # Sky: banded with an ordered dither between the bands, as 16-bit skies were.
    bayer = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
    for y in range(BASE):
        f = y / BASE
        for x in range(BW):
            v = f ** 1.6 * 9 + bayer[y % 4][x % 4] / 16
            band = min(9, int(v)) / 9
            c = lerp(top, mid, band / 0.6) if band < 0.6 else lerp(mid, low, (band - 0.6) / 0.4)
            p.set(x, y, c)
    # The ember sun, low, with a ring of glow.
    sx, sy = CX + 76, BASE - 70
    for y in range(sy - 60, sy + 60):
        for x in range(sx - 60, sx + 60):
            d = math.hypot(x - sx, y - sy)
            if d < 22:
                p.set(x, y, "#ffd27a" if d < 19 else "#ffb05a")
            elif d < 42 and (x + y) % 2 == 0 and d < 30 + 10 * hashn(x, y):
                p.set(x, y, lerp(p.get(x, y), rgb("#ff9a4a"), 0.35))
    # Far mountains, ruined towers on them, then the near hills.
    for x in range(BW):
        h = 44 + 12 * math.sin(x / 37) + 8 * math.sin(x / 13 + 1.3) + 4 * math.sin(x / 5.7)
        for y in range(int(BASE - h), BASE):
            p.set(x, y, "#3c2236")
    for tx, th, tw in ((40, 30, 8), (156, 40, 9), (180, 28, 7), (292, 46, 10), (310, 26, 6), (488, 36, 9), (590, 32, 8)):
        ground = int(BASE - (44 + 12 * math.sin(tx / 37) + 8 * math.sin(tx / 13 + 1.3)))
        p.rect(tx, ground - th, tw, th, "#2f1a2b")
        for i in range(0, tw, 3):
            p.rect(tx + i, ground - th - 3, 2, 3, "#2f1a2b")
        p.rect(tx + tw // 2 - 1, ground - th + 10, 2, 3, "#ff8a3d")          # a lit window
    for x in range(BW):
        h = 16 + 6 * math.sin(x / 23 + 0.5) + 3 * math.sin(x / 7.3)
        for y in range(int(BASE - h), BASE):
            p.set(x, y, "#28161f" if y > BASE - h + 2 else "#4a2a33")
    # The ash ground: a lit lip, then darker toward the viewer, cracks with embers, pebbles.
    for y in range(BASE, BH):
        f = (y - BASE) / (BH - BASE)
        for x in range(BW):
            c = lerp(rgb("#5c463b"), rgb("#1a1210"), min(1, f * 1.6))
            if hashn(x, y) < 0.06:
                c = lerp(c, rgb("#000000"), 0.25)
            p.set(x, y, c)
    p.rect(0, BASE, BW, 2, "#8a6a52")
    p.rect(0, BASE + 2, BW, 1, "#6e5444")
    for i in range(40):
        x = int(hashn(i, 1) * BW)
        y = BASE + 6 + int(hashn(i, 2) * 180)
        n = 5 + int(hashn(i, 3) * 12)
        for j in range(n):
            xx, yy = x + j, y + int(2 * math.sin(j / 2 + i))
            p.set(xx, yy, "#120c0a")
            if j % 3 == 1:
                p.set(xx, yy - 1, "#c2502e" if hashn(i, j) < 0.5 else "#ff8a3d")
    for i in range(100):
        x = int(hashn(i, 7) * BW)
        y = BASE + 4 + int(hashn(i, 8) * 260)
        p.rect(x, y, 3, 2, "#7a6052"); p.rect(x, y + 2, 3, 1, "#2a1d18")
    # Embers drifting up through the sky.
    for i in range(130):
        x = int(hashn(i, 11) * BW)
        y = int(hashn(i, 12) * (BASE - 20))
        p.set(x, y, "#ffcf7a" if i % 3 else "#ff8a3d")
    return p


def table():
    p = Pix(BW, BH)
    # The plank floor: planks of three tones, a 2 px seam, rows staggered.
    tones = [rgb("#3e2a1e"), rgb("#4a3324"), rgb("#35241a")]
    for y in range(BH):
        row = y // 96
        for x in range(BW):
            xx = x + row * 13
            plank = xx // 28
            c = tones[(plank + row) % 3]
            if xx % 28 < 2 or y % 96 < 2:
                c = rgb("#1a110c")
            elif hashn(x // 3, y // 9, plank) < 0.1:
                c = lerp(c, rgb("#5a3f2c"), 0.7)
            p.set(x, y, c)
    # The table: world x -11.6..11.6, y -4.9..4.3, a wood rim round the felt, its shadow on the floor.
    x0, x1 = CX - 186, CX + 186
    y0, y1 = BASE - 69, BASE + 78
    for y in range(y0 + 6, y1 + 6):
        for x in range(x0 + 5, x1 + 5):
            p.set(x, y, lerp(p.get(x, y), rgb("#000000"), 0.55))
    p.rect(x0, y0, x1 - x0, y1 - y0, "#7a4a28")
    p.rect(x0, y0, x1 - x0, 3, "#b0703c"); p.rect(x0, y0, 3, y1 - y0, "#b0703c")
    p.rect(x0, y1 - 3, x1 - x0, 3, "#43271a"); p.rect(x1 - 3, y0, 3, y1 - y0, "#43271a")
    for y in range(y0 + 8, y1 - 8):
        for x in range(x0 + 8, x1 - 8):
            p.set(x, y, rgb("#2f5240") if hashn(x, y, 3) > 0.09 else rgb("#284838"))
    p.rect(x0 + 7, y0 + 7, x1 - x0 - 14, 2, "#1b3328"); p.rect(x0 + 7, y0 + 7, 2, y1 - y0 - 14, "#1b3328")
    # The line between the sides (world y = -1.8: the heroes' row under it), an ember crest at its middle.
    mid = BASE + 29
    for x in range(x0 + 16, x1 - 16, 6):
        p.rect(x, mid, 3, 2, "#4a7660")
    for a in range(0, 360, 5):
        p.rect(round(CX + 15 * math.cos(math.radians(a))), round(mid + 15 * math.sin(math.radians(a))), 2, 2, "#4a7660")
    p.disc(CX, mid + 1, 5, "#c2502e"); p.disc(CX, mid + 1, 2.5, "#ffb347")
    # Candles, dice and a stack of coins on the felt's corners.
    for cx, cy in ((x0 + 22, y0 + 22), (x1 - 22, y0 + 22)):
        for y in range(cy - 20, cy + 20):
            for x in range(cx - 20, cx + 20):
                d = math.hypot(x - cx, y - cy)
                if 7 < d < 20 and (x + y) % 2 == 0:
                    p.set(x, y, lerp(p.get(x, y), rgb("#ffcf7a"), 0.22))
        p.disc(cx, cy, 7, "#e6d6b0"); p.disc(cx, cy, 4, "#f6eedf"); p.disc(cx, cy, 2, "#ffb347")
    for i, (dx, dy) in enumerate(((x0 + 22, y1 - 30), (x0 + 34, y1 - 24))):
        p.rect(dx, dy, 9, 9, "#e9e2d0"); p.rect(dx, dy + 8, 9, 1, "#a89e88")
        p.rect(dx + 4, dy + 4, 2, 2, "#c2302a")
        if i:
            p.rect(dx + 1, dy + 1, 2, 2, "#c2302a"); p.rect(dx + 6, dy + 6, 2, 2, "#c2302a")
    for j in range(4):
        p.disc(x1 - 28, y1 - 22 - j * 3, 7, "#a8822a", 4); p.disc(x1 - 28, y1 - 23 - j * 3, 6, "#ffc857", 3)
    return p


# ── the cards' art (40 x 32: art kind card, 5:4) ──

TYPES = {"attack": ("#3a1414", "#a83a2a"), "skill": ("#13283d", "#2f6f8f"), "power": ("#3b2a0c", "#c9952a"), "poison": ("#142b14", "#4f8a3a")}


def art_ground(kind):
    a, b = TYPES[kind]
    p = Pix(40, 32)
    for y in range(32):
        for x in range(40):
            f = y / 31 + (0.06 if hashn(x // 2, y // 2, 9) > 0.8 else 0)
            p.set(x, y, lerp(rgb(a), rgb(b), min(1, 1 - f * 0.85)))
    return p


def sword(p, x, y, n=16, hilt="#c9a24a"):
    for i in range(n):
        p.set(x + i, y - i, "#e6ebf2"); p.set(x + i + 1, y - i, "#a7b0bd")
    p.line(x - 3, y - 3, x + 3, y + 3, hilt)
    p.rect(x - 4, y + 2, 3, 3, "#6b4428")


def shield(p, cx, cy, r, face="#5d7fb5", rim="#c9a24a"):
    for y in range(cy - r, cy + r + 4):
        w = r if y < cy else max(0, r - (y - cy) * r // (r + 3))
        p.rect(cx - w, y, 2 * w, 1, rim)
        if w > 1:
            p.rect(cx - w + 1, y, 2 * w - 2, 1, face)
    p.rect(cx - 1, cy - r + 2, 2, 2 * r - 2, rim)


def card_art(name):
    if name == "strike":
        p = art_ground("attack"); sword(p, 10, 26, 18)
    elif name == "defend":
        p = art_ground("skill"); shield(p, 20, 13, 9)
    elif name == "bash":
        p = art_ground("attack")
        p.rect(18, 6, 4, 22, "#6b4428"); p.rect(11, 5, 18, 9, "#8d96a6"); p.rect(12, 6, 6, 7, "#b9c1cf")
        for i in range(7):
            p.set(30 + i, 18 + (i * 5) % 7, "#ffc857")
    elif name == "cleave":
        p = art_ground("attack")
        for a in range(200, 340, 2):
            for r in (11, 12, 13):
                p.set(round(20 + r * math.cos(math.radians(a))), round(24 + r * math.sin(math.radians(a))), "#ffd27a" if r == 12 else "#ff6a2b")
        sword(p, 26, 22, 10)
    elif name == "venom-dart":
        p = art_ground("poison")
        p.line(6, 24, 30, 10, "#cfd6de"); p.line(6, 25, 30, 11, "#8a8f9a")
        p.rect(4, 23, 4, 4, "#c2302a")
        p.disc(32, 10, 2.5, "#7ed957"); p.rect(31, 14, 2, 3, "#7ed957"); p.set(32, 19, "#7ed957")
    elif name == "toxic-cloud":
        p = art_ground("poison")
        for cx, cy, r in ((14, 18, 7), (23, 14, 8), (29, 20, 6), (19, 22, 6)):
            p.disc(cx, cy, r, "#6fbf4a")
        for cx, cy, r in ((14, 16, 4), (23, 11, 4)):
            p.disc(cx, cy, r, "#a8e07a")
        for i in range(6):
            p.set(8 + i * 5, 28 - (i % 2), "#7ed957")
    elif name == "ward":
        p = art_ground("skill"); shield(p, 20, 12, 11, "#2f6f8f", "#ffc857")
        p.rect(16, 11, 8, 2, "#ffe28a"); p.rect(19, 7, 2, 12, "#ffe28a")
    elif name == "insight":
        p = art_ground("skill")
        p.disc(20, 16, 12, "#e6ebf2", 6); p.disc(20, 16, 5, "#5dc2ff"); p.disc(20, 16, 2, INK)
        p.set(18, 14, "#ffffff")
        for i in range(5):
            p.set(6 + i * 7, 4, "#ffe28a")
    elif name == "adrenaline":
        p = art_ground("power")
        pts = [(24, 3), (13, 17), (20, 17), (15, 29), (28, 13), (21, 13)]
        for y in range(3, 30):
            for x in range(10, 30):
                inside = False
                j = len(pts) - 1
                for i in range(len(pts)):
                    xi, yi = pts[i]; xj, yj = pts[j]
                    if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
                        inside = not inside
                    j = i
                if inside:
                    p.set(x, y, "#ffe28a")
    elif name == "flurry":
        p = art_ground("attack")
        for k in range(3):
            sword(p, 6 + k * 10, 24 - k * 2, 9)
    elif name == "iron-hymn":
        p = art_ground("skill")
        for y in range(6, 24):
            w = 4 + (y - 6) // 3
            p.rect(20 - w, y, 2 * w, 1, "#c9a24a")
            p.rect(20 - w, y, 2, 1, "#ffe28a")
        p.rect(11, 24, 18, 2, "#8a5a32"); p.disc(20, 27, 2, "#6b4428")
        for i in range(3):
            p.set(33 + i, 8 + i * 4, "#ffe28a"); p.set(6 - i, 8 + i * 4, "#ffe28a")
    elif name == "heavy-blow":
        p = art_ground("attack")
        p.rect(19, 4, 3, 26, "#6b4428")
        for y in range(4, 18):
            w = 9 - abs(y - 11) // 2
            p.rect(21, y, w, 1, "#b9c1cf"); p.set(21 + w, y, "#e6ebf2")
    elif name == "rally":
        p = art_ground("skill")
        p.rect(12, 4, 2, 26, "#8a5a32")
        for y in range(5, 17):
            p.rect(14, y, 16 - (y - 5) // 4, 1, "#c2502e")
        p.rect(16, 9, 8, 2, "#ffc857")
    elif name == "sunder":
        p = art_ground("attack"); shield(p, 20, 13, 9, "#5a5662", "#8d96a6")
        p.line(14, 4, 26, 26, "#ffd27a"); p.line(15, 4, 27, 26, "#ff6a2b")
    elif name == "kindle":
        p = art_ground("power")
        for y in range(8, 29):
            w = max(1, round(8 * math.sin((y - 8) / 21 * math.pi) * (1 if y > 14 else (y - 6) / 9)))
            p.rect(20 - w, y, 2 * w, 1, "#ff6a2b")
            if w > 3:
                p.rect(20 - w + 3, y, 2 * w - 6, 1, "#ffb347")
        p.disc(20, 24, 3, "#ffe28a")
    else:
        raise ValueError(name)
    return p


CARDS = ["strike", "defend", "bash", "cleave", "venom-dart", "toxic-cloud", "ward", "insight", "adrenaline", "flurry", "iron-hymn", "heavy-blow", "rally", "sunder", "kindle"]


# ── sounds ─────────────────────────────────────────────────────────────────

RATE = 22050


def music():
    """Sixteen bars at 84 BPM in A minor: a low war drum, a bass and a horn's answer, looped seamlessly."""
    beat = 60 / 84
    total = int(64 * beat * RATE)
    out = [0.0] * total
    bass = [110.0, 110.0, 87.31, 98.0]
    for i in range(total):
        t = i / RATE
        bar = int(t / (4 * beat)) % 4
        f = bass[bar]
        out[i] += 0.045 * (math.sin(2 * math.pi * f * t) + 0.4 * math.sin(2 * math.pi * f * 2 * t))
    for b in range(64):
        s0 = int(b * beat * RATE)
        accent = 1.0 if b % 4 == 0 else 0.55
        for i in range(int(0.25 * RATE)):
            t = i / RATE
            v = accent * 0.16 * math.exp(-t / 0.07) * math.sin(2 * math.pi * (70 - 30 * t) * t)
            out[(s0 + i) % total] += v
    horn = [(0, 440.0, 2), (2, 523.25, 1), (3, 493.88, 1), (4, 440.0, 3), (8, 392.0, 2), (10, 440.0, 2), (12, 329.63, 4),
            (32, 440.0, 2), (34, 587.33, 2), (36, 523.25, 2), (38, 493.88, 2), (40, 440.0, 4), (48, 392.0, 2), (50, 349.23, 2), (52, 329.63, 4)]
    for b, f, n in horn:
        s0 = int(b * beat * RATE)
        ln = int(n * beat * RATE)
        for i in range(ln):
            t = i / RATE
            env = min(1, t / 0.06) * math.exp(-t / (n * beat * 0.9))
            v = 0.05 * env * (math.sin(2 * math.pi * f * t) + 0.5 * math.sin(2 * math.pi * f * 2 * t) + 0.25 * math.sin(2 * math.pi * f * 3 * t))
            out[(s0 + i) % total] += v
    return out


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("card.wav", mix(noise(0.12, decay=0.04, gain=0.25, seed=5), tone(0.1, 600, 1200, shape="sine", decay=0.05, gain=0.2)))
    w("hit.wav", mix(noise(0.12, decay=0.035, gain=0.5, seed=3), tone(0.14, 240, 110, shape="square", decay=0.05, gain=0.3)))
    w("heavy.wav", mix(noise(0.3, decay=0.08, gain=0.55, seed=31), tone(0.32, 140, 50, shape="square", decay=0.12, gain=0.4)))
    w("block.wav", concat(tone(0.04, 900, shape="square", decay=0.03, gain=0.25), tone(0.18, 1350, 1200, shape="tri", decay=0.09, gain=0.3)))
    w("hurt.wav", mix(tone(0.25, 330, 110, shape="square", decay=0.1, gain=0.35), noise(0.15, decay=0.05, gain=0.25, seed=9)))
    w("poison.wav", concat(*[tone(0.06, f, f * 0.7, shape="sine", decay=0.05, gain=0.32) for f in (520, 440, 360)]))
    w("buff.wav", concat(*[tone(0.07, f, shape="tri", gain=0.33, decay=0.06) for f in (392.0, 523.25, 659.25)], tone(0.2, 783.99, shape="tri", gain=0.33, decay=0.1)))
    w("draw.wav", concat(noise(0.05, decay=0.02, gain=0.25, seed=17), noise(0.05, decay=0.02, gain=0.22, seed=18), noise(0.05, decay=0.02, gain=0.2, seed=19)))
    w("turn.wav", concat(tone(0.1, 440, shape="tri", decay=0.08, gain=0.3), tone(0.22, 660, shape="tri", decay=0.15, gain=0.3)))
    w("foe-turn.wav", concat(tone(0.12, 220, shape="square", decay=0.1, gain=0.25), tone(0.3, 165, shape="square", decay=0.2, gain=0.25)))
    w("die.wav", concat(tone(0.08, 330, 660, shape="square", decay=0.05, gain=0.28), noise(0.4, decay=0.15, gain=0.35, seed=4)))
    w("heal.wav", concat(*[tone(0.09, f, shape="sine", gain=0.35, decay=0.08) for f in (523.25, 659.25, 783.99, 1046.5)]))
    w("won.wav", concat(*[tone(0.14, f, shape="tri", gain=0.4, decay=0.12) for f in (440.0, 554.37, 659.25, 554.37)], tone(0.8, 880.0, shape="tri", gain=0.4, decay=0.45)))
    w("lost.wav", concat(*[tone(0.28, f, shape="tri", gain=0.4, decay=0.22) for f in (392.0, 349.23, 311.13)], tone(0.9, 220.0, shape="tri", gain=0.4, decay=0.5)))
    w("battle.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook.\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


def main():
    write("sprites/units.png", sheet().png())
    write("sprites/units.png.ogt", PIXEL)
    write("sprites/units.ogt", SHEET)
    for j, (name, fn, _) in enumerate(UNITS):
        write(f"sprites/portraits/{name}.png", portrait(fn, j >= HEROES).png())
        write(f"sprites/portraits/{name}.png.ogt", PICTURE)
    write("sprites/battlefield.png", battlefield().png())
    write("sprites/battlefield.png.ogt", PICTURE)
    write("sprites/table.png", table().png())
    write("sprites/table.png.ogt", PICTURE)
    for name in CARDS:
        write(f"cards/{name}.png", card_art(name).png())
        write(f"cards/{name}.png.ogt", PICTURE)
    sounds()


if __name__ == "__main__":
    main()
