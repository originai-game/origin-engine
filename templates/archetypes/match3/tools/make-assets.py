#!/usr/bin/env python3
"""Regenerates the match3 archetype's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/models/sprites.glb: every sprite as a unit quad with its own
  emissive, alpha-masked material (src/art.ts lists them in the same order),
- assets/audio/*.wav: select, swap, refused, clear, cascade, won and lost,
- assets/fonts/ui.ttf (+ ui.ttf.ogt): Noto Sans SC subset to Latin-1 and the
  HUD's characters (needs fontTools and the project template's font; skipped
  with a note when either is missing, since the committed subset is there).

Standard library only except the font subset; deterministic.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_glb, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")


def backdrop():
    c = Canvas(16, 256)
    bands = ["#1b1f3a", "#1f2443", "#232a4c", "#283055", "#2d365e", "#333c66"]
    for k, col in enumerate(bands):
        c.rect(0, k * 256 / len(bands), 16, 256 / len(bands) + 1, col)
    return c


def board():
    c = Canvas(256, 256)
    c.rect(0, 0, 256, 256, "#0f1222", radius=20)
    c.rect(6, 6, 244, 244, "#171b30", radius=16)
    return c


def cell():
    c = Canvas(64, 64)
    c.rect(2, 2, 60, 60, "#20253f", radius=8)
    return c


def star_pts(cx, cy, r0, r1, n, rot=-math.pi / 2):
    pts = []
    for i in range(n * 2):
        r = r0 if i % 2 == 0 else r1
        a = rot + i * math.pi / n
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def tile(colour):
    """A rounded paper tile with a motif; the six colours have six shapes, so colour is never the only cue."""
    base, dark, light, shape = {
        "ruby": ("#e8453c", "#a52a24", "#ff9a8f", "circle"),
        "sky": ("#3c8ce8", "#22579c", "#9cc9ff", "square"),
        "leaf": ("#3cb56a", "#23794a", "#9be3b6", "triangle"),
        "sun": ("#f5b82e", "#b07d12", "#ffe28a", "diamond"),
        "plum": ("#9a52d6", "#65308f", "#d1a8f5", "hex"),
        "coral": ("#ff7a3d", "#b84d1d", "#ffbe9a", "star"),
    }[colour]
    c = Canvas(128, 128)
    c.rect(6, 10, 116, 114, dark, radius=22)
    c.rect(6, 4, 116, 112, base, radius=22)
    c.rect(16, 12, 96, 20, light, radius=10, alpha=0.55)
    m = "#fff8ec"
    if shape == "circle":
        c.circle(64, 62, 30, m); c.circle(64, 62, 16, base)
    elif shape == "square":
        c.rect(36, 34, 56, 56, m, radius=8); c.rect(50, 48, 28, 28, base, radius=4)
    elif shape == "triangle":
        c.poly([(64, 26), (98, 90), (30, 90)], m); c.poly([(64, 52), (78, 80), (50, 80)], base)
    elif shape == "diamond":
        c.poly([(64, 24), (98, 62), (64, 100), (30, 62)], m); c.poly([(64, 46), (78, 62), (64, 78), (50, 62)], base)
    elif shape == "hex":
        c.poly([(64 + 34 * math.cos(a), 62 + 34 * math.sin(a)) for a in [i * math.pi / 3 + math.pi / 6 for i in range(6)]], m)
        c.circle(64, 62, 13, base)
    else:
        c.poly(star_pts(64, 64, 36, 15, 5), m)
    return c


def ring():
    """The selection frame: a rounded square with its middle punched out (the mask keeps the frame)."""
    c = Canvas(128, 128)
    c.rect(2, 2, 124, 124, "#fff3c4", radius=26)
    r, lo, hi = 18, 12, 116
    for i, p in enumerate(c.px):
        x, y = i % 128 + 0.5, i // 128 + 0.5
        if lo <= x < hi and lo <= y < hi:
            cx = min(max(x, lo + r), hi - r)
            cy = min(max(y, lo + r), hi - r)
            if (x - cx) ** 2 + (y - cy) ** 2 <= r * r:
                p[0] = p[1] = p[2] = p[3] = 0.0
    return c


COLOURS = ["ruby", "sky", "leaf", "sun", "plum", "coral"]

SPRITES = [("backdrop", backdrop, False), ("board", board, True), ("cell", cell, True)] + \
          [("tile_" + k, (lambda k=k: tile(k)), True) for k in COLOURS] + [("ring", ring, True)]

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
    os.makedirs(os.path.join(assets, "models"), exist_ok=True)
    preview = os.environ.get("OG_SPRITE_PREVIEW")
    out = []
    for name, make, masked in SPRITES:
        canvas = make()
        data = png_bytes(canvas.w, canvas.h, canvas.rgba())
        if preview:
            os.makedirs(preview, exist_ok=True)
            open(os.path.join(preview, name + ".png"), "wb").write(data)
        out.append((name, data, masked))
    write_glb(os.path.join(assets, "models", "sprites.glb"), out)

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
