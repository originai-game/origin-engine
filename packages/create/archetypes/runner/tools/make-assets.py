#!/usr/bin/env python3
"""Regenerates the runner archetype's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/sprites/*.png: pixel-art sheets (`kind pixel`: lossless, no mips,
  sampled nearest) with their flipbook sheets (`.ogt`, `type flipbook`):
  the courier (idle, an 8-frame run with footfall events, jump, fall and a
  crash), the props (crate, fence, moon, cloud) and the spinning coin,
- assets/sprites/sky.png: the night sky's colour bands,
- assets/tiles/city.png + city.ogt: the city tileset (roofs, near and far
  buildings, lit and dark windows, a blinking window and antenna light),
- assets/maps/{far,near,roofs}.ogt: the three parallax strips as tilemaps,
- assets/audio/*.wav: jump, coin, crash and start sounds,
- assets/fonts/ui.ttf: Noto Sans SC subset to Latin-1 and the game's
  strings (needs fontTools and the project template's font; skipped with a
  note when either is missing, since the committed subset is already there).

Standard library only except the font subset; deterministic. Pixel art is
drawn texel by texel (`Pix`), so nearest sampling shows clean pixels.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import hex_rgb, png_bytes, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")

INK = "#14161b"
PIXEL_IMPORT = """type import
version 1
---
# Written by tools/make-assets.py: pixel art, lossless, sampled nearest.
texture {
  kind pixel
}
"""


class Pix:
    """A pixel grid: every texel opaque or transparent, no antialiasing."""

    def __init__(self, w, h):
        self.w, self.h = w, h
        self.px = [None] * (w * h)

    def set(self, x, y, col):
        x, y = int(math.floor(x)), int(math.floor(y))
        if 0 <= x < self.w and 0 <= y < self.h:
            self.px[y * self.w + x] = col

    def get(self, x, y):
        return self.px[y * self.w + x] if 0 <= x < self.w and 0 <= y < self.h else None

    def rect(self, x, y, w, h, col):
        for yy in range(int(round(y)), int(round(y + h))):
            for xx in range(int(round(x)), int(round(x + w))):
                self.set(xx, yy, col)

    def disc(self, cx, cy, r, col, ry=None):
        ry = r if ry is None else ry
        for yy in range(int(cy - ry - 1), int(cy + ry + 2)):
            for xx in range(int(cx - r - 1), int(cx + r + 2)):
                dx = (xx + 0.5 - cx) / max(r, 0.01)
                dy = (yy + 0.5 - cy) / max(ry, 0.01)
                if dx * dx + dy * dy <= 1.0:
                    self.set(xx, yy, col)

    def line(self, x0, y0, x1, y1, col, width=1):
        n = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        for i in range(n + 1):
            t = i / n
            x = x0 + (x1 - x0) * t
            y = y0 + (y1 - y0) * t
            self.rect(round(x - width / 2), round(y - width / 2), width, width, col)

    def blit(self, other, ox, oy):
        for y in range(other.h):
            for x in range(other.w):
                c = other.px[y * other.w + x]
                if c is not None:
                    self.set(ox + x, oy + y, c)

    def outline(self, col):
        """A one-texel outline around every opaque region (readable silhouettes on a dark sky)."""
        add = []
        for y in range(self.h):
            for x in range(self.w):
                if self.px[y * self.w + x] is not None:
                    continue
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    c = self.get(x + dx, y + dy)
                    if c is not None and c != col:
                        add.append((x, y))
                        break
        for x, y in add:
            self.set(x, y, col)

    def png(self):
        rows = []
        for y in range(self.h):
            row = bytearray()
            for x in range(self.w):
                c = self.px[y * self.w + x]
                row += (bytes(hex_rgb(c)) + b"\xff") if c is not None else b"\x00\x00\x00\x00"
            rows.append(bytes(row))
        return png_bytes(self.w, self.h, rows)


SPACING = 2  # texels between frames, so a frame never samples its neighbour


def sheet(frames, fw, fh, cols):
    rows = (len(frames) + cols - 1) // cols
    s = Pix(fw * cols + SPACING * (cols - 1), fh * rows + SPACING * (rows - 1))
    for i, f in enumerate(frames):
        s.blit(f, (i % cols) * (fw + SPACING), (i // cols) * (fh + SPACING))
    return s


# ── the courier: 32 x 40 frames, feet at the bottom row ────────────────────

FW, FH = 32, 40
SKIN, JACKET, DARK, SCARF, HELMET, SHOE, BAG, STRAP, PANTS, PANTS_DARK = (
    "#f0c39a", "#2fa59a", "#1d6f68", "#ff7a3d", "#f4f4f4", "#2a2433", "#c98b3f", "#8a5a24", "#3b3550", "#2c2740")
SLEEVE = "#5fd8c9"


def limb(p, hip, a1, a2, l1, l2, col, w=3):
    """A two-segment limb from `hip`: angles from straight down, positive forward (+x)."""
    k = (hip[0] + l1 * math.sin(a1), hip[1] + l1 * math.cos(a1))
    f = (k[0] + l2 * math.sin(a2), k[1] + l2 * math.cos(a2))
    p.line(hip[0], hip[1], k[0], k[1], col, w)
    p.line(k[0], k[1], f[0], f[1], col, w)
    return f


def courier(legs, arms, bob=0, crash=False, lean=0):
    """legs/arms: ((upper, lower) back, (upper, lower) front), radians from straight down."""
    p = Pix(FW, FH)
    hip = (14 + lean * 0.3, 25 + bob)
    sh = (15 + lean, 15 + bob)
    f = limb(p, hip, legs[0][0], legs[0][1], 7, 7, PANTS_DARK)
    p.rect(f[0] - 1, f[1] - 1, 4, 2, SHOE)
    limb(p, (sh[0], sh[1] + 1), arms[0][0], arms[0][1], 4, 5, DARK, 2)
    # scarf trailing back, the mail bag on its strap
    p.rect(sh[0] - 9, sh[1] - 1, 8, 2, SCARF)
    p.rect(sh[0] - 12, sh[1] + 1, 4, 2, SCARF)
    p.rect(sh[0] - 7, sh[1] + 4, 6, 6, BAG)
    p.rect(sh[0] - 7, sh[1] + 4, 6, 1, "#e0a85a")
    # torso
    p.rect(sh[0] - 3, sh[1] - 1, 8, 11, JACKET)
    p.rect(sh[0] - 3, sh[1] - 1, 2, 11, DARK)
    p.line(sh[0] - 3, sh[1] + 5, sh[0] + 4, sh[1] - 1, STRAP)
    p.rect(sh[0] - 3, sh[1] + 9, 8, 2, PANTS)
    f = limb(p, hip, legs[1][0], legs[1][1], 7, 7, PANTS)
    p.rect(f[0] - 1, f[1] - 1, 4, 2, SHOE)
    h = limb(p, (sh[0] + 2, sh[1] + 1), arms[1][0], arms[1][1], 4, 5, SLEEVE, 2)
    p.rect(h[0] - 1, h[1] - 1, 2, 2, SKIN)
    # head: face, helmet, visor, scarf at the neck
    hx, hy = sh[0] + 2, sh[1] - 6
    p.disc(hx, hy, 4.5, SKIN)
    p.rect(hx - 5, hy - 5, 9, 3, HELMET)
    p.rect(hx - 4, hy - 6, 7, 1, HELMET)
    p.rect(hx + 1, hy - 3, 4, 1, "#c8d4e0")
    p.rect(hx + 2, hy - 1, 1, 1, SHOE)
    p.rect(sh[0] - 3, sh[1] - 2, 8, 2, SCARF)
    if crash:
        for sx, sy in ((hx - 7, hy - 10), (hx + 3, hy - 12), (hx + 8, hy - 7)):
            p.rect(sx, sy, 1, 3, "#ffd23f")
            p.rect(sx - 1, sy + 1, 3, 1, "#ffd23f")
    p.outline(INK)
    return p


def run_frame(k):
    """Frame k of 8: legs and arms swing in opposite phases; contact on frames 2 and 6."""
    ph = 2 * math.pi * k / 8

    def leg(off):
        s = math.sin(ph + off)
        thigh = 0.8 * s
        bend = 0.2 + 1.2 * max(0.0, -math.cos(ph + off))
        return (thigh, thigh - bend)

    def arm(off):
        s = math.sin(ph + off)
        return (-0.8 * s, -0.8 * s - 1.2)

    bob = 0 if k % 4 == 2 else -1
    return courier((leg(math.pi), leg(0)), (arm(0), arm(math.pi)), bob=bob, lean=1)


FRAMES = (
    [run_frame(k) for k in range(8)]                                                          # 0-7 run
    + [courier(((-0.7, -1.7), (1.3, 0.1)), ((-1.0, -2.4), (2.2, 2.8)), bob=-2, lean=1)]       # 8 jump (rising)
    + [courier(((-0.2, -0.9), (0.7, 0.0)), ((-2.2, -2.6), (2.4, 2.8)), bob=-1)]                  # 9 fall
    + [courier(((-1.3, -1.7), (1.2, 1.5)), ((-1.6, -1.1), (1.7, 1.1)), crash=True, lean=l, bob=6)  # 10-13 crash
       for l in (-3, -2, -1, -1)]
    + [courier(((0.08, 0.0), (-0.08, 0.0)), ((0.15, -0.1), (-0.15, 0.1)), bob=b) for b in (0, -1)]  # 14-15 idle
)

COURIER_OGT = """type flipbook
version 1
---
# The courier (tools/make-assets.py): 32 x 40 frames standing on their feet
# (the pivot: the entity's position is the feet, so a squash keeps them down).
texture "courier.png"
pivot 0.5 0
grid 32 40 8 2
spacing 2
clip idle {
  frames 14 15
  fps 2
}
clip run {
  frames 0 1 2 3 4 5 6 7
  fps 14
  event 2 "step"
  event 6 "step"
}
clip jump {
  frames 8
  fps 1
}
clip fall {
  frames 9
  fps 1
}
clip crash {
  frames 10 11 12 13
  fps 12
  mode once
}
"""


# ── props: crate, fence, moon, cloud (one sheet) and the coin ──────────────

def crate():
    p = Pix(16, 16)
    p.rect(0, 0, 16, 16, "#7a4a2a")
    p.rect(2, 2, 12, 12, "#b8743f")
    p.line(2.5, 2.5, 13, 13, "#7a4a2a", 2)
    p.line(13, 2.5, 2.5, 13, "#7a4a2a", 2)
    p.rect(2, 2, 12, 1, "#d08f55")
    p.rect(0, 15, 16, 1, "#5a341c")
    return p


def fence():
    p = Pix(16, 32)
    p.rect(2, 2, 2, 30, "#6b6f80")
    p.rect(12, 2, 2, 30, "#6b6f80")
    for i, y in enumerate((4, 14, 24)):
        p.rect(0, y, 16, 5, "#f2f2f2")
        for x in range(16):
            if ((x + (y - y) + i * 2) // 3) % 2 == 0:
                for d in range(5):
                    p.set((x + d) % 16, y + 4 - d, "#e8453c")
    p.rect(6, 0, 4, 2, "#ffb020")
    p.outline(INK)
    return p


def moon():
    p = Pix(32, 32)
    p.disc(16, 16, 14, "#fff3c4")
    p.disc(11, 12, 2.5, "#ead9a2")
    p.disc(20, 20, 3.5, "#ead9a2")
    p.disc(21, 10, 1.5, "#ead9a2")
    return p


def cloud():
    p = Pix(64, 24)
    for x, y, r in ((14, 15, 7), (25, 11, 9), (38, 12, 8), (49, 15, 6), (32, 17, 7)):
        p.disc(x, y, r, "#2e2a55")
    p.rect(8, 15, 48, 7, "#2e2a55")
    for x, y, r in ((25, 10, 7), (38, 11, 6)):
        p.disc(x, y - 1, r, "#3b3566")
    return p


def props_sheet():
    s = Pix(72, 60)
    s.blit(crate(), 0, 0)
    s.blit(fence(), 18, 0)
    s.blit(moon(), 36, 0)
    s.blit(cloud(), 0, 36)
    return s


PROPS_OGT = """type flipbook
version 1
---
# Props (tools/make-assets.py): one frame each, 2 texels apart.
texture "props.png"
frame crate 0 0 16 16
frame fence 18 0 16 32
frame moon 36 0 32 32
frame cloud 0 36 64 24
clip crate {
  frames crate
}
clip fence {
  frames fence
}
clip moon {
  frames moon
}
clip cloud {
  frames cloud
}
"""


def coin(k):
    """Spin frame k of 6: the coin's width follows |cos|."""
    p = Pix(16, 16)
    w = max(1.0, 6.5 * abs(math.cos(math.pi * k / 6)))
    if w > 2:
        p.disc(8, 8, w, "#b8860b", ry=6.5)
        p.disc(8, 8, w - 1.5, "#ffd23f", ry=5)
        p.rect(8 - max(1, int(w / 3)), 5, max(1, int(w / 3)), 6, "#fff3b0")
    else:
        p.rect(7, 2, 2, 12, "#ffd23f")
        p.rect(7, 2, 1, 12, "#b8860b")
    p.outline(INK)
    return p


