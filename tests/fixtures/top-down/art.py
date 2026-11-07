"""Generates the top-down fixture's art (README.md): a 16-texel tileset and
the farm's map, and the loose crop and prop PNGs the cook packs into one
texture (assets/props/props.ogt, `crop each`). Pixel art with dark outlines,
one texel three screen pixels at 960x540. Deterministic; rerun it to change
them:

    python3 tests/fixtures/top-down/art.py
"""

import os
import random

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.join(HERE, "assets")
T = 16  # a tile's texels
OUTLINE = (42, 27, 30)


def rgb(h):
    return tuple(int(h[i : i + 2], 16) for i in (1, 3, 5))


def mix(a, b, t):
    return tuple(int(x + (y - x) * t) for x, y in zip(a, b))


class Canvas:
    def __init__(self, w, h):
        self.img = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        self.px = self.img.load()
        self.w, self.h = w, h

    def put(self, x, y, c):
        if 0 <= x < self.w and 0 <= y < self.h:
            self.px[x, y] = (rgb(c) if isinstance(c, str) else tuple(c)) + (255,)

    def rect(self, x0, y0, w, h, c):
        for y in range(y0, y0 + h):
            for x in range(x0, x0 + w):
                self.put(x, y, c)

    def blob(self, cx, cy, rx, ry, light, mid, dark, seed=0, rough=0.0):
        """An ellipse lit from the top left, three tones and a ragged rim."""
        rnd = random.Random(seed)
        for y in range(int(cy - ry - 1), int(cy + ry + 2)):
            for x in range(int(cx - rx - 1), int(cx + rx + 2)):
                dx, dy = (x + 0.5 - cx) / rx, (y + 0.5 - cy) / ry
                d = dx * dx + dy * dy
                if d > 1 + rough * (rnd.random() - 0.5):
                    continue
                s = dx * 0.6 + dy * 0.8
                c = light if s < -0.35 else (mid if s < 0.35 else dark)
                self.put(x, y, c)

    def outlined(self):
        """A one-texel dark outline round every drawn texel (Stardew's line)."""
        src = self.img.copy().load()
        for y in range(self.h):
            for x in range(self.w):
                if src[x, y][3]:
                    continue
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < self.w and 0 <= ny < self.h and src[nx, ny][3]:
                        self.px[x, y] = OUTLINE + (255,)
                        break
        return self.img


# ---- tiles ------------------------------------------------------------------

GRASS = [rgb("#5f9a3c"), rgb("#538a35"), rgb("#6eab45"), rgb("#467a2e")]
SOIL = [rgb("#8a5a35"), rgb("#76492a"), rgb("#9c6a40"), rgb("#5f3a22")]
WET = [rgb("#5e3b25"), rgb("#4f301d"), rgb("#6b4430"), rgb("#3f2717")]
PATH = [rgb("#b99566"), rgb("#a7845a"), rgb("#c9a676"), rgb("#8e6f4a")]


def tile(seed, pal, kind):
    rnd = random.Random(seed)
    c = Canvas(T, T)
    for y in range(T):
        for x in range(T):
            c.put(x, y, pal[0])
    if kind == "grass":
        for _ in range(18):
            x, y = rnd.randrange(T), rnd.randrange(1, T)
            c.put(x, y, pal[2])
            c.put(x, y - 1, pal[2] if rnd.random() < 0.5 else pal[1])
        for _ in range(10):
            c.put(rnd.randrange(T), rnd.randrange(T), pal[3])
    elif kind == "flowers":
        for _ in range(12):
            c.put(rnd.randrange(T), rnd.randrange(T), pal[2])
        for col in ("#f2e6a0", "#e98fb0", "#f4f4f4"):
            x, y = rnd.randrange(1, T - 1), rnd.randrange(1, T - 1)
            c.put(x, y, col)
            c.put(x + 1, y, rgb(col) if False else mix(rgb(col), pal[0], 0.4))
    elif kind == "soil":
        # Furrows: a dark line every fourth row, lit lumps above it.
        for y in range(T):
            for x in range(T):
                if y % 4 == 3:
                    c.put(x, y, pal[3])
                elif y % 4 == 0 and rnd.random() < 0.6:
                    c.put(x, y, pal[2])
                elif rnd.random() < 0.12:
                    c.put(x, y, pal[1])
    elif kind == "path":
        for _ in range(26):
            c.put(rnd.randrange(T), rnd.randrange(T), pal[rnd.choice((1, 2))])
        for _ in range(3):
            x, y = rnd.randrange(T - 1), rnd.randrange(T - 1)
            c.rect(x, y, 2, 1, pal[2])
            c.rect(x, y + 1, 2, 1, pal[3])
    return c.img


