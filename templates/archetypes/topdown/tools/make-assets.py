#!/usr/bin/env python3
"""Regenerates the topdown archetype's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/tiles/dungeon.png + dungeon.ogt: a 16 px pixel-art tileset (the 47
  blob shapes of the wall terrain, floor slabs, a three-frame torch, a door)
  and its tileset text (solid walls, the animated torch, the blob47 rules),
- assets/maps/tomb.tmj: the two-room Tiled map (floor, walls with
  `autotile` = wall and `collision`, the door, torches above the hero, and
  the objects: spawn, enemy, pickups, door, rooms),
- assets/sprites/{hero,slime,items}.png + .ogt: flipbook sheets,
- assets/audio/*.wav: hit, hurt, coin, key and door sounds,
- assets/fonts/ui.ttf: Noto Sans SC subset to Latin-1 (needs fontTools and
  the project template's font; skipped with a note when either is missing).

Standard library only except the font subset; deterministic.
"""
import json, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import png_bytes, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")
T = 16  # pixels per tile and per sprite frame


def rgb(h, a=255):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


CLEAR = (0, 0, 0, 0)


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

    def blit(self, other, ox, oy, flip=False):
        for y in range(other.h):
            for x in range(other.w):
                c = other.p[y * other.w + (other.w - 1 - x if flip else x)]
                if c[3]:
                    self.set(ox + x, oy + y, c)

    def outline(self, colour="#14161b"):
        """A one-pixel outline around every opaque pixel (on transparent ones)."""
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
    """A small deterministic hash for speckles."""
    h = 2166136261
    for x in v:
        h = ((h ^ (x & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    return h


# ── the tileset ──────────────────────────────────────────────────────────

N, NE, E, SE, S, SW, W, NW = 1, 2, 4, 8, 16, 32, 64, 128


def reduce_blob(m):
    """Keep a corner bit only when both of its edge bits are set (og-tilemap's rule)."""
    out = m & (N | E | S | W)
    for corner, a, b in ((NE, N, E), (SE, S, E), (SW, S, W), (NW, N, W)):
        if m & corner and m & a and m & b:
            out |= corner
    return out


BLOB = sorted({reduce_blob(m) for m in range(256)})  # the 47 shapes; tile id = index
assert len(BLOB) == 47
FLOOR = [47, 48, 49]
MOSS = 50
TORCH = [51, 52, 53]
DOOR_TOP, DOOR_BOTTOM = 54, 55
COLS, ROWS = 8, 8

TOP = "#1b1724"
TOP_DOT = "#231e2e"
RIM = "#6d5f80"
FACE = "#4a3f5c"
FACE_HI = "#56496a"
MORTAR = "#2a2336"
FLOOR_A, FLOOR_B, GROUT = "#312c3a", "#38323f", "#24202b"


def wall_tile(m):
    t = Pix(T, T)
    t.rect(0, 0, T, T, TOP)
    for i in range(6):
        h = hashn(m, i)
        t.set(h % T, (h >> 8) % T, TOP_DOT)
    if not m & S:
        # The face towards the viewer: bricks under a light rim.
        t.rect(0, 8, T, 8, FACE)
        for row, y in enumerate(range(8, 16, 4)):
            t.rect(0, y + 3, T, 1, MORTAR)
            off = 0 if row % 2 == 0 else 4
            for x in range(off, T, 8):
                t.rect(x, y, 1, 3, MORTAR)
                t.set(x + 1, y, FACE_HI)
        t.rect(0, 7, T, 1, RIM)
    if not m & N:
        t.rect(0, 0, T, 2, RIM)
    if not m & W:
        t.rect(0, 0, 2, 8 if not m & S else T, RIM)
    if not m & E:
        t.rect(T - 2, 0, 2, 8 if not m & S else T, RIM)
    # Inner corners: both edges filled, the diagonal open.
    if m & N and m & W and not m & NW:
        t.rect(0, 0, 2, 2, RIM)
    if m & N and m & E and not m & NE:
        t.rect(T - 2, 0, 2, 2, RIM)
    if m & S and m & W and not m & SW:
        t.rect(0, T - 2, 2, 2, RIM)
    if m & S and m & E and not m & SE:
        t.rect(T - 2, T - 2, 2, 2, RIM)
    return t


def floor_tile(v):
    t = Pix(T, T)
    t.rect(0, 0, T, T, FLOOR_A)
    # Two slabs per tile, offset by variant.
    t.rect(0, 0, T, 1, GROUT)
    t.rect(0, 0, 1, T, GROUT)
    t.rect(0, 8, T, 1, GROUT)
    t.rect(4 + 4 * v, 1, 1, 7, GROUT)
    t.rect(10 - 3 * v, 9, 1, 7, GROUT)
    for i in range(5 + v * 2):
        h = hashn(v, i, 7)
        t.set(1 + h % 15, 1 + (h >> 8) % 15, FLOOR_B)
    return t


def moss_tile():
    t = floor_tile(0)
    for i in range(18):
        h = hashn(99, i)
        t.set(h % T, (h >> 8) % T, "#2f4a36" if i % 3 else "#3c5e40")
    return t


def torch_tile(frame):
    t = Pix(T, T)
    # The sconce on the wall face, the flame above it.
    # A glow on the bricks behind it, the sconce, the flame above it.
    for y in range(6, 15):
        for x in range(3, 13):
            if (x - 7.5) ** 2 + (y - 10) ** 2 < 20 and (x + y + frame) % 2 == 0:
                t.set(x, y, "#5e3f3a")
    t.rect(6, 12, 4, 2, "#8a6a3a")
    t.rect(7, 14, 2, 2, "#5b4426")
    flames = [
        [(7, 7, 2, 5, "#ff7a2f"), (7, 9, 2, 3, "#ffb347"), (7, 10, 1, 2, "#ffe08a")],
        [(6, 8, 3, 4, "#ff7a2f"), (7, 9, 2, 3, "#ffb347"), (8, 10, 1, 2, "#ffe08a")],
        [(7, 6, 3, 6, "#ff7a2f"), (8, 8, 1, 4, "#ffb347"), (8, 10, 1, 1, "#ffe08a")],
    ][frame]
    for x, y, w, h, c in flames:
        t.rect(x, y, w, h, c)
    t.set(6 + frame, 5, "#ff7a2f")
    return t


def door_tile(top):
    t = Pix(T, T)
    t.rect(0, 0, T, T, "#5a3720")
    for x in (1, 5, 10, 14):
        t.rect(x, 0, 1, T, "#3e2514")
    t.rect(0, 4 if top else 10, T, 2, "#8b8f99")
    if top:
        t.rect(0, 0, T, 2, "#3e2514")
    else:
        t.rect(6, 2, 4, 4, "#d9b04a")  # the lock plate
        t.rect(7, 3, 2, 2, "#5a3720")
        t.rect(0, T - 1, T, 1, "#2a1810")
    return t


def tileset():
    sheet = Pix(COLS * T, ROWS * T)
    tiles = {i: wall_tile(m) for i, m in enumerate(BLOB)}
    for k, i in enumerate(FLOOR):
        tiles[i] = floor_tile(k)
    tiles[MOSS] = moss_tile()
    for k, i in enumerate(TORCH):
        tiles[i] = torch_tile(k)
    tiles[DOOR_TOP] = door_tile(True)
    tiles[DOOR_BOTTOM] = door_tile(False)
    for i, t in tiles.items():
        sheet.blit(t, (i % COLS) * T, (i // COLS) * T)
    return sheet


def tileset_text():
    lines = [
        "type tileset",
        "version 1",
        "---",
        "# Written by tools/make-assets.py: tiles 0-46 are the wall terrain's 47 blob",
        "# shapes, 47-49 floor, 50 moss, 51-53 the torch, 54-55 the door.",
        'texture "tiles/dungeon.png"',
        f"grid {T} {T} {COLS} {ROWS}",
        "filter nearest",
        "solid " + " ".join(str(i) for i in range(len(BLOB))),
        f"tile {TORCH[0]} {{",
        "  frames " + " ".join(str(i) for i in TORCH + [TORCH[1]]),
        "  fps 7",
        "}",
        "terrain wall {",
        "  mode blob47",
    ]
    lines += [f"  rule {m} {i}" for i, m in enumerate(BLOB)]
    lines += [f"  default {BLOB.index(255)}", "  edge filled", "}"]
    return "\n".join(lines) + "\n"


# ── the map ──────────────────────────────────────────────────────────────

MW, MH = 40, 12
DOOR = (19, 5, 2, 2)  # x y w h in cells: the doorway in the dividing wall


def map_layout():
    walls = [[0] * MW for _ in range(MH)]
    for y in range(MH):
        for x in range(MW):
            if y in (0, MH - 1) or x in (0, MW - 1, 19, 20):
                walls[y][x] = 1
    # Pillars: a 2x2 block in the first room, an L and a short wall in the second.
    for x, y in ((6, 3), (7, 3), (6, 4), (7, 4), (26, 3), (26, 4), (26, 5), (27, 5), (28, 5), (33, 7), (34, 7), (35, 7), (35, 8)):
        walls[y][x] = 1
    floor = [[0] * MW for _ in range(MH)]
    for y in range(MH):
        for x in range(MW):
            h = hashn(x, y, 3)
            floor[y][x] = FLOOR[h % 3] if h % 11 else MOSS
    door = [[0] * MW for _ in range(MH)]
    dx, dy, dw, dh = DOOR
    for y in range(dy, dy + dh):
        for x in range(dx, dx + dw):
            door[y][x] = DOOR_TOP if y == dy else DOOR_BOTTOM
    decor = [[0] * MW for _ in range(MH)]
    for x in (3, 10, 16, 23, 30, 36):
        decor[0][x] = TORCH[0]
    return floor, walls, door, decor


def tiled_map():
    floor, walls, door, decor = map_layout()
    gid = lambda t: t + 1  # noqa: E731  (firstgid 1)
    flat = lambda g, f: [f(v) for row in g for v in row]  # noqa: E731

    def tile_layer(i, name, cells, props, f):
        return {
            "id": i, "name": name, "type": "tilelayer", "x": 0, "y": 0,
            "width": MW, "height": MH, "opacity": 1, "visible": True,
            "data": flat(cells, f), "properties": props,
        }

    def prop(n, t, v):
        return {"name": n, "type": t, "value": v}

    def obj(i, name, kind, x, y, w=0, h=0, props=None):
        o = {"id": i, "name": name, "type": kind, "x": x * T, "y": y * T, "width": w * T, "height": h * T,
             "rotation": 0, "visible": True}
        if w == 0 and h == 0:
            o["point"] = True
        if props:
            o["properties"] = props
        return o

    objects = [
        obj(1, "spawn", "spawn", 4.5, 6.5),
        obj(2, "slime", "enemy", 13.5, 5.5),
        obj(3, "coin-0", "coin", 3.5, 2.5),
        obj(4, "coin-1", "coin", 10.5, 8.5),
        obj(5, "coin-2", "coin", 15.5, 2.5),
        obj(6, "key", "key", 16.5, 9.5),
        obj(7, "coin-3", "coin", 24.5, 8.5),
        obj(8, "coin-4", "coin", 31.5, 3.5),
        obj(9, "coin-5", "coin", 37.5, 9.5),
        obj(10, "door", "door", DOOR[0], DOOR[1], DOOR[2], DOOR[3]),
        obj(11, "room-1", "room", 0, 0, 20, MH, [prop("index", "int", 1)]),
        obj(12, "room-2", "room", 20, 0, 20, MH, [prop("index", "int", 2)]),
    ]
    return {
        "compressionlevel": -1, "height": MH, "width": MW, "infinite": False,
        "orientation": "orthogonal", "renderorder": "right-down",
        "tiledversion": "1.10.2", "type": "map", "version": "1.10",
        "tilewidth": T, "tileheight": T, "nextlayerid": 6, "nextobjectid": 13,
        "tilesets": [{"firstgid": 1, "source": "../tiles/dungeon.tsx"}],
        "layers": [
            tile_layer(1, "floor", floor, [], gid),
            tile_layer(2, "walls", walls, [prop("autotile", "string", "wall"), prop("collision", "bool", True)],
                       lambda v: gid(BLOB.index(255)) if v else 0),
            tile_layer(3, "door", door, [], lambda v: gid(v) if v else 0),
            tile_layer(4, "decor", decor, [prop("z", "int", 15)], lambda v: gid(v) if v else 0),
            {"id": 5, "name": "objects", "type": "objectgroup", "x": 0, "y": 0, "opacity": 1, "visible": True,
             "draworder": "topdown", "objects": objects},
        ],
    }


# ── sprites ──────────────────────────────────────────────────────────────

OUT = "#14161b"
SKIN, HAIR, TUNIC, TUNIC_D, BELT, BOOT, STEEL = "#f2c79a", "#e8a33a", "#3f9b62", "#2b6e45", "#7a4a2a", "#4a2f22", "#d8dde6"


def hero_frame(facing, step, attack=-1):
    """facing: 0 down, 1 up, 2 side (right). step: 0-3 walk phase. attack: -1, 0 wind-up, 1 strike."""
    f = Pix(T, T)
    bob = 1 if step in (1, 3) else 0
    # Legs alternate.
    lx = [(5, 9), (5, 9), (5, 9), (5, 9)][step]
    if facing == 2:
        a, b = [(6, 8), (5, 9), (6, 8), (7, 7)][step]
        f.rect(a, 12, 2, 3 - (step == 1), BOOT)
        f.rect(b, 12, 2, 3 - (step == 3), BOOT)
    else:
        left_up = step == 1
        right_up = step == 3
        f.rect(lx[0], 12, 2, 2 if left_up else 3, BOOT)
        f.rect(lx[1], 12, 2, 2 if right_up else 3, BOOT)
    # Body.
    f.rect(4, 7 + bob, 8, 5, TUNIC)
    f.rect(4, 10 + bob, 8, 1, BELT)
    f.rect(4, 7 + bob, 1, 5, TUNIC_D)
    # Head.
    f.rect(4, 1 + bob, 8, 6, SKIN)
    if facing == 1:
        f.rect(4, 1 + bob, 8, 5, HAIR)
    elif facing == 0:
        f.rect(4, 1 + bob, 8, 2, HAIR)
        f.set(6, 4 + bob, OUT)
        f.set(9, 4 + bob, OUT)
    else:
        f.rect(4, 1 + bob, 8, 2, HAIR)
        f.rect(4, 1 + bob, 3, 5, HAIR)
        f.set(10, 4 + bob, OUT)
    # The sword.
    if attack >= 0:
        if facing == 0:
            f.rect(10, 9, 3, 1, BELT)
            f.rect(11, 10, 2 if attack else 1, 6 if attack else 2, STEEL)
        elif facing == 1:
            f.rect(2, 6, 3, 1, BELT)
            f.rect(3, 0 if attack else 4, 2 if attack else 1, 6 if attack else 2, STEEL)
        else:
            f.rect(11, 8, 1, 3, BELT)
            f.rect(12, 9, 4 if attack else 2, 2 if attack else 1, STEEL)
    f.outline()
    return f


def hero_sheet():
    # Rows: down, up, side. Columns: walk 0-3, attack 0-1.
    s = Pix(6 * T, 3 * T)
    for r in range(3):
        for c in range(4):
            s.blit(hero_frame(r, c), c * T, r * T)
        for k in range(2):
            s.blit(hero_frame(r, 0, k), (4 + k) * T, r * T)
    return s


HERO_OGT = """type flipbook
version 1
---
# Written by tools/make-assets.py. Rows: down, up, side (facing right; flipX
# faces left); columns: four walk frames, then the attack's wind-up and
# strike. The clip order is the code's (src/art.ts): idle, walk, attack,
# each down, up, side.
texture "hero.png"
pivot 0.5 0
grid 16 16 6 3
clip idle_down {
  frames 0
  fps 1
}
clip idle_up {
  frames 6
  fps 1
}
clip idle_side {
  frames 12
  fps 1
}
clip walk_down {
  frames 0 1 2 3
  fps 8
  event 1 "step"
  event 3 "step"
}
clip walk_up {
  frames 6 7 8 9
  fps 8
  event 1 "step"
  event 3 "step"
}
clip walk_side {
  frames 12 13 14 15
  fps 8
  event 1 "step"
  event 3 "step"
}
clip attack_down {
  frames 4 5
  durations 0.07 0.18
  mode once
  event 1 "hit"
}
clip attack_up {
  frames 10 11
  durations 0.07 0.18
  mode once
  event 1 "hit"
}
clip attack_side {
  frames 16 17
  durations 0.07 0.18
  mode once
  event 1 "hit"
}
"""

SLIME, SLIME_D, SLIME_HI = "#6fd16a", "#2f7a3a", "#c8f5b0"


def slime_frame(k, hurt=False):
    f = Pix(T, T)
    h = [6, 7, 8, 7][k] if not hurt else 4
    w = [12, 11, 10, 11][k] if not hurt else 14
    x0 = (T - w) // 2
    y0 = T - 1 - h
    f.rect(x0, y0 + 1, w, h, SLIME)
    f.rect(x0 + 1, y0, w - 2, 1, SLIME)
    f.rect(x0, T - 2, w, 1, SLIME_D)
    f.rect(x0 + 2, y0 + 1, 2, 1, SLIME_HI)
    ey = y0 + h // 2
    if hurt:
        f.rect(x0 + 3, ey, 2, 1, OUT)
        f.rect(x0 + w - 5, ey, 2, 1, OUT)
    else:
        f.rect(x0 + 3, ey, 2, 2, "#ffffff")
        f.rect(x0 + w - 5, ey, 2, 2, "#ffffff")
        f.set(x0 + 4, ey + 1, OUT)
        f.set(x0 + w - 4, ey + 1, OUT)
    f.outline()
    return f


def slime_sheet():
    s = Pix(5 * T, T)
    for k in range(4):
        s.blit(slime_frame(k), k * T, 0)
    s.blit(slime_frame(0, True), 4 * T, 0)
    return s


SLIME_OGT = """type flipbook
version 1
---
# Written by tools/make-assets.py: a four-frame bob and a hurt frame.
texture "slime.png"
pivot 0.5 0
grid 16 16 5 1
clip bob {
  frames 0 1 2 3
  fps 6
  mode pingPong
}
clip hurt {
  frames 4
  durations 0.25
  mode once
}
"""

GOLD, GOLD_D, GOLD_HI = "#f2c14e", "#b9862b", "#fff1b0"


def coin_frame(k):
    f = Pix(T, T)
    w = [8, 6, 2, 6][k]
    x0 = (T - w) // 2
    f.rect(x0, 4, w, 8, GOLD)
    f.rect(x0 + 1, 3, max(1, w - 2), 1, GOLD)
    f.rect(x0 + 1, 12, max(1, w - 2), 1, GOLD_D)
    if w > 2:
        f.rect(x0 + 1, 5, 1, 4, GOLD_HI)
    f.outline()
    return f


def key_frame():
    f = Pix(T, T)
    f.rect(3, 4, 5, 5, "#e8d07a")
    f.rect(5, 6, 1, 1, CLEAR)
    f.rect(8, 6, 6, 2, "#e8d07a")
    f.rect(11, 8, 1, 2, "#e8d07a")
    f.rect(13, 8, 1, 2, "#e8d07a")
    f.rect(3, 4, 1, 5, "#b99a3e")
    f.outline()
    return f


def items_sheet():
    s = Pix(5 * T, T)
    for k in range(4):
        s.blit(coin_frame(k), k * T, 0)
    s.blit(key_frame(), 4 * T, 0)
    return s


ITEMS_OGT = """type flipbook
version 1
---
# Written by tools/make-assets.py: a spinning coin and a key.
texture "items.png"
pivot 0.5 0
grid 16 16 5 1
clip coin {
  frames 0 1 2 3
  fps 10
}
clip key {
  frames 4
  fps 1
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

FONT_IMPORT = """type import
version 1
---
# Written by tools/make-assets.py: the HUD and floating texts use Latin-1 only.
font {
  charset latin
  em_px 48
  px_range 6
}
"""


def write(path, data):
    path = os.path.join(assets, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    mode = "wb" if isinstance(data, bytes) else "w"
    with open(path, mode) as f:
        f.write(data)


def main():
    write("tiles/dungeon.png", tileset().png())
    write("tiles/dungeon.png.ogt", PIXEL)
    write("tiles/dungeon.ogt", tileset_text())
    write("maps/tomb.tmj", json.dumps(tiled_map(), indent=1, sort_keys=True) + "\n")
    for name, sheet, ogt in (("hero", hero_sheet(), HERO_OGT), ("slime", slime_sheet(), SLIME_OGT), ("items", items_sheet(), ITEMS_OGT)):
        write(f"sprites/{name}.png", sheet.png())
        write(f"sprites/{name}.png.ogt", PIXEL)
        write(f"sprites/{name}.ogt", ogt)

    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    write_wav(os.path.join(a, "hit.wav"), mix(noise(0.12, decay=0.04, gain=0.5, seed=3), tone(0.12, 220, 110, shape="square", decay=0.05, gain=0.35)))
    write_wav(os.path.join(a, "hurt.wav"), tone(0.25, 440, 140, shape="square", decay=0.1, gain=0.35))
    write_wav(os.path.join(a, "coin.wav"), concat(tone(0.05, 988, shape="square", decay=0.04, gain=0.3), tone(0.18, 1319, shape="square", decay=0.07, gain=0.3)))
    write_wav(os.path.join(a, "key.wav"), concat(tone(0.08, 659, shape="tri", gain=0.4), tone(0.08, 880, shape="tri", gain=0.4), tone(0.2, 1175, shape="tri", gain=0.4, decay=0.1)))
    write_wav(os.path.join(a, "door.wav"), mix(noise(0.4, decay=0.15, gain=0.4, seed=11), tone(0.4, 90, 60, shape="tri", gain=0.5, decay=0.2)))

    fonts = os.path.join(assets, "fonts")
    os.makedirs(fonts, exist_ok=True)
    font_src = os.path.join(root, "..", "..", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf")
    try:
        subset_font(font_src, os.path.join(fonts, "ui.ttf"), "")
    except Exception as e:  # noqa: BLE001
        print(f"font subset skipped ({e}); the committed assets/fonts/ui.ttf stays")
    write("fonts/ui.ttf.ogt", FONT_IMPORT)


if __name__ == "__main__":
    main()
