#!/usr/bin/env python3
"""Regenerates Lamp Dive's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/tiles/tomb.png + tomb.ogt: a 16 px pixel-art tileset: the wall
  terrain's sixteen bitmask shapes (stone tops with a lit rim and a brick
  face where the floor is), three floor slabs, the stairs, the relic's
  pedestal and the two fog tiles (unseen, remembered),
- assets/maps/dive.ogt: the 28 x 20 floor every dive carves at run time
  (all wall at first; layers ground, walls, decor, fog),
- assets/sprites/dive.png + dive.ogt: the flipbook sheet (the lamp-bearer,
  the rat, the skeleton, the guardian, the items, the relic) and
  assets/sprites/glow.png (the lamp's light),
- assets/audio/*.wav: steps, hits, hurts, pickups, the stairs, the
  guardian, the fall and the win, and the tomb's music loop,

Standard library only; deterministic.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import png_bytes, write_wav, tone, noise, mix, concat  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")
T = 16
W, H = 28, 20  # the floor's cells


def write(path, data):
    path = os.path.join(assets, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(data, bytes) else "w") as f:
        f.write(data)


def rgb(h, a=255):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


CLEAR = (0, 0, 0, 0)
INK = "#0b0910"


class Pix:
    """Hard-edged pixels (no antialiasing): pixel art, sampled nearest."""

    def __init__(self, w, h):
        self.w, self.h = w, h
        self.p = [CLEAR] * (w * h)

    def set(self, x, y, c):
        if 0 <= x < self.w and 0 <= y < self.h:
            self.p[y * self.w + x] = rgb(c) if isinstance(c, str) else c

    def rect(self, x, y, w, h, c):
        for yy in range(y, y + h):
            for xx in range(x, x + w):
                self.set(xx, yy, c)

    def blit(self, other, ox, oy):
        for y in range(other.h):
            for x in range(other.w):
                c = other.p[y * other.w + x]
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
                    if 0 <= xx < self.w and 0 <= yy < self.h and src[yy * self.w + xx][3]:
                        self.set(x, y, colour)
                        break

    def png(self):
        rows = [bytes(v for c in self.p[y * self.w:(y + 1) * self.w] for v in c) for y in range(self.h)]
        return png_bytes(self.w, self.h, rows)


def hashn(*v):
    h = 2166136261
    for x in v:
        h = ((h ^ (x & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    return h


# ── the tileset ────────────────────────────────────────────────────────────

N, E, S, Wm = 1, 2, 4, 8
STONE = ("#211b27", "#271f2e", "#2d2535")
RIM = "#6b5c7a"
FACE = ("#3a3044", "#43384e")


def wall_tile(mask):
    """A wall cell whose neighbours in `mask` (N E S W) are walls too: a
    stone top, a lit rim where it meets the floor, a brick face below."""
    t = Pix(T, T)
    for y in range(T):
        for x in range(T):
            t.set(x, y, STONE[hashn(x // 4, y // 3, mask & 0) % 3])
    if not mask & N:
        t.rect(0, 0, T, 1, RIM)
    if not mask & Wm:
        t.rect(0, 0, 1, T, RIM)
    if not mask & E:
        t.rect(T - 1, 0, 1, T, "#1a1520")
    if not mask & S:
        # The face the floor below sees: two rows of bricks.
        for y in range(T - 6, T):
            for x in range(T):
                t.set(x, y, FACE[(x // 4 + (y // 3)) % 2])
        t.rect(0, T - 6, T, 1, RIM)
        for y in (T - 3,):
            t.rect(0, y, T, 1, INK)
    return t


def floor_tile(v):
    t = Pix(T, T)
    base = ("#6a5a6e", "#66566a", "#6e5e72")[v]
    t.rect(0, 0, T, T, base)
    t.rect(0, 0, T, 1, "#57495c")
    t.rect(0, 0, 1, T, "#57495c")
    for k in range(3):
        h = hashn(v, k, 3)
        t.set(2 + h % 12, 2 + (h >> 8) % 12, "#7a6a7e")
    if v == 2:
        t.rect(4, 9, 5, 1, "#57495c")
    return t


def stairs_tile():
    t = floor_tile(0)
    for i in range(5):
        y = 3 + i * 2
        shade = ("#2a2230", "#221b28", "#1a1520", "#130f18", "#0b0910")[i]
        t.rect(2 + i, y, 12 - 2 * i, 2, shade)
        t.rect(2 + i, y, 12 - 2 * i, 1, "#6b5c7a" if i == 0 else shade)
    return t


def pedestal_tile():
    t = floor_tile(1)
    for y in range(T):
        for x in range(T):
            d = math.hypot(x - 7.5, y - 7.5)
            if 5.5 <= d <= 6.6:
                t.set(x, y, "#c9a24a")
            elif d < 5.5:
                t.set(x, y, "#5e5068")
    return t


def fog_tile(alpha):
    t = Pix(T, T)
    t.rect(0, 0, T, T, rgb("#08060b", alpha))
    return t


def tileset():
    sheet = Pix(8 * T, 4 * T)
    tiles = [wall_tile(m) for m in range(16)] + [floor_tile(0), floor_tile(1), floor_tile(2), stairs_tile(), pedestal_tile(), fog_tile(255), fog_tile(150)]
    for i, t in enumerate(tiles):
        sheet.blit(t, (i % 8) * T, (i // 8) * T)
    return sheet


TILESET = """type tileset
version 1
---
# Written by tools/make-assets.py: 0-15 the wall terrain's bitmask shapes
# (N 1, E 2, S 4, W 8 filled), 16-18 floor slabs, 19 the stairs down, 20 the
# relic's pedestal, 21 fog never seen, 22 fog remembered.
texture "tiles/tomb.png"
grid 16 16 8 4
filter nearest
solid 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15
terrain wall {
  mode bitmask16
""" + "".join(f"  rule {m} {m}\n" for m in range(16)) + """  default 15
  edge filled
}
"""


def map_text():
    full = "#" * W
    empty = "." * W
    out = ["type tilemap", "version 1", "---",
           "# The floor every dive carves at run time (tools/make-assets.py): 28 x 20",
           "# cells of 16 px, one world unit each, all wall until og.tilemap.setTile",
           "# opens the rooms and corridors; the fog layer starts dark.",
           f"size {W} {H}", "tile 16 16", "unit 1", 'tileset "tiles/tomb.ogt"',
           "layer ground {", "  z -12"]
    out += [f'  row "{empty}"' for _ in range(H)]
    out += ["}", "layer walls {", "  z -11", "  terrain wall", "  collision true", '  legend "#" 1']
    out += [f'  row "{full}"' for _ in range(H)]
    out += ["}", "layer decor {", "  z -10"]
    out += [f'  row "{empty}"' for _ in range(H)]
    out += ["}", "layer fog {", "  z 30", '  legend "#" 21']
    out += [f'  row "{full}"' for _ in range(H)]
    out += ["}"]
    return "\n".join(out) + "\n"


# ── sprites ────────────────────────────────────────────────────────────────

def hero(k):
    t = Pix(T, T)
    # The lamp-bearer: a hooded figure in ochre, the lamp held out to the right.
    t.rect(5, 3, 6, 5, "#6b4a2e")          # hood
    t.rect(6, 5, 4, 3, "#e8c9a0")          # face
    t.set(7, 6, INK); t.set(9, 6, INK)
    t.rect(4, 8, 8, 6, "#a8743a")          # cloak
    t.rect(5, 8, 6, 1, "#c48a45")
    t.rect(5, 14, 2, 2, "#3a2a1c"); t.rect(9, 14, 2, 2, "#3a2a1c")
    t.rect(12, 8, 1, 2, "#6b4a2e")         # arm
    lamp = ("#ffd27a", "#ffe9b0")[k]
    t.rect(12, 10, 3, 3, lamp)
    t.set(13, 9, "#3a2a1c")
    t.outline()
    return t


def rat(k):
    t = Pix(T, T)
    y = 9 + (k & 1)
    t.rect(3, y, 9, 4, "#7a7480")
    t.rect(10, y - 1, 3, 3, "#8a8490")
    t.set(12, y, INK)
    t.set(13, y + 1, "#e08a9a")
    t.rect(1, y + 2, 2, 1, "#e08a9a")
    t.set(0, y + 1 - (k & 1), "#e08a9a")
    t.rect(4, y + 4, 1, 1, "#5a5460"); t.rect(9, y + 4, 1, 1, "#5a5460")
    t.outline()
    return t


def skeleton(k):
    t = Pix(T, T)
    b = 1 if k else 0
    t.rect(5, 1 + b, 6, 5, "#e6e0cf")      # skull
    t.set(6, 3 + b, INK); t.set(9, 3 + b, INK)
    t.rect(7, 5 + b, 2, 1, INK)
    t.rect(7, 6 + b, 2, 6, "#d8d2c0")      # spine
    for y in (7, 9):
        t.rect(5, y + b, 6, 1, "#d8d2c0")  # ribs
    t.rect(4, 7 + b, 1, 4, "#d8d2c0"); t.rect(11, 7 + b, 1, 4, "#d8d2c0")
    t.rect(6, 12, 1, 4, "#d8d2c0"); t.rect(9, 12, 1, 4, "#d8d2c0")
    t.rect(12, 4 + b, 1, 7, "#8a8f9a")     # a rusty blade
    t.outline()
    return t


def guardian(k):
    t = Pix(T, T)
    # The moss-faced guardian: a stone bulk with a mossy face and two embers for eyes.
    t.rect(2, 2, 12, 13, "#5a5662")
    t.rect(3, 3, 10, 5, "#4f7a3a")         # moss
    for x in (4, 7, 10):
        t.rect(x, 7, 2, 2, "#4f7a3a")
    eye = ("#ff8a3a", "#ffc06a")[k]
    t.rect(5, 5, 2, 2, eye); t.rect(9, 5, 2, 2, eye)
    t.rect(4, 10, 8, 1, "#3c3944")
    t.rect(1, 9, 2, 5, "#5a5662"); t.rect(13, 9, 2, 5, "#5a5662")
    t.outline()
    return t


def potion(_):
    t = Pix(T, T)
    t.rect(7, 3, 2, 3, "#c9c2a8")
    t.rect(5, 6, 6, 7, "#d9405a")
    t.rect(6, 7, 2, 2, "#ff8a9a")
    t.outline()
    return t


def blade(_):
    t = Pix(T, T)
    for i in range(8):
        t.set(4 + i, 11 - i, "#d8dde6")
        t.set(5 + i, 11 - i, "#a8afbc")
    t.rect(3, 11, 3, 1, "#c9a24a"); t.rect(4, 10, 1, 3, "#c9a24a")
    t.rect(2, 12, 2, 2, "#6b4a2e")
    t.outline()
    return t


def ward(_):
    t = Pix(T, T)
    t.rect(4, 3, 8, 7, "#5d7fb5")
    t.rect(5, 10, 6, 2, "#5d7fb5"); t.rect(6, 12, 4, 1, "#5d7fb5")
    t.rect(7, 4, 2, 7, "#c9a24a"); t.rect(5, 6, 6, 2, "#c9a24a")
    t.outline()
    return t


def oil(_):
    t = Pix(T, T)
    t.rect(5, 6, 6, 7, "#8a5a2e")
    t.rect(6, 4, 4, 2, "#6b4a2e")
    t.rect(10, 5, 3, 1, "#6b4a2e")
    t.rect(6, 8, 4, 3, "#ffd27a")
    t.outline()
    return t


def coin(k):
    t = Pix(T, T)
    w = (6, 4)[k]
    t.rect(8 - w // 2, 5, w, 7, "#ffcd4a")
    t.rect(8 - w // 2 + 1, 6, 1, 4, "#fff0b0")
    t.outline()
    return t


def relic(k):
    t = Pix(T, T)
    # The relic: a small sun disc on a stem, shining.
    t.rect(7, 9, 2, 5, "#c9a24a")
    t.rect(5, 13, 6, 2, "#c9a24a")
    for y in range(T):
        for x in range(T):
            if math.hypot(x - 7.5, y - 5.5) <= 3.6:
                t.set(x, y, "#ffd27a" if k else "#ffe9b0")
    if k:
        for (x, y) in ((7, 0), (12, 5), (2, 5), (7, 10)):
            t.set(x, y, "#fff6d8")
    t.outline()
    return t


# Frames in sheet order, eight a row: the clips name them (src/art.ts reads clips by position).
FRAMES = [hero(0), hero(1), rat(0), rat(1), skeleton(0), skeleton(1), guardian(0), guardian(1),
          potion(0), blade(0), ward(0), oil(0), coin(0), coin(1), relic(0), relic(1)]


def sheet():
    s = Pix(8 * T, 2 * T)
    for i, f in enumerate(FRAMES):
        s.blit(f, (i % 8) * T, (i // 8) * T)
    return s


SHEET = """type flipbook
version 1
---
# Written by tools/make-assets.py: 16 px frames. Everything breathes on the
# spot (a turn-based dive has no walk cycle: steps are tweens).
texture "dive.png"
pivot 0.5 0.5
grid 16 16 8 2
clip hero {
  frames 0 1
  durations 0.5 0.18
}
clip rat {
  frames 2 3
  fps 4
}
clip skeleton {
  frames 4 5
  fps 2
  mode pingPong
}
clip guardian {
  frames 6 7
  durations 0.6 0.3
}
clip potion {
  frames 8
  fps 1
}
clip blade {
  frames 9
  fps 1
}
clip ward {
  frames 10
  fps 1
}
clip oil {
  frames 11
  fps 1
}
clip coin {
  frames 12 13
  fps 5
  mode pingPong
}
clip relic {
  frames 14 15
  durations 0.4 0.25
}
"""

PIXEL = """type import
version 1
---
# Pixel art: lossless, no mips, sampled nearest.
texture {
  kind pixel
}
"""

LINEAR = """type import
version 1
---
# A soft light: lossless, sampled linear.
texture {
  kind color
}
"""


def glow():
    n = 64
    rows = []
    for y in range(n):
        row = bytearray()
        for x in range(n):
            d = math.hypot(x + 0.5 - n / 2, y + 0.5 - n / 2) / (n / 2)
            a = max(0.0, 1 - d) ** 1.8
            row += bytes([255, 255, 255, round(a * 255)])
        rows.append(bytes(row))
    return png_bytes(n, n, rows)


# ── sounds ─────────────────────────────────────────────────────────────────

RATE = 22050


def music():
    """Eight bars at 66 BPM in D minor: a low drone and a bell that answers itself, looped seamlessly."""
    beat = 60 / 66
    total = int(32 * beat * RATE)
    out = [0.0] * total
    for i in range(total):
        t = i / RATE
        swell = 0.6 + 0.4 * math.sin(2 * math.pi * t / (16 * beat))
        out[i] += 0.05 * swell * (math.sin(2 * math.pi * 73.42 * t) + 0.5 * math.sin(2 * math.pi * 110.0 * t))
    bells = [(0, 587.33), (3, 523.25), (6, 440.0), (8, 466.16), (12, 587.33), (14, 698.46), (19, 659.25), (22, 587.33), (24, 440.0), (28, 523.25)]
    for b, f in bells:
        s0 = int(b * beat * RATE)
        for i in range(int(3 * beat * RATE)):
            t = i / RATE
            v = 0.07 * math.exp(-t / 1.1) * (math.sin(2 * math.pi * f * t) + 0.35 * math.sin(2 * math.pi * f * 2.76 * t))
            out[(s0 + i) % total] += v
    return out


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("hit.wav", mix(noise(0.1, decay=0.03, gain=0.45, seed=3), tone(0.12, 260, 120, shape="square", decay=0.05, gain=0.3)))
    w("hurt.wav", mix(tone(0.25, 330, 110, shape="square", decay=0.1, gain=0.35), noise(0.15, decay=0.05, gain=0.2, seed=9)))
    w("kill.wav", concat(tone(0.06, 220, 440, shape="square", decay=0.04, gain=0.3), noise(0.25, decay=0.1, gain=0.35, seed=4)))
    w("pickup.wav", concat(tone(0.06, 784, shape="square", decay=0.05, gain=0.28), tone(0.16, 1175, shape="square", decay=0.07, gain=0.28)))
    w("item.wav", concat(*[tone(0.07, f, shape="tri", gain=0.35, decay=0.06) for f in (523.25, 659.25, 783.99)], tone(0.22, 1046.5, shape="tri", gain=0.35, decay=0.12)))
    w("drink.wav", concat(*[tone(0.07, f, f * 0.8, shape="sine", gain=0.4, decay=0.05) for f in (330, 392, 440)]))
    w("stairs.wav", mix(noise(0.6, decay=0.25, gain=0.25, seed=21), tone(0.6, 220, 110, shape="tri", decay=0.3, gain=0.4)))
    w("roar.wav", mix(tone(0.7, 110, 70, shape="square", decay=0.3, gain=0.3), noise(0.7, decay=0.3, gain=0.3, seed=13)))
    w("telegraph.wav", concat(tone(0.08, 880, shape="square", decay=0.05, gain=0.25), tone(0.08, 880, shape="square", decay=0.05, gain=0.25)))
    w("fall.wav", concat(*[tone(0.26, f, shape="tri", gain=0.4, decay=0.2) for f in (392.0, 349.23, 293.66)], tone(0.8, 220.0, shape="tri", gain=0.4, decay=0.45)))
    w("won.wav", concat(*[tone(0.14, f, shape="tri", gain=0.4, decay=0.12) for f in (587.33, 698.46, 880.0, 698.46)], tone(0.7, 1174.66, shape="tri", gain=0.4, decay=0.4)))
    w("bump.wav", tone(0.06, 140, 100, shape="tri", decay=0.03, gain=0.4))
    w("tomb.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup", "step": "footstep"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook.\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


def main():
    write("tiles/tomb.png", tileset().png())
    write("tiles/tomb.png.ogt", PIXEL)
    write("tiles/tomb.ogt", TILESET)
    write("maps/dive.ogt", map_text())
    write("sprites/dive.png", sheet().png())
    write("sprites/dive.png.ogt", PIXEL)
    write("sprites/dive.ogt", SHEET)
    write("sprites/glow.png", glow())
    write("sprites/glow.png.ogt", LINEAR)
    sounds()


if __name__ == "__main__":
    main()
