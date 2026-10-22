#!/usr/bin/env python3
"""Regenerates the match3 archetype's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/sprites/gems.png + gems.ogt: the gem flipbook, one row per colour
  (idle with a glint after a long hold, a clear burst), lossless,
- assets/sprites/panel.png (the board's nine-slice backing), backdrop.png
  and ring.png (the selection frame),
- assets/sprites/cells.png + cells.ogt + board.ogt: the checker tileset and
  the 7 x 7 tilemap under the gems,
- assets/audio/*.wav: select, swap, refused, clear, cascade, won and lost,
- assets/fonts/ui.ttf (+ ui.ttf.ogt): Noto Sans SC subset to Latin-1 and the
  HUD's characters (needs fontTools and the project template's font; skipped
  with a note when either is missing, since the committed subset is there).

Standard library only except the font subset; deterministic.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, hex_rgb, png_bytes, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")


INK = "#14161b"  # the brand ink: the backdrop, the panel's shadow and the gems' outlines


def blank(w, h):
    return Canvas(w, h)


def mixc(a, b, t):
    """Mix two #rrggbb colours."""
    ra, ga, ba = hex_rgb(a)
    rb, gb, bb = hex_rgb(b)
    return "#%02x%02x%02x" % tuple(round(x + (y - x) * t) for x, y in ((ra, rb), (ga, gb), (ba, bb)))


def inside_poly(pts):
    def inside(px, py):
        c = False
        j = len(pts) - 1
        for i in range(len(pts)):
            xi, yi = pts[i]; xj, yj = pts[j]
            if (yi > py) != (yj > py) and px < (xj - xi) * (py - yi) / (yj - yi) + xi:
                c = not c
            j = i
        return c
    return inside


def paint_where(c, inside, colour, alpha=1.0):
    c._paint(inside, (0, 0, c.w, c.h), colour, alpha)


# ── the gems: six colours, six cuts, so colour is never the only cue ─────

GEMS = [
    # name, body, dark, light, cut
    ("ruby", "#ff3048", "#a3122a", "#ff9aa6", "round"),
    ("sky", "#2c8bff", "#1449a8", "#9fd0ff", "emerald"),
    ("leaf", "#25d36a", "#0f7a3a", "#a4f5c2", "trillion"),
    ("sun", "#ffc414", "#b57800", "#fff0a0", "kite"),
    ("plum", "#b057ff", "#5f22a8", "#e2bcff", "hex"),
    ("coral", "#ff7424", "#ad3b00", "#ffc49a", "star"),
]

FRAME = 80        # px per frame (the gem sits in it with a margin)
FRAMES = 8        # per colour: 0 idle, 1-3 the glint, 4-7 the clear burst
GLINT = (1, 2, 3)
BURST = (4, 5, 6, 7)


def cut(name, cx, cy, r):
    """The outline of a cut, clockwise from the top (pixels, y down)."""
    def ring(n, rot, rad=r):
        return [(cx + rad * math.cos(rot + i * 2 * math.pi / n), cy + rad * math.sin(rot + i * 2 * math.pi / n)) for i in range(n)]
    if name == "round":
        return ring(12, -math.pi / 2)
    if name == "emerald":  # a square with clipped corners
        k = r * 0.82; s = r * 0.38
        return [(cx - k + s, cy - k), (cx + k - s, cy - k), (cx + k, cy - k + s), (cx + k, cy + k - s),
                (cx + k - s, cy + k), (cx - k + s, cy + k), (cx - k, cy + k - s), (cx - k, cy - k + s)]
    if name == "trillion":
        return ring(3, -math.pi / 2, r * 1.12)
    if name == "kite":
        return [(cx, cy - r * 1.05), (cx + r * 0.78, cy - r * 0.1), (cx, cy + r * 1.05), (cx - r * 0.78, cy - r * 0.1)]
    if name == "hex":
        return ring(6, -math.pi / 2)
    pts = []  # star
    for i in range(10):
        rad = r * (1.08 if i % 2 == 0 else 0.5)
        a = -math.pi / 2 + i * math.pi / 5
        pts.append((cx + rad * math.cos(a), cy + rad * math.sin(a)))
    return pts


def scaled(pts, cx, cy, k, dy=0.0):
    return [(cx + (x - cx) * k, cy + (y - cy) * k + dy) for x, y in pts]