def tiles():
    kinds = [("grass", GRASS)] * 4 + [("flowers", GRASS)] + [("soil", SOIL)] * 2 + [("soil", WET)] + [("path", PATH)] * 4
    sheet = Image.new("RGBA", (T * 8, T * 2), (0, 0, 0, 0))
    for i, (kind, pal) in enumerate(kinds):
        sheet.paste(tile(100 + i, pal, kind), ((i % 8) * T, (i // 8) * T))
    save(sheet, "tiles", "farm.png")
    # Pixel art: lossless, no mips, sampled nearest (not the ETC1S default).
    with open(os.path.join(ASSETS, "tiles", "farm.png.ogt"), "w") as f:
        f.write("type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n")
    with open(os.path.join(ASSETS, "tiles", "farm.ogt"), "w") as f:
        f.write("type tileset\nversion 1\n---\n# Generated by art.py: grass 0-3, flowers 4, soil 5-6, watered soil 7, path 8-11.\n")
        f.write('texture "tiles/farm.png"\ngrid 16 16 8 2\nfilter nearest\n')


# The map: 48 x 30 tiles of grass, a path down and across, two plots.
W, H = 48, 30
PATH_X = (22, 23)
PATH_Y = (17, 18)
PLOTS = ((8, 5, 10, 8), (27, 5, 12, 8), (8, 21, 10, 6))


def ground():
    rnd = random.Random(9)
    rows = []
    for y in range(H):
        row = []
        for x in range(W):
            t = rnd.choice((0, 0, 0, 1, 1, 2, 3)) if rnd.random() > 0.04 else 4
            if x in PATH_X or y in PATH_Y:
                t = 8 + rnd.randrange(4)
            for px, py, pw, ph in PLOTS:
                if px <= x < px + pw and py <= y < py + ph:
                    t = 7 if (x - px) < 3 else 5 + rnd.randrange(2)
            row.append(str(t))
        rows.append("  row " + " ".join(row))
    lines = ["type tilemap", "version 1", "---", "# Generated by art.py: the farm's ground.",
             f"size {W} {H}", "tile 16 16", "unit 1", 'tileset "tiles/farm.ogt"', "layer ground {"]
    lines += rows + ["}"]
    with open(os.path.join(ASSETS, "tiles", "farm-map.ogt"), "w") as f:
        f.write("\n".join(lines) + "\n")


# ---- props -------------------------------------------------------------------

LEAF = ("#7cc04f", "#4f9a3a", "#2f6b2e")


def leaves(c, x, y, n, seed):
    rnd = random.Random(seed)
    for i in range(n):
        lx = x + rnd.randint(-3, 3)
        ly = y - i * 2 - rnd.randint(0, 1)
        side = -1 if i % 2 else 1
        for k in range(3):
            c.put(lx + side * k, ly - k // 2, LEAF[min(k, 2)])
    c.rect(x, y - n * 2, 1, n * 2 + 1, LEAF[1])


def crop(stage, seed, fruit=None):
    c = Canvas(14, 16)
    leaves(c, 7, 14, stage + 1, seed)
    if fruit:
        light, mid, dark, r = fruit
        c.blob(7, 13 - r, r, r * 0.8, light, mid, dark)
    return c.outlined()


def pumpkin():
    c = Canvas(16, 14)
    c.blob(8, 8, 7, 5, "#f6a23c", "#e07a22", "#b4531a")
    for x in (5, 11):
        for y in range(4, 13):
            c.put(x, y, "#b4531a")
    c.rect(7, 1, 2, 3, "#5d7a2e")
    return c.outlined()


def cauliflower():
    c = Canvas(16, 14)
    c.blob(8, 9, 7, 4, LEAF[0], LEAF[1], LEAF[2], seed=3)
    c.blob(8, 7, 4.5, 3.5, "#fbf7e8", "#e9e1c6", "#c9bf9d", seed=4, rough=0.3)
    return c.outlined()


def wheat(seed):
    rnd = random.Random(seed)
    c = Canvas(14, 22)
    for i in range(5):
        x = 2 + i * 2 + rnd.randint(0, 1)
        top = rnd.randint(2, 6)
        c.rect(x, top + 4, 1, 21 - top - 4, "#b9a24a")
        for k in range(4):
            c.put(x, top + k, "#f0d878" if k % 2 else "#d8b552")
            c.put(x + 1, top + k + 1, "#c79e3e")
    return c.outlined()


def sunflower():
    c = Canvas(14, 30)
    c.rect(6, 10, 2, 19, LEAF[1])
    c.rect(2, 18, 4, 2, LEAF[0])
    c.rect(8, 22, 4, 2, LEAF[2])
    c.blob(7, 6, 6, 6, "#ffe066", "#f3c02c", "#d39a1c")
    c.blob(7, 6, 2.5, 2.5, "#7a4a24", "#5a3418", "#3e2210")
    return c.outlined()


def tree():
    c = Canvas(44, 60)
    c.rect(19, 36, 7, 22, "#6b4428")
    c.rect(19, 36, 2, 22, "#8a5a36")
    c.rect(24, 36, 2, 22, "#4f301d")
    c.rect(16, 56, 13, 2, "#4f301d")
    for i, (x, y, r) in enumerate(((22, 24, 18), (12, 30, 10), (32, 30, 10), (22, 12, 12))):
        c.blob(x, y, r, r * 0.85, "#5fae4a", "#3f8a3a", "#2a6332", seed=i, rough=0.25)
    rnd = random.Random(2)
    for _ in range(30):
        c.put(rnd.randint(8, 36), rnd.randint(6, 30), "#78c25a")
    return c.outlined()


def pine():
    c = Canvas(30, 52)
    c.rect(13, 42, 4, 9, "#5a3820")
    for i in range(5):
        y = 6 + i * 8
        r = 4 + i * 2.6
        for yy in range(y, y + 10):
            half = int(r * (yy - y + 2) / 12)
            for x in range(15 - half, 15 + half + 1):
                c.put(x, yy, "#2f6b3e" if x > 15 + half // 3 else ("#3f8a4c" if x > 15 - half // 2 else "#56a65a"))
    return c.outlined()


def bush(seed):
    c = Canvas(22, 15)
    c.blob(11, 8, 10, 6, "#6ab24c", "#4a903c", "#2f6b32", seed=seed, rough=0.3)
    rnd = random.Random(seed)
    for _ in range(4):
        c.put(rnd.randint(4, 18), rnd.randint(4, 10), "#e05a5a")
    return c.outlined()


def rock():
    c = Canvas(16, 12)
    c.blob(8, 7, 7, 4.5, "#b6b3ad", "#8e8a84", "#625e59", seed=5, rough=0.2)
    return c.outlined()


def stump():
    c = Canvas(16, 12)
    c.rect(2, 4, 12, 7, "#6b4428")
    c.blob(8, 4, 6, 2.5, "#d6b07a", "#c09560", "#9c7446")
    return c.outlined()


def fence_h():
    c = Canvas(16, 14)
    for x in (1, 14):
        c.rect(x - 1, 2, 3, 12, "#9c7446")
        c.rect(x - 1, 2, 1, 12, "#c09560")
    c.rect(0, 5, 16, 2, "#b38a58")
    c.rect(0, 9, 16, 2, "#8a6238")
    return c.outlined()


def fence_post():
    c = Canvas(5, 16)
    c.rect(1, 1, 3, 15, "#9c7446")
    c.rect(1, 1, 1, 15, "#c09560")
    return c.outlined()


def lamp():
    c = Canvas(12, 34)
    c.rect(5, 10, 2, 23, "#3a3a44")
    c.rect(3, 31, 6, 2, "#2a2a32")
    c.rect(2, 3, 8, 7, "#2e2e36")
    c.rect(3, 4, 6, 5, "#ffe7a3")
    c.rect(4, 5, 4, 3, "#fff6d8")
    c.rect(1, 2, 10, 1, "#2e2e36")
    return c.outlined()


def crate():
    c = Canvas(16, 15)
    c.rect(1, 1, 14, 13, "#b38a58")
    for y in (1, 7, 13):
        c.rect(1, y, 14, 1, "#7a5532")
    for x in (1, 14):
        c.rect(x, 1, 1, 13, "#7a5532")
    for k in range(5):
        c.put(3 + k * 2, 2 + k, "#8a6238")
    return c.outlined()


def barrel():
    c = Canvas(14, 17)
    for y in range(1, 16):
        w = 5 if y in (1, 15) else 6
        for x in range(7 - w, 7 + w):
            c.put(x, y, "#9c6a40" if x < 6 else ("#7a5030" if x < 10 else "#5f3a22"))
    for y in (4, 12):
        c.rect(1, y, 12, 1, "#4a4a52")
    return c.outlined()


def chest():
    c = Canvas(16, 13)
    c.rect(1, 3, 14, 9, "#9c6a40")
    c.rect(1, 1, 14, 4, "#b98450")
    c.rect(1, 5, 14, 1, "#5f3a22")
    c.rect(7, 4, 2, 3, "#f0c84a")
    return c.outlined()


def scarecrow():
    c = Canvas(20, 30)
    c.rect(9, 8, 2, 21, "#7a5532")
    c.rect(2, 12, 16, 2, "#7a5532")
    c.rect(6, 12, 8, 9, "#5b7fb5")
    c.rect(6, 12, 2, 9, "#7a9cd0")
    c.blob(10, 7, 4, 4, "#f0dca0", "#d8c080", "#b09a5c")
    c.rect(4, 2, 12, 2, "#c79e3e")
    c.rect(6, 0, 8, 3, "#d8b552")
    return c.outlined()


def sign():
    c = Canvas(14, 16)
    c.rect(6, 7, 2, 9, "#7a5532")
    c.rect(1, 1, 12, 7, "#b38a58")
    c.rect(3, 3, 8, 1, "#6b4428")
    c.rect(3, 5, 6, 1, "#6b4428")
    return c.outlined()


def mailbox():
    c = Canvas(12, 18)
    c.rect(5, 8, 2, 10, "#6b4428")
    c.rect(1, 2, 10, 7, "#4a76b0")
    c.rect(1, 2, 10, 2, "#6b96d0")
    c.rect(9, 0, 2, 4, "#e05a5a")
    return c.outlined()


def flowers(seed, col):
    rnd = random.Random(seed)
    c = Canvas(14, 10)
    for _ in range(6):
        x, y = rnd.randint(2, 11), rnd.randint(3, 8)
        c.rect(x, y, 1, 10 - y, LEAF[1])
        c.put(x, y, col)
        c.put(x - 1, y, mix(rgb(col), (255, 255, 255), 0.3))
        c.put(x + 1, y, mix(rgb(col), (0, 0, 0), 0.25))
    return c.outlined()


def hay():
    c = Canvas(18, 13)
    c.blob(9, 7, 8, 5, "#f2d27a", "#d8b552", "#ad8a34", seed=8, rough=0.3)
    return c.outlined()


def well():
    c = Canvas(24, 30)
    c.rect(3, 18, 18, 11, "#8e8a84")
    for y in (20, 24):
        for x in range(3, 21, 4):
            c.rect(x + (y // 4) % 2 * 2, y, 3, 1, "#625e59")
    c.rect(4, 8, 2, 11, "#6b4428")
    c.rect(18, 8, 2, 11, "#6b4428")
    c.rect(1, 4, 22, 4, "#a8423a")
    c.rect(1, 4, 22, 1, "#c95a4e")
    c.rect(5, 17, 14, 2, "#2a3a5c")
    return c.outlined()


def save(img, *path):
    p = os.path.join(ASSETS, *path)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    img.save(p, optimize=True)


PROPS = {
    "parsnip_0": lambda: crop(0, 1),
    "parsnip_1": lambda: crop(1, 2),
    "parsnip_2": lambda: crop(3, 3, ("#fbf2d0", "#ead9a4", "#c7b27a", 2.5)),
    "tomato": lambda: crop(4, 4, ("#ff7a6a", "#e0453a", "#a8282a", 2.2)),
    "pumpkin": pumpkin,
    "cauliflower": cauliflower,
    "wheat_a": lambda: wheat(1),
    "wheat_b": lambda: wheat(2),
    "sunflower": sunflower,
    "tree": tree,
    "pine": pine,
    "bush_a": lambda: bush(1),
    "bush_b": lambda: bush(2),
    "rock": rock,
    "stump": stump,
    "fence_h": fence_h,
    "fence_post": fence_post,
    "lamp": lamp,
    "crate": crate,
    "barrel": barrel,
    "chest": chest,
    "scarecrow": scarecrow,
    "sign": sign,
    "mailbox": mailbox,
    "flowers_a": lambda: flowers(1, "#f7e26b"),
    "flowers_b": lambda: flowers(2, "#e98fb0"),
    "flowers_c": lambda: flowers(3, "#a0c4ff"),
    "hay": hay,
    "well": well,
}


def main():
    tiles()
    ground()
    sheet = ["type flipbook", "version 1", "---",
             "# Generated by art.py: the loose crop and prop PNGs, a still clip",
             "# each, packed by the cook in cells of their own size.",
             "align feet", "crop each", "padding 1", "filter nearest"]
    sizes = []
    for name, fn in PROPS.items():
        img = fn()
        save(img, "props", name + ".png")
        sheet.append(f'source "{name}.png"')
        x0, y0, x1, y1 = img.getbbox()
        sizes.append(f"  {name}: {{ w: {x1 - x0 + 2}, h: {y1 - y0 + 2} }},")
    with open(os.path.join(ASSETS, "props", "props.ogt"), "w") as f:
        f.write("\n".join(sheet) + "\n")
    os.makedirs(os.path.join(HERE, "src"), exist_ok=True)
    with open(os.path.join(HERE, "src", "art.ts"), "w") as f:
        f.write("// Generated by art.py: each prop's cell in props/props.ogt (texels), and the map's layout.\n")
        f.write("export const PROPS = {\n" + "\n".join(sizes) + "\n} as const;\n")
        f.write(f"export const MAP = {{ w: {W}, h: {H}, pathX: {list(PATH_X)}, pathY: {list(PATH_Y)}, plots: {[list(p) for p in PLOTS]} }} as const;\n")


if __name__ == "__main__":
    main()