COIN_OGT = """type flipbook
version 1
---
# The coin (tools/make-assets.py): six frames of a turn.
texture "coin.png"
grid 16 16 6 1
spacing 2
clip spin {
  frames 0 1 2 3 4 5
  fps 10
}
"""


def sky():
    """Flat bands from night to dusk, one texel wide."""
    bands = ["#0b0e24", "#0f122b", "#141634", "#1a1a3e", "#211d47", "#2a2151", "#34255a", "#3f2a5f"]
    p = Pix(1, len(bands))
    for k, col in enumerate(bands):
        p.set(0, k, col)
    return p


# ── the city tileset: 16 x 16 tiles, 4 x 4 ────────────────────────────────

ROOF, ROOF_LIP, BRICK, MORTAR = "#5ec4b6", "#3a8f86", "#2b2540", "#352e4f"
NEAR, NEAR_EDGE = "#191432", "#272048"
FAR, FAR_EDGE = "#29234f", "#332c5e"
LIT, LIT2, COOL, DARKWIN = "#f7c76b", "#ffd98a", "#7fd1e8", "#100d22"


def bricks(p, rows):
    for y, off in rows:
        for x in range(-8 + off, 16, 8):
            p.rect(x + 1, y, 6, 3, MORTAR)


def tile(kind):
    p = Pix(16, 16)
    if kind == "roof":                       # 0: the roof's teal cap over bricks
        p.rect(0, 0, 16, 16, BRICK)
        p.rect(0, 0, 16, 3, ROOF)
        p.rect(0, 3, 16, 1, ROOF_LIP)
        p.rect(3, 1, 2, 1, "#8fe0d4")
        bricks(p, ((6, 0), (10, 4), (14, 0)))
    elif kind in ("brick", "brick_lit", "brick_dark"):   # 1-3
        p.rect(0, 0, 16, 16, BRICK)
        bricks(p, ((1, 0), (5, 4), (9, 0), (13, 4)))
        if kind != "brick":
            p.rect(4, 3, 8, 9, INK)
            p.rect(5, 4, 6, 7, LIT if kind == "brick_lit" else DARKWIN)
            p.rect(8, 4, 1, 7, INK)
    elif kind == "far":                      # 4
        p.rect(0, 0, 16, 16, FAR)
    elif kind == "far_win":                  # 5: lit specks
        p.rect(0, 0, 16, 16, FAR)
        p.rect(3, 4, 2, 2, LIT)
        p.rect(10, 9, 2, 2, COOL)
        p.rect(11, 2, 2, 2, LIT2)
    elif kind == "far_top":                  # 6: a stepped top
        p.rect(0, 6, 16, 10, FAR)
        p.rect(4, 2, 8, 4, FAR)
        p.rect(0, 6, 4, 1, FAR_EDGE)
        p.rect(12, 6, 4, 1, FAR_EDGE)
        p.rect(4, 2, 8, 1, FAR_EDGE)
    elif kind in ("antenna", "antenna_on"):  # 7, 15: light off and on
        p.rect(7, 3, 2, 13, FAR)
        p.rect(4, 12, 8, 4, FAR)
        p.rect(6, 0, 4, 3, "#ff4a3d" if kind == "antenna_on" else "#4a1d2a")
    elif kind == "near":                     # 8
        p.rect(0, 0, 16, 16, NEAR)
    elif kind in ("near_lit", "near_dark", "near_lit2"):  # 9, 10, 14
        p.rect(0, 0, 16, 16, NEAR)
        a = {"near_lit": (LIT, DARKWIN, DARKWIN, LIT2), "near_dark": (DARKWIN, DARKWIN, DARKWIN, DARKWIN),
             "near_lit2": (LIT2, LIT, DARKWIN, LIT)}[kind]
        for (x, y), col in zip(((3, 3), (9, 3), (3, 10), (9, 10)), a):
            p.rect(x, y, 4, 4, col)
    elif kind == "near_top":                 # 11: cornice
        p.rect(0, 4, 16, 12, NEAR)
        p.rect(0, 3, 16, 2, NEAR_EDGE)
    elif kind == "tank":                     # 12: a water tank on legs
        p.rect(3, 2, 10, 8, NEAR_EDGE)
        p.rect(2, 1, 12, 2, NEAR_EDGE)
        p.rect(7, 0, 2, 1, NEAR_EDGE)
        p.rect(4, 10, 1, 6, NEAR_EDGE)
        p.rect(11, 10, 1, 6, NEAR_EDGE)
        p.line(4, 15, 11, 10, NEAR_EDGE)
    elif kind == "sign":                     # 13: a neon sign
        p.rect(0, 0, 16, 16, NEAR)
        p.rect(2, 4, 12, 8, "#ff4fa3")
        p.rect(3, 5, 10, 6, "#3a1033")
        p.rect(5, 7, 6, 2, "#ff8fd0")
    return p