def gem(c, spec, cx, cy, r, bright=0.0):
    """A faceted gem: an ink rim, facets lit from the top left, a table, a highlight."""
    _, body, dark, light, name = spec
    if bright:
        body, dark, light = mixc(body, "#ffffff", bright), mixc(dark, "#ffffff", bright), mixc(light, "#ffffff", bright)
    outer = cut(name, cx, cy, r)
    c.poly(scaled(outer, cx, cy, 1.0, 3.0), INK, 0.55)          # its shadow on the cell
    c.poly(scaled(outer, cx, cy, 1.09), INK)                     # the rim
    c.poly(outer, body)
    table = scaled(outer, cx, cy - r * 0.06, 0.56)
    lx, ly = -0.62, -0.78                                        # towards the light
    n = len(outer)
    for i in range(n):
        a, b = outer[i], outer[(i + 1) % n]
        ta, tb = table[i], table[(i + 1) % n]
        mx, my = (a[0] + b[0]) / 2 - cx, (a[1] + b[1]) / 2 - cy
        ln = math.hypot(mx, my) or 1.0
        lit = (mx * lx + my * ly) / ln                           # -1 away from the light, 1 facing it
        col = mixc(body, light, lit * 0.75) if lit > 0 else mixc(body, dark, -lit * 0.7)
        c.poly([a, b, tb, ta], col)
    c.poly(table, mixc(body, light, 0.35))
    c.poly(scaled(table, cx, cy, 0.55, -r * 0.08), mixc(body, light, 0.6), 0.7)
    c.ellipse(cx - r * 0.3, cy - r * 0.42, r * 0.2, r * 0.12, "#ffffff", 0.75)
    return outer


def sparkle(c, x, y, s, alpha=1.0):
    c.poly([(x, y - s), (x + s * 0.22, y - s * 0.22), (x + s, y), (x + s * 0.22, y + s * 0.22),
            (x, y + s), (x - s * 0.22, y + s * 0.22), (x - s, y), (x - s * 0.22, y - s * 0.22)], "#ffffff", alpha)


def gem_frame(spec, f):
    c = blank(FRAME, FRAME)
    cx = cy = FRAME / 2
    r = FRAME * 0.36
    if f == 0 or f in GLINT:
        outer = gem(c, spec, cx, cy, r)
        if f in GLINT:
            # A diagonal band of light sweeping across the gem, kept inside its outline.
            t = (f - GLINT[0] + 0.5) / len(GLINT)
            inside = inside_poly(outer)
            u0 = -0.6 + 1.2 * t
            band = lambda px, py: inside(px, py) and abs(((px - cx) + (py - cy)) / (2 * r) - u0) < 0.16
            paint_where(c, band, "#ffffff", 0.7)
            if f == GLINT[1]:
                sparkle(c, cx + r * 0.45, cy - r * 0.55, r * 0.42)
        return c
    k = f - BURST[0]  # 0..3: a white-hot gem, then it breaks into a ring and shards
    if k == 0:
        gem(c, spec, cx, cy, r * 1.06, bright=0.55)
        sparkle(c, cx, cy, r * 0.9, 0.9)
        return c
    _, body, _, light, name = spec
    spread = (0.0, 0.55, 0.85, 1.05)[k]
    fade = (1.0, 0.95, 0.7, 0.35)[k]
    rr = r * (0.55 + 0.22 * k)
    c.ring(cx, cy, rr, max(2.0, 7 - 2 * k), light, fade)
    for i in range(6):
        a = -math.pi / 2 + i * math.pi / 3 + 0.3
        d = r * spread
        x, y = cx + d * math.cos(a), cy + d * math.sin(a)
        s = r * (0.36 - 0.07 * k)
        shard = cut(name, x, y, s)
        c.poly(scaled(shard, x, y, 1.15), INK, fade)
        c.poly(shard, body if i % 2 == 0 else light, fade)
    if k < 3:
        sparkle(c, cx, cy, r * (0.5 - 0.12 * k), fade)
    return c


def gem_sheet():
    """gems.png: one row per colour, FRAMES frames across."""
    w, h = FRAME * FRAMES, FRAME * len(GEMS)
    rows = [bytearray(w * 4) for _ in range(h)]
    for gi, spec in enumerate(GEMS):
        for f in range(FRAMES):
            fr = gem_frame(spec, f).rgba()
            for y in range(FRAME):
                rows[gi * FRAME + y][f * FRAME * 4:(f + 1) * FRAME * 4] = fr[y]
    return png_bytes(w, h, rows)


