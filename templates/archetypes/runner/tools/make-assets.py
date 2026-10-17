#!/usr/bin/env python3
"""Regenerates the runner archetype's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/models/sprites.glb: every sprite as a unit quad with its own
  emissive, alpha-masked material (src/art.ts lists them in the same order),
- assets/audio/*.wav: jump, coin, crash and start sounds,
- assets/fonts/ui.ttf: Noto Sans SC subset to Latin-1 and the game's
  strings (needs fontTools and the project template's font; skipped with a
  note when either is missing, since the committed subset is already there).

Standard library only except the font subset; deterministic.
"""
import os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_glb, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")

INK = "#14161b"


def sky():
    """Flat bands from night to dusk: smooth gradients band under ETC1S, flat colours stay clean."""
    c = Canvas(16, 256)
    bands = ["#0d1129", "#141634", "#1c1a40", "#261f4c", "#322457", "#3f2a5f"]
    for k, col in enumerate(bands):
        c.rect(0, k * 256 / len(bands), 16, 256 / len(bands) + 1, col)
    return c


def moon():
    c = Canvas(128, 128)
    c.circle(64, 64, 58, "#fff3c4")
    c.circle(46, 50, 10, "#ead9a2"); c.circle(80, 78, 14, "#ead9a2"); c.circle(78, 40, 6, "#ead9a2")
    return c


def cloud():
    c = Canvas(256, 96)
    for x, y, r in ((60, 60, 30), (100, 44, 38), (150, 50, 34), (195, 62, 26), (128, 68, 30)):
        c.circle(x, y, r, "#3b3566")
    c.rect(34, 60, 186, 28, "#3b3566", radius=14)
    return c


def building(variant):
    c = Canvas(128, 256)
    body = ["#1d1838", "#221c42", "#1a1533"][variant]
    if variant == 0:
        c.rect(14, 40, 100, 216, body)
        c.rect(54, 14, 20, 30, body); c.rect(62, 0, 4, 16, body)
    elif variant == 1:
        c.rect(6, 80, 116, 176, body)
        c.poly([(6, 80), (64, 44), (122, 80)], body)
    else:
        c.rect(24, 20, 80, 236, body)
        c.rect(10, 120, 108, 136, body)
    lit = ["#f7c76b", "#ffd98a", "#7fd1e8"]
    k = variant * 7 + 3
    for y in range(60 if variant != 2 else 36, 250, 22):
        for x in range(22 if variant != 2 else 34, 106 if variant != 2 else 96, 18):
            k = (k * 1103515245 + 12345) & 0x7FFFFFFF
            if k % 5 < 2:
                c.rect(x, y, 9, 12, lit[k % 3], radius=1)
    return c


def ground():
    c = Canvas(128, 96)
    c.rect(0, 0, 128, 96, "#2b2540")
    c.rect(0, 0, 128, 14, "#5ec4b6")
    c.rect(0, 14, 128, 5, "#3a8f86")
    for row, y in enumerate(range(26, 96, 18)):
        off = 0 if row % 2 == 0 else 16
        for x in range(-16 + off, 128, 32):
            c.rect(x + 2, y, 28, 14, "#352e4f", radius=2)
    return c


def crate():
    c = Canvas(128, 128)
    c.rect(6, 6, 116, 116, "#7a4a2a", radius=8)
    c.rect(14, 14, 100, 100, "#b8743f", radius=4)
    c.line(22, 22, 106, 106, 14, "#7a4a2a"); c.line(106, 22, 22, 106, 14, "#7a4a2a")
    c.rect(14, 14, 100, 10, "#d08f55")
    return c


def fence():
    c = Canvas(96, 192)
    c.rect(10, 10, 14, 182, "#6b6f80"); c.rect(72, 10, 14, 182, "#6b6f80")
    for i, y in enumerate((20, 80, 140)):
        c.rect(0, y, 96, 36, "#f2f2f2", radius=4)
        for s in range(-2, 6):
            x0 = s * 24 + (i % 2) * 12
            c.poly([(x0, y + 36), (x0 + 12, y + 36), (x0 + 30, y), (x0 + 18, y)], "#e8453c")
        c.rect(0, y, 96, 36, "#000000", alpha=0.0)
    c.circle(48, 10, 9, "#ffb020")
    # trim the stripes back to the boards
    return c