TILES = ["roof", "brick", "brick_lit", "brick_dark", "far", "far_win", "far_top", "antenna",
         "near", "near_lit", "near_dark", "near_top", "tank", "sign", "near_lit2", "antenna_on"]

CITY_OGT = """type tileset
version 1
---
# The city (tools/make-assets.py): 0 roof, 1-3 bricks, 4-7 far skyline,
# 8-14 near buildings, 15 the antenna's light on.
texture "tiles/city.png"
grid 16 16 4 4
spacing 2
filter nearest
# A window that goes on and off, and an antenna light that blinks.
tile 9 {
  frames 9 14 9 10
  durations 2.5 0.4 1.8 0.6
}
tile 7 {
  frames 7 15
  durations 0.9 0.5
}
"""


def city_sheet():
    """4 x 4 tiles, SPACING apart, each edge extruded into the gap (a tile never samples its neighbour)."""
    step = 16 + SPACING
    s = Pix(4 * step - SPACING, 4 * step - SPACING)
    for i, k in enumerate(TILES):
        t = tile(k)
        ox, oy = (i % 4) * step, (i // 4) * step
        for y in range(-1, 17):
            for x in range(-1, 17):
                c = t.get(min(15, max(0, x)), min(15, max(0, y)))
                if c is not None:
                    s.set(ox + x, oy + y, c)
    return s


# ── the parallax strips (tilemaps) ─────────────────────────────────────────

def far_rows():
    w, h = 40, 9
    heights = [3, 5, 5, 4, 7, 7, 7, 3, 4, 6, 6, 2, 5, 8, 8, 5, 4, 4, 6, 3,
               3, 7, 7, 7, 5, 4, 6, 6, 3, 5, 5, 8, 4, 4, 6, 6, 3, 2, 5, 5]
    rows = []
    for y in range(h):
        level = h - 1 - y
        row = ""
        for x in range(w):
            top = heights[x]
            if level < top - 1:
                row += "w" if (x * 7 + level * 3) % 5 == 0 else "#"
            elif level == top - 1:
                row += "^"
            elif level == top and top >= 7 and x % 3 == 1:
                row += "a"
            else:
                row += "."
        rows.append(row)
    return w, h, rows


def near_rows():
    w, h = 24, 6
    heights = [3, 3, 3, 0, 5, 5, 5, 5, 0, 0, 4, 4, 4, 4, 4, 0, 6, 6, 6, 0, 3, 3, 3, 3]
    rows = []
    for y in range(h):
        level = h - 1 - y
        row = ""
        for x in range(w):
            top = heights[x]
            if top == 0:
                row += "t" if level == 0 and x == 9 else "."
            elif level < top - 1:
                k = (x * 5 + level * 7) % 6
                row += "s" if (x == 12 and level == 2) else ("L" if k == 0 else "l" if k in (1, 4) else "d" if k == 2 else "n")
            elif level == top - 1:
                row += "c"
            else:
                row += "."
        rows.append(row)
    return w, h, rows


def roof_rows():
    w = 16
    windows = "".join("L" if x % 5 == 1 else "D" if x % 5 == 3 else "b" for x in range(w))
    return w, 3, ["=" * w, windows, "b" * w]


def map_ogt(name, unit, size, rows, legend, comment):
    w, h = size
    out = ["type tilemap", "version 1", "---", f"# {comment} (tools/make-assets.py).", f"size {w} {h}", "tile 16 16",
           f"unit {unit}", 'tileset "tiles/city.ogt"', f"layer {name} {{", "  z 0", "  repeat x"]
    for ch, t in legend:
        out.append(f'  legend "{ch}" {t}')
    for r in rows:
        out.append(f'  row "{r}"')
    out.append("}")
    return "\n".join(out) + "\n"


def main():
    def write(rel, data):
        path = os.path.join(assets, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb" if isinstance(data, bytes) else "w") as f:
            f.write(data)

    def png(rel, pix):
        write(rel, pix.png())
        write(rel + ".ogt", PIXEL_IMPORT)

    png("sprites/courier.png", sheet(FRAMES, FW, FH, 8))
    png("sprites/props.png", props_sheet())
    png("sprites/coin.png", sheet([coin(k) for k in range(6)], 16, 16, 6))
    png("sprites/sky.png", sky())
    png("tiles/city.png", city_sheet())
    write("sprites/courier.ogt", COURIER_OGT)
    write("sprites/props.ogt", PROPS_OGT)
    write("sprites/coin.ogt", COIN_OGT)
    write("tiles/city.ogt", CITY_OGT)
    w, h, rows = far_rows()
    write("maps/far.ogt", map_ogt("far", 0.5, (w, h), rows, [("#", 4), ("w", 5), ("^", 6), ("a", 7)],
                                  "The far skyline, 0.5 units a tile"))
    w, h, rows = near_rows()
    write("maps/near.ogt", map_ogt("near", 0.65, (w, h), rows,
                                   [("n", 8), ("L", 9), ("l", 14), ("d", 10), ("c", 11), ("t", 12), ("s", 13)],
                                   "The near buildings, 0.65 units a tile"))
    w, h, rows = roof_rows()
    write("maps/roofs.ogt", map_ogt("roofs", 0.65, (w, h), rows, [("=", 0), ("b", 1), ("L", 2), ("D", 3)],
                                    "The roof the courier runs on, 0.65 units a tile"))

    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    write_wav(os.path.join(a, "jump.wav"), tone(0.18, 320, 760, shape="square", decay=0.08, gain=0.35))
    write_wav(os.path.join(a, "coin.wav"), concat(tone(0.06, 988, shape="square", decay=0.05, gain=0.3),
                                                  tone(0.22, 1319, shape="square", decay=0.08, gain=0.3)))
    write_wav(os.path.join(a, "crash.wav"), mix(noise(0.45, decay=0.12, gain=0.6, seed=7),
                                                 tone(0.45, 180, 60, shape="tri", decay=0.15, gain=0.5)))
    write_wav(os.path.join(a, "start.wav"), concat(tone(0.09, 523, shape="tri", gain=0.4, decay=0.06),
                                                   tone(0.09, 659, shape="tri", gain=0.4, decay=0.06),
                                                   tone(0.2, 784, shape="tri", gain=0.4, decay=0.1)))

    fonts = os.path.join(assets, "fonts")
    os.makedirs(fonts, exist_ok=True)
    font_src = os.path.join(root, "..", "..", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf")
    try:
        subset_font(font_src, os.path.join(fonts, "ui.ttf"), CHARS)
    except Exception as e:  # noqa: BLE001
        print(f"font subset skipped ({e}); the committed assets/fonts/ui.ttf stays")
    with open(os.path.join(fonts, "ui.ttf.ogt"), "w") as f:
        f.write(FONT_IMPORT.format(chars=CHARS))


# Every CJK character the HUD shows (src/systems.ts); Latin-1 comes from the charset.
CHARS = "屋顶快跑点击或按空格开始小跳住高再来一次"

FONT_IMPORT = """type import
version 1
---
# Written by tools/make-assets.py: the atlas holds Latin-1 and the HUD's own
# CJK characters (CHARS there); add a character to CHARS before showing it.
font {{
  charset latin
  chars "{chars}"
  em_px 48
  px_range 6
}}
"""

if __name__ == "__main__":
    main()