def gem_flipbook():
    """gems.ogt: idle_<colour> loops the glint after a long hold, clear_<colour> bursts once."""
    lines = [
        "type flipbook",
        "version 1",
        "---",
        "# Written by tools/make-assets.py. Clip 2c is colour c's idle loop, 2c + 1 its clear",
        "# burst (src/art.ts relies on that order).",
        'texture "gems.png"',
        "filter linear",
        "pivot 0.5 0.5",
        f"grid {FRAME} {FRAME} {FRAMES} {len(GEMS)}",
    ]
    for gi, (name, *_rest) in enumerate(GEMS):
        base = gi * FRAMES
        idle = " ".join(str(base + f) for f in (0,) + GLINT)
        burst = " ".join(str(base + f) for f in BURST)
        lines += [
            f"clip idle_{name} {{",
            f"  frames {idle}",
            "  durations 2.4 0.05 0.06 0.05",
            "  mode loop",
            "}",
            f"clip clear_{name} {{",
            f"  frames {burst}",
            "  fps 24",
            "  mode once",
            '  event 1 "burst"',
            "}",
        ]
    return "\n".join(lines) + "\n"


# ── the board and the frame ───────────────────────────────────────────────

PANEL = 96
PANEL_INSET = 32  # nine-slice insets in px, every side


def panel():
    """The board's backing, drawn nine-sliced: a soft ink shadow, a rim catching the light, a dark well."""
    c = blank(PANEL, PANEL)
    c.rect(2, 6, 92, 88, "#08090c", radius=26, alpha=0.8)
    c.rect(2, 2, 92, 90, "#2a2f3c", radius=26)
    c.rect(4, 5, 88, 86, "#0e1015", radius=24)
    c.rect(7, 8, 82, 80, "#111318", radius=21)
    return c


def backdrop():
    """A quiet lift of the light behind the board, fading to nothing at its edge (the ink
    around it is the camera's background): a tiny smooth image the sampler stretches."""
    n = 16
    rows = []
    lift = hex_rgb("#262b3b")
    for y in range(n):
        row = bytearray()
        for x in range(n):
            d = min(1.0, math.hypot((x + 0.5) / n - 0.5, (y + 0.5) / n - 0.5) / 0.5)
            t = 1 - d * d * (3 - 2 * d)
            row += bytes(lift) + bytes([round(255 * 0.55 * t)])
        rows.append(bytes(row))
    return png_bytes(n, n, rows)


def ring():
    """The selection frame: a warm rounded square outline with a faint inner glow."""
    n = 96
    c = blank(n, n)
    c.rect(3, 3, 90, 90, "#ffd76a", radius=22, alpha=0.25)
    c.rect(4, 4, 88, 88, "#fff1c2", radius=21)
    lo, hi, rad = 11, 85, 15
    for i, p in enumerate(c.px):
        x, y = i % n + 0.5, i // n + 0.5
        if lo <= x < hi and lo <= y < hi:
            qx = min(max(x, lo + rad), hi - rad)
            qy = min(max(y, lo + rad), hi - rad)
            d = math.hypot(x - qx, y - qy)
            if d <= rad:
                keep = 0.18 * max(0.0, 1 - (min(x - lo, hi - x, y - lo, hi - y)) / 10)
                for j in range(4):
                    p[j] *= keep
    return c


TILE = 64  # px per checker cell


def cells():
    """cells.png: two checker cells (a tileset of 2 x 1)."""
    c = blank(TILE * 2, TILE)
    c.rect(2, 2, TILE - 4, TILE - 4, "#1c1f28", radius=9)
    c.rect(2, 2, TILE - 4, 6, "#232733", radius=9, alpha=0.6)
    c.rect(TILE + 2, 2, TILE - 4, TILE - 4, "#171a21", radius=9)
    c.rect(TILE + 2, 2, TILE - 4, 6, "#1d2029", radius=9, alpha=0.6)
    return c


def cells_tileset():
    return "\n".join([
        "type tileset",
        "version 1",
        "---",
        "# Written by tools/make-assets.py: the board's two checker cells.",
        'texture "sprites/cells.png"',
        f"grid {TILE} {TILE} 2 1",
        "filter nearest",
    ]) + "\n"