def coin():
    c = Canvas(96, 96)
    c.circle(48, 48, 44, "#b8860b")
    c.circle(48, 48, 38, "#ffd23f")
    star = []
    import math
    for i in range(10):
        r = 22 if i % 2 == 0 else 9
        a = -math.pi / 2 + i * math.pi / 5
        star.append((48 + r * math.cos(a), 49 + r * math.sin(a)))
    c.poly(star, "#fff3b0")
    c.ellipse(34, 30, 9, 5, "#fff8d6", alpha=0.8)
    return c


def courier(pose):
    """A courier with a teal jacket, an orange scarf and a mail bag."""
    c = Canvas(128, 160)
    skin, jacket, dark, scarf, helmet, shoe = "#f0c39a", "#2fa59a", "#1d6f68", "#ff7a3d", "#f4f4f4", "#2a2433"
    legs = {
        "run_a": [((60, 104), (44, 128), (34, 150)), ((68, 104), (84, 124), (100, 136))],
        "run_b": [((60, 104), (70, 130), (64, 154)), ((68, 104), (56, 126), (40, 132))],
        "jump": [((60, 104), (48, 120), (60, 136)), ((68, 104), (86, 116), (96, 132))],
        "crash": [((60, 104), (40, 116), (24, 120)), ((68, 104), (92, 112), (110, 108))],
        "idle": [((60, 104), (58, 130), (56, 154)), ((68, 104), (72, 130), (74, 154))],
    }[pose]
    arms = {
        "run_a": [((58, 66), (80, 82), (96, 74)), ((70, 66), (50, 86), (40, 100))],
        "run_b": [((58, 66), (44, 84), (52, 100)), ((70, 66), (88, 78), (100, 66))],
        "jump": [((58, 66), (40, 52), (34, 34)), ((70, 66), (92, 56), (104, 40))],
        "crash": [((58, 66), (36, 60), (20, 70)), ((70, 66), (96, 70), (112, 80))],
        "idle": [((58, 66), (50, 88), (52, 104)), ((70, 66), (78, 88), (78, 104))],
    }[pose]
    # back leg and arm darker
    (a0, a1, a2), (b0, b1, b2) = legs
    c.line(*a0, *a1, 13, dark); c.line(*a1, *a2, 12, dark); c.ellipse(a2[0] + 4, a2[1] + 3, 10, 6, shoe)
    (h0, h1, h2), (k0, k1, k2) = arms
    c.line(*h0, *h1, 10, dark); c.line(*h1, *h2, 9, dark)
    # scarf trailing back
    c.poly([(58, 52), (30, 46), (14, 58), (30, 58), (56, 62)], scarf)
    # bag
    c.rect(34, 70, 26, 24, "#c98b3f", radius=4); c.line(44, 60, 74, 84, 4, "#8a5a24")
    # torso
    c.rect(50, 52, 34, 56, jacket, radius=12)
    c.line(*b0, *b1, 13, jacket if pose != "crash" else dark); c.line(*b1, *b2, 12, "#3b3550"); c.ellipse(b2[0] + 4, b2[1] + 3, 10, 6, shoe)
    c.line(*k0, *k1, 10, jacket); c.line(*k1, *k2, 9, jacket); c.circle(*k2, 6, skin)
    # head
    c.circle(70, 36, 20, skin)
    c.poly([(48, 34), (52, 16), (70, 10), (90, 18), (92, 32)], helmet)
    c.rect(78, 28, 14, 6, "#2a2433", radius=3)
    c.rect(52, 52, 26, 8, scarf, radius=4)
    if pose == "crash":
        c.line(84, 30, 92, 38, 3, INK); c.line(92, 30, 84, 38, 3, INK)
    return c


SPRITES = [
    ("sky", sky, False),
    ("moon", moon, True),
    ("cloud", cloud, True),
    ("building_a", lambda: building(0), True),
    ("building_b", lambda: building(1), True),
    ("building_c", lambda: building(2), True),
    ("ground", ground, False),
    ("crate", crate, True),
    ("fence", fence, True),
    ("coin", coin, True),
    ("runner_idle", lambda: courier("idle"), True),
    ("runner_run_a", lambda: courier("run_a"), True),
    ("runner_run_b", lambda: courier("run_b"), True),
    ("runner_jump", lambda: courier("jump"), True),
    ("runner_crash", lambda: courier("crash"), True),
]


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
