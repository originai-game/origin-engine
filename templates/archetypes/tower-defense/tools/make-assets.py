#!/usr/bin/env python3
"""Regenerates Path Guard's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/tiles/garden.png + garden.ogt: the night garden's 32 px tileset
  (two grasses, the hedge, a stone, the gate, the doorstep, a tower pad,
  a flower), drawn with antialiased shapes,
- assets/maps/garden.ogt: the 9 x 13 field (ground, the hedges and stones
  that block, the pads layer the towers fill at run time),
- assets/sprites/garden.png + garden.ogt: the flipbook sheet (three
  visitors, three plants, the lantern, the shots, the cursor) and
  assets/sprites/ring.png (a tower's range) and spark.png (particles),
- assets/audio/*.wav: the plant, the shots, the pop, the leak, the waves,
  the win and the loss, and the night's music loop,

Standard library only; deterministic.
"""
import math, os, re, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_wav, tone, noise, mix, concat  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")
T = 32  # pixels per tile and per sprite frame

INK = "#0d1a14"


def write(path, data):
    path = os.path.join(assets, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(data, bytes) else "w") as f:
        f.write(data)


def canvas_png(c):
    return png_bytes(c.w, c.h, c.rgba())


def hashn(*v):
    h = 2166136261
    for x in v:
        h = ((h ^ (x & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    return h


# ── the tileset ────────────────────────────────────────────────────────────

GRASS = ("#1d3b2a", "#1d3b2a")
BLADE = "#2c5a3b"


def grass(c, ox, oy, variant):
    c.rect(ox, oy, T, T, GRASS[variant])
    for k in range(7):
        h = hashn(variant, k, 7)
        x = ox + 3 + h % 26
        y = oy + 4 + (h >> 8) % 24
        c.line(x, y + 4, x + ((h >> 16) % 3) - 1, y, 1.3, BLADE)
    if variant == 1:
        for k in range(2):
            h = hashn(9, k)
            c.circle(ox + 6 + h % 20, oy + 6 + (h >> 8) % 20, 1.6, "#e8c55a")


def hedge(c, ox, oy):
    c.rect(ox, oy, T, T, "#10291c")
    for (x, y, r) in ((8, 10, 9), (22, 9, 9), (15, 20, 10), (6, 24, 7), (26, 24, 7)):
        c.circle(ox + x, oy + y, r + 1.2, INK)
    for (x, y, r) in ((8, 10, 9), (22, 9, 9), (15, 20, 10), (6, 24, 7), (26, 24, 7)):
        c.circle(ox + x, oy + y, r, "#1f5234")
        c.circle(ox + x - 2, oy + y - 3, r * 0.45, "#2f7046")


def stone(c, ox, oy):
    grass(c, ox, oy, 0)
    c.ellipse(ox + 16, oy + 25, 12, 4, "#0f2218", 0.8)
    c.poly([(ox + 5, oy + 24), (ox + 8, oy + 10), (ox + 17, oy + 5), (ox + 27, oy + 11), (ox + 28, oy + 24)], INK)
    c.poly([(ox + 7, oy + 23), (ox + 9.5, oy + 11), (ox + 17, oy + 7), (ox + 25.5, oy + 12), (ox + 26, oy + 23)], "#6f7887")
    c.poly([(ox + 10, oy + 13), (ox + 17, oy + 8.5), (ox + 23, oy + 12), (ox + 15, oy + 15)], "#97a0ae")


def gate(c, ox, oy):
    c.rect(ox, oy, T, T, "#4c3a2a")
    for k in range(5):
        h = hashn(3, k)
        c.circle(ox + 4 + h % 24, oy + 4 + (h >> 8) % 24, 1.5, "#5e4834")
    for x in (2, 25):
        c.rect(ox + x - 1, oy, 7, T, INK, radius=2)
        c.rect(ox + x, oy, 5, T, "#8a5a35", radius=2)
        c.rect(ox + x + 1, oy + 2, 1.5, T - 4, "#a87447")


def doorstep(c, ox, oy):
    c.rect(ox, oy, T, T, "#3a3328")
    c.rect(ox + 2, oy + 3, 28, 26, INK, radius=4)
    c.rect(ox + 3, oy + 4, 26, 24, "#8d8270", radius=3)
    c.rect(ox + 5, oy + 6, 22, 3, "#b3a78f", radius=1.5)
    c.circle(ox + 16, oy + 16, 9, "#ffcf73", 0.25)


def pad(c, ox, oy):
    c.ellipse(ox + 16, oy + 18, 13, 10, INK)
    c.ellipse(ox + 16, oy + 18, 12, 9, "#4a3626")
    c.ellipse(ox + 16, oy + 17, 9, 6, "#5d4430")


def flower(c, ox, oy):
    for k in range(3):
        h = hashn(5, k)
        x, y = ox + 6 + h % 20, oy + 6 + (h >> 8) % 20
        for a in range(5):
            ang = a * 2 * math.pi / 5
            c.circle(x + 2.2 * math.cos(ang), y + 2.2 * math.sin(ang), 1.5, "#c9b6f2")
        c.circle(x, y, 1.2, "#ffd166")


def tileset():
    c = Canvas(10 * T, T)
    grass(c, 0, 0, 0)
    grass(c, T, 0, 1)
    hedge(c, 2 * T, 0)
    stone(c, 3 * T, 0)
    gate(c, 4 * T, 0)
    doorstep(c, 5 * T, 0)
    pad(c, 6 * T, 0)
    flower(c, 7 * T, 0)
    pad(c, 8 * T, 0)
    pad(c, 9 * T, 0)
    return c


TILESET = """type tileset
version 1
---
# Written by tools/make-assets.py: 0-1 grass, 2 hedge, 3 stone, 4 the gate,
# 5 the doorstep, 6 a thorn's pad, 7 night flowers, 8 a frost lily's pad,
# 9 a hedge's pad (the pads layer says which plant stands on a cell).
texture "tiles/garden.png"
grid 32 32 10 1
filter linear
solid 2 3
"""

MAP = [
    "HHHHGHHHH",
    "H.,.....H",
    "H.......H",
    "H.R...R.H",
    "H.....,.H",
    "H.,.....H",
    "H...R...H",
    "H.......H",
    "HR.....RH",
    "H....,..H",
    "H..R.R..H",
    "H.,.....H",
    "HHHHDHHHH",
]


def map_text():
    ground = []
    walls = []
    decor = []
    for y, row in enumerate(MAP):
        g, w, d = "", "", ""
        for x, ch in enumerate(row):
            g += {"G": "g", "D": "d", ",": "b"}.get(ch, "a")
            w += {"H": "H", "R": "R"}.get(ch, ".")
            d += "f" if ch == "." and hashn(x, y, 31) % 9 == 0 else "."
        ground.append(g)
        walls.append(w)
        decor.append(d)
    out = ["type tilemap", "version 1", "---",
           "# Path Guard's field (tools/make-assets.py): 9 x 13 cells of 32 px, one",
           "# world unit each. Visitors come through the gate (top) and walk to the",
           "# doorstep (bottom); hedges and stones block them; the pads layer is",
           "# empty until a plant is set on a cell.",
           f"size {len(MAP[0])} {len(MAP)}", "tile 32 32", "unit 1", 'tileset "tiles/garden.ogt"',
           "layer ground {", "  z -12", '  legend "a" 0', '  legend "b" 1', '  legend "g" 4', '  legend "d" 5']
    out += [f'  row "{r}"' for r in ground]
    out += ["}", "layer decor {", "  z -11", '  legend "f" 7']
    out += [f'  row "{r}"' for r in decor]
    out += ["}", "layer walls {", "  z -10", "  collision true", '  legend "H" 2', '  legend "R" 3']
    out += [f'  row "{r}"' for r in walls]
    out += ["}", "layer pads {", "  z -9"]
    out += [f'  row "{"." * len(MAP[0])}"' for _ in MAP]
    out += ["}",
            "object gate {", '  kind "spawn"', "  at 4.5 0.5", "}",
            "object doorstep {", '  kind "goal"', "  at 4.5 12.5", "}"]
    return "\n".join(out) + "\n"


# ── the sprite sheet ───────────────────────────────────────────────────────

def beetle(c, ox, oy, k):
    leg = 2 if k else -2
    for i, y in enumerate((12, 17, 22)):
        d = leg if i % 2 == 0 else -leg
        c.line(ox + 8, oy + y, ox + 3, oy + y + d, 2, INK)
        c.line(ox + 24, oy + y, ox + 29, oy + y - d, 2, INK)
    c.ellipse(ox + 16, oy + 17, 10, 12, INK)
    c.ellipse(ox + 16, oy + 17, 8.6, 10.6, "#5b3fa6")
    c.line(ox + 16, oy + 9, ox + 16, oy + 27, 1.4, INK)
    c.ellipse(ox + 12.5, oy + 14, 2.4, 4, "#8d6fe0")
    c.ellipse(ox + 16, oy + 6, 6, 4, INK)
    c.ellipse(ox + 16, oy + 6, 4.8, 3, "#2b1f4f")
    c.circle(ox + 13.5, oy + 5.5, 1.3, "#ffd166")
    c.circle(ox + 18.5, oy + 5.5, 1.3, "#ffd166")


def moth(c, ox, oy, k):
    lift = 4 if k else 0
    for s in (-1, 1):
        cx = ox + 16 + s * 8
        c.ellipse(cx, oy + 13 - lift * 0.5, 8, 9 - lift, INK)
        c.ellipse(cx, oy + 13 - lift * 0.5, 6.8, 7.8 - lift, "#d9c9f7")
        c.circle(cx + s * 1.5, oy + 12 - lift * 0.5, 2, "#9a83d6")
        c.ellipse(cx - s * 1, oy + 22, 5, 4.5, INK)
        c.ellipse(cx - s * 1, oy + 22, 4, 3.5, "#bfaeea")
    c.ellipse(ox + 16, oy + 17, 3.2, 9, INK)
    c.ellipse(ox + 16, oy + 17, 2.2, 8, "#4f3d86")
    c.line(ox + 15, oy + 8, ox + 12, oy + 3, 1.2, INK)
    c.line(ox + 17, oy + 8, ox + 20, oy + 3, 1.2, INK)


def snail(c, ox, oy, k):
    st = 2 if k else 0
    c.ellipse(ox + 16, oy + 20 + st * 0.5, 7, 11 + st, INK)
    c.ellipse(ox + 16, oy + 20 + st * 0.5, 5.8, 9.8 + st, "#a9c46e")
    c.circle(ox + 16, oy + 15, 10.5, INK)
    c.circle(ox + 16, oy + 15, 9.3, "#d98b4a")
    c.ring(ox + 16, oy + 15, 6.5, 1.6, "#9c5a2a")
    c.ring(ox + 16, oy + 15, 3.5, 1.6, "#9c5a2a")
    c.line(ox + 14, oy + 29 + st, ox + 12, oy + 31, 1.2, INK)
    c.line(ox + 18, oy + 29 + st, ox + 20, oy + 31, 1.2, INK)


def stem(c, ox, oy, sway):
    c.line(ox + 16, oy + 30, ox + 16 + sway, oy + 16, 4, INK)
    c.line(ox + 16, oy + 30, ox + 16 + sway, oy + 16, 2.4, "#3f8a4f")
    c.ellipse(ox + 10, oy + 26, 5, 2.6, INK)
    c.ellipse(ox + 10, oy + 26, 4, 1.8, "#4fa35f")
    c.ellipse(ox + 22, oy + 24, 5, 2.6, INK)
    c.ellipse(ox + 22, oy + 24, 4, 1.8, "#4fa35f")


def thorn(c, ox, oy, k):
    sway = (-1, 1, 0)[k]
    stem(c, ox, oy, sway)
    hx, hy = ox + 16 + sway, oy + 12
    r = 9 if k < 2 else 7.5
    for a in range(8):
        ang = a * math.pi / 4
        c.poly([(hx + (r - 1) * math.cos(ang - 0.25), hy + (r - 1) * math.sin(ang - 0.25)),
                (hx + (r + 4) * math.cos(ang), hy + (r + 4) * math.sin(ang)),
                (hx + (r - 1) * math.cos(ang + 0.25), hy + (r - 1) * math.sin(ang + 0.25))], INK)
    c.circle(hx, hy, r + 1, INK)
    c.circle(hx, hy, r, "#e05a6a")
    c.circle(hx - 2.5, hy - 3, r * 0.35, "#f28b96")
    if k == 2:
        c.circle(hx, hy - 4.5, 3, INK)
    else:
        c.circle(hx - 2.5, hy, 1.2, INK)
        c.circle(hx + 2.5, hy, 1.2, INK)


def frost(c, ox, oy, k):
    sway = (-1, 1, 0)[k]
    stem(c, ox, oy, sway)
    hx, hy = ox + 16 + sway, oy + 12
    open_ = 1.25 if k == 2 else 1.0
    for a in range(6):
        ang = a * math.pi / 3 + math.pi / 6
        px, py = hx + 6 * open_ * math.cos(ang), hy + 6 * open_ * math.sin(ang)
        c.ellipse(px, py, 5.6, 5.6, INK)
    for a in range(6):
        ang = a * math.pi / 3 + math.pi / 6
        px, py = hx + 6 * open_ * math.cos(ang), hy + 6 * open_ * math.sin(ang)
        c.ellipse(px, py, 4.5, 4.5, "#9ad8ff")
    c.circle(hx, hy, 4, "#eaf8ff")
    c.circle(hx, hy, 1.8, "#5cb6e8")


def bush(c, ox, oy, k):
    lift = 1 if k else 0
    for (x, y, r) in ((10, 19, 8), (22, 19, 8), (16, 12 - lift, 9)):
        c.circle(ox + x, oy + y, r + 1.2, INK)
    for (x, y, r) in ((10, 19, 8), (22, 19, 8), (16, 12 - lift, 9)):
        c.circle(ox + x, oy + y, r, "#2f7a45")
        c.circle(ox + x - 2, oy + y - 2.5, r * 0.45, "#47a35f")
    c.circle(ox + 20, oy + 9 - lift, 1.6, "#ffd166")


def lantern(c, ox, oy, k):
    c.rect(ox + 14, oy + 14, 4, 18, INK, radius=1)
    c.rect(ox + 15, oy + 14, 2, 18, "#6b4a2e")
    glow = (0.35, 0.5)[k]
    c.circle(ox + 16, oy + 11, 11, "#ffcf73", glow * 0.5)
    c.rect(ox + 9, oy + 3, 14, 15, INK, radius=3)
    c.rect(ox + 10.5, oy + 4.5, 11, 12, "#ffcf73", radius=2)
    c.rect(ox + 12.5, oy + 6.5, 7, 8, "#fff1c9", radius=1.5)
    c.rect(ox + 8, oy + 2, 16, 3, "#3a2a1c", radius=1.5)


def seed(c, ox, oy):
    c.circle(ox + 16, oy + 16, 5.2, INK)
    c.circle(ox + 16, oy + 16, 4, "#ff8a96")
    c.circle(ox + 14.6, oy + 14.6, 1.4, "#ffd0d6")


def flake(c, ox, oy):
    for a in range(3):
        ang = a * math.pi / 3
        dx, dy = 6 * math.cos(ang), 6 * math.sin(ang)
        c.line(ox + 16 - dx, oy + 16 - dy, ox + 16 + dx, oy + 16 + dy, 3, INK)
    for a in range(3):
        ang = a * math.pi / 3
        dx, dy = 6 * math.cos(ang), 6 * math.sin(ang)
        c.line(ox + 16 - dx, oy + 16 - dy, ox + 16 + dx, oy + 16 + dy, 1.6, "#dff4ff")


def cursor(c, ox, oy, colour):
    for (x, y, sx, sy) in ((2, 2, 1, 1), (30, 2, -1, 1), (2, 30, 1, -1), (30, 30, -1, -1)):
        c.line(ox + x, oy + y, ox + x + sx * 8, oy + y, 3.4, INK)
        c.line(ox + x, oy + y, ox + x, oy + y + sy * 8, 3.4, INK)
        c.line(ox + x, oy + y, ox + x + sx * 8, oy + y, 2, colour)
        c.line(ox + x, oy + y, ox + x, oy + y + sy * 8, 2, colour)


def turned(src):
    """The canvas turned a quarter clockwise: its top edge becomes its right edge."""
    out = Canvas(src.w, src.h)
    for y in range(src.h):
        for x in range(src.w):
            out.px[y * src.w + x] = list(src.px[(src.w - 1 - x) * src.w + y])
    return out


# Frames in sheet order (8 columns): the flipbook's clips name them.
FRAMES = [
    ("beetle", 0), ("beetle", 1), ("moth", 0), ("moth", 1), ("snail", 0), ("snail", 1), ("lantern", 0), ("lantern", 1),
    ("thorn", 0), ("thorn", 1), ("thorn", 2), ("frost", 0), ("frost", 1), ("frost", 2), ("bush", 0), ("bush", 1),
    ("seed", 0), ("flake", 0), ("cursor", 0), ("cursor", 1),
]


def sheet():
    c = Canvas(8 * T, 3 * T)
    draw = {"lantern": lantern, "thorn": thorn, "frost": frost, "bush": bush}
    # The visitors are drawn head up and turned to face +x: og.nav's
    # faceMovement turns a 2D sprite's +x the way it walks.
    walkers = {"beetle": (beetle, 1), "moth": (moth, 1), "snail": (snail, 3)}
    for i, (name, k) in enumerate(FRAMES):
        ox, oy = (i % 8) * T, (i // 8) * T
        if name in walkers:
            f, turns = walkers[name]
            tmp = Canvas(T, T)
            f(tmp, 0, 0, k)
            for _ in range(turns):
                tmp = turned(tmp)
            for y in range(T):
                for x in range(T):
                    c.px[(oy + y) * c.w + ox + x] = tmp.px[y * T + x]
        elif name in draw:
            draw[name](c, ox, oy, k)
        elif name == "seed":
            seed(c, ox, oy)
        elif name == "flake":
            flake(c, ox, oy)
        elif name == "cursor":
            cursor(c, ox, oy, "#fff4dc" if k == 0 else "#ff6b6b")
    return c


SHEET = """type flipbook
version 1
---
# Written by tools/make-assets.py: 32 px frames, eight a row. The visitors
# crawl, the plants sway and open on the frame they shoot ("shoot" ends the
# once-clips), the lantern flickers.
texture "garden.png"
pivot 0.5 0.5
grid 32 32 8 3
clip beetle {
  frames 0 1
  fps 6
}
clip moth {
  frames 2 3
  fps 10
}
clip snail {
  frames 4 5
  fps 3
}
clip lantern {
  frames 6 7
  durations 0.6 0.25
}
clip thorn {
  frames 8 9
  fps 2
  mode pingPong
}
clip thorn_shoot {
  frames 10 8
  durations 0.12 0.1
  mode once
}
clip frost {
  frames 11 12
  fps 2
  mode pingPong
}
clip frost_shoot {
  frames 13 11
  durations 0.14 0.1
  mode once
}
clip hedge {
  frames 14 15
  fps 1.5
  mode pingPong
}
clip seed {
  frames 16
  fps 1
}
clip flake {
  frames 17
  fps 1
}
clip cursor {
  frames 18
  fps 1
}
clip refused {
  frames 19
  fps 1
}
"""


def ring():
    c = Canvas(128, 128)
    c.circle(64, 64, 62, "#fff4dc", 0.08)
    c.ring(64, 64, 62, 2.5, "#fff4dc", 0.7)
    return c


def spark():
    c = Canvas(16, 16)
    for r, a in ((7.5, 0.15), (5, 0.35), (3, 0.8), (1.6, 1.0)):
        c.circle(8, 8, r, "#ffffff", a)
    return c


def glow():
    """A soft round glow, fully clear at its edge (a falloff, not rings)."""
    n = 64
    rows = []
    for y in range(n):
        row = bytearray()
        for x in range(n):
            d = math.hypot(x + 0.5 - n / 2, y + 0.5 - n / 2) / (n / 2)
            a = max(0.0, 1 - d) ** 2.2
            row += bytes([255, 255, 255, round(a * 255)])
        rows.append(bytes(row))
    return png_bytes(n, n, rows)


LINEAR = """type import
version 1
---
# Drawn art: lossless, sampled linear.
texture {
  kind color
}
"""

POP = """type particles
version 1
---
# A visitor turned away: a puff of petals thrown out, drawn as sprites.
maxParticles 48
shape point
spread 180
burst 0 14
duration 0.1
lifetime 0.35 0.6
speed 1.6 3.2
size 0.12 0.22
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 0.86 0.45 1
  key 1 0.79 0.71 0.95 0
}
gravity 0 -2 0
drag 2
texture "sprites/spark.png"
render sprite 20
"""


FIREFLIES = """type particles
version 1
---
# Fireflies over the garden: slow, drifting, blinking out; drawn as glowing sprites above the lawn.
maxParticles 40
shape box 7 9 0
rate 5
lifetime 3 5
speed 0.05 0.25
size 0.12 0.2
sizeOverLife 0 0.2 0.3 1 0.8 1 1 0
colorOverLife {
  key 0 0.85 1 0.45 0
  key 0.2 0.85 1 0.45 0.9
  key 0.8 0.95 0.9 0.4 0.7
  key 1 0.95 0.9 0.4 0
}
noise 0.5 0.4 0.3
texture "sprites/spark.png"
blend additive
render sprite 6
"""


# ── sounds ─────────────────────────────────────────────────────────────────

RATE = 22050


def chord_pad(seconds, freqs, gain):
    n = int(seconds * RATE)
    out = [0.0] * n
    for f in freqs:
        for i in range(n):
            t = i / RATE
            env = min(1.0, t / 0.4) * min(1.0, (seconds - t) / 0.4)
            out[i] += gain * env * (math.sin(2 * math.pi * f * t) + 0.3 * math.sin(4 * math.pi * f * t))
    return out


def pluck(seconds, f, gain=0.25):
    n = int(seconds * RATE)
    return [gain * math.exp(-i / RATE / 0.25) * (math.sin(2 * math.pi * f * i / RATE) + 0.4 * math.sin(6 * math.pi * f * i / RATE)) for i in range(n)]


def music():
    """Four bars at 84 BPM in A minor pentatonic: a soft pad and a plucked arpeggio, looped seamlessly."""
    beat = 60 / 84
    bars = [(220.0, 261.63, 329.63), (196.0, 246.94, 293.66), (174.61, 220.0, 261.63), (196.0, 246.94, 329.63)]
    arp = [440.0, 523.25, 659.25, 587.33, 523.25, 440.0, 392.0, 440.0]
    total = int(16 * beat * RATE)
    out = [0.0] * total
    for b, ch in enumerate(bars):
        pad = chord_pad(4 * beat, ch, 0.05)
        start = int(b * 4 * beat * RATE)
        for i, s in enumerate(pad):
            if start + i < total:
                out[start + i] += s
        for k in range(8):
            f = arp[(k + b * 2) % 8] * (0.5 if b == 2 and k % 2 else 1)
            p = pluck(beat, f, 0.12)
            s0 = int((b * 4 + k * 0.5) * beat * RATE)
            for i, s in enumerate(p):
                out[(s0 + i) % total] += s
    return out


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("plant.wav", mix(noise(0.12, decay=0.03, gain=0.35, seed=5), tone(0.18, 180, 90, shape="tri", decay=0.07, gain=0.5)))
    w("thorn.wav", tone(0.09, 880, 520, shape="tri", decay=0.03, gain=0.35))
    w("frost.wav", mix(tone(0.22, 1568, 1568, decay=0.08, gain=0.22), tone(0.22, 2349, 2349, decay=0.05, gain=0.12)))
    w("pop.wav", concat(tone(0.05, 300, 700, shape="sine", decay=0.03, gain=0.45), tone(0.12, 1319, 1319, shape="square", decay=0.05, gain=0.12)))
    w("leak.wav", mix(tone(0.7, 196, 180, decay=0.3, gain=0.5), tone(0.7, 98, 96, shape="tri", decay=0.35, gain=0.4)))
    w("wave.wav", concat(tone(0.18, 392, shape="tri", gain=0.4, decay=0.12), tone(0.3, 523.25, shape="tri", gain=0.4, decay=0.2)))
    w("clear.wav", concat(*[tone(0.09, f, shape="tri", gain=0.35, decay=0.08) for f in (523.25, 659.25, 783.99)], tone(0.3, 1046.5, shape="tri", gain=0.35, decay=0.2)))
    w("won.wav", concat(*[tone(0.14, f, shape="tri", gain=0.4, decay=0.12) for f in (523.25, 659.25, 783.99, 659.25)], tone(0.6, 1046.5, shape="tri", gain=0.4, decay=0.35)))
    w("lost.wav", concat(*[tone(0.22, f, shape="tri", gain=0.4, decay=0.18) for f in (392.0, 349.23, 311.13)], tone(0.7, 261.63, shape="tri", gain=0.4, decay=0.4)))
    w("night.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook (the UI\'s {name[3:]} sound).\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


def main():
    write("tiles/garden.png", canvas_png(tileset()))
    write("tiles/garden.png.ogt", LINEAR)
    write("tiles/garden.ogt", TILESET)
    write("maps/garden.ogt", map_text())
    write("sprites/garden.png", canvas_png(sheet()))
    write("sprites/garden.png.ogt", LINEAR)
    write("sprites/garden.ogt", SHEET)
    write("sprites/ring.png", canvas_png(ring()))
    write("sprites/ring.png.ogt", LINEAR)
    write("sprites/spark.png", canvas_png(spark()))
    write("sprites/spark.png.ogt", LINEAR)
    write("sprites/glow.png", glow())
    write("sprites/glow.png.ogt", LINEAR)
    write("fx/pop.ogt", POP)
    write("fx/fireflies.ogt", FIREFLIES)
    sounds()


if __name__ == "__main__":
    main()