def board_map(size=7):
    rows = []
    for y in range(size):
        rows.append('  row "' + "".join("ab"[(x + y) % 2] for x in range(size)) + '"')
    return "\n".join([
        "type tilemap",
        "version 1",
        "---",
        "# Written by tools/make-assets.py: the 7 x 7 checker under the gems, one world unit a cell;",
        "# src/main.ts places its top-left corner at the board's.",
        f"size {size} {size}",
        f"tile {TILE} {TILE}",
        "unit 1",
        'tileset "sprites/cells.ogt"',
        "layer cells {",
        "  z 0",
        '  legend "a" 0',
        '  legend "b" 1',
        *rows,
        "}",
    ]) + "\n"


PIXEL_IMPORT = """type import
version 1
---
# Written by tools/make-assets.py: lossless RGBA8 ({why}).
texture {{
  kind pixel
}}
"""


CHARS = "三消开始游戏点击或拖动交换相邻方块连成同色分数步数目标胜利失败再来一次剩余最高"

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


def main():
    sprites = os.path.join(assets, "sprites")
    os.makedirs(sprites, exist_ok=True)
    preview = os.environ.get("OG_SPRITE_PREVIEW")

    def png(name, data, why=None):
        open(os.path.join(sprites, name), "wb").write(data)
        if why:
            with open(os.path.join(sprites, name + ".ogt"), "w") as f:
                f.write(PIXEL_IMPORT.format(why=why))
        if preview:
            os.makedirs(preview, exist_ok=True)
            open(os.path.join(preview, name), "wb").write(data)

    def canvas_png(name, canvas, why=None):
        png(name, png_bytes(canvas.w, canvas.h, canvas.rgba()), why)

    # The gem sheet is a colour texture (filtered, with mips), not `kind pixel`: the runtime
    # samples a pixel-kind texture nearest even when its flipbook says `filter linear`.
    png("gems.png", gem_sheet())
    with open(os.path.join(sprites, "gems.ogt"), "w") as f:
        f.write(gem_flipbook())
    canvas_png("panel.png", panel())
    png("backdrop.png", backdrop())
    canvas_png("ring.png", ring())
    canvas_png("cells.png", cells(), "a tileset, drawn one texel to a pixel")
    with open(os.path.join(sprites, "cells.ogt"), "w") as f:
        f.write(cells_tileset())
    with open(os.path.join(sprites, "board.ogt"), "w") as f:
        f.write(board_map())

    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    write_wav(os.path.join(a, "select.wav"), tone(0.08, 880, shape="tri", decay=0.04, gain=0.35))
    write_wav(os.path.join(a, "swap.wav"), tone(0.14, 440, 660, shape="tri", decay=0.07, gain=0.35))
    write_wav(os.path.join(a, "refused.wav"), concat(tone(0.08, 220, shape="square", decay=0.05, gain=0.25),
                                                     tone(0.12, 185, shape="square", decay=0.06, gain=0.25)))
    write_wav(os.path.join(a, "clear.wav"), mix(tone(0.3, 660, 1320, shape="sine", decay=0.12, gain=0.4),
                                                 noise(0.12, decay=0.03, gain=0.15, seed=3)))
    write_wav(os.path.join(a, "cascade.wav"), concat(tone(0.08, 784, shape="tri", decay=0.05, gain=0.35),
                                                     tone(0.08, 988, shape="tri", decay=0.05, gain=0.35),
                                                     tone(0.16, 1175, shape="tri", decay=0.08, gain=0.35)))
    write_wav(os.path.join(a, "won.wav"), concat(*[tone(0.12, f, shape="tri", decay=0.1, gain=0.4) for f in (523, 659, 784, 1047)]))
    write_wav(os.path.join(a, "lost.wav"), concat(*[tone(0.18, f, shape="tri", decay=0.12, gain=0.4) for f in (392, 330, 262)]))

    fonts = os.path.join(assets, "fonts")
    os.makedirs(fonts, exist_ok=True)
    font_src = os.path.join(root, "..", "..", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf")
    try:
        subset_font(font_src, os.path.join(fonts, "ui.ttf"), CHARS)
    except Exception as e:  # noqa: BLE001
        print(f"font subset skipped ({e}); the committed assets/fonts/ui.ttf stays")
    with open(os.path.join(fonts, "ui.ttf.ogt"), "w") as f:
        f.write(FONT_IMPORT.format(chars=CHARS))


if __name__ == "__main__":
    main()
