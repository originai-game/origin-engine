#!/usr/bin/env python3
"""Regenerates the UI kit demo's self-made assets (CC0), byte for byte:

    python3 templates/ui-kit/tools/make-assets.py templates/ui-kit/assets

- textures/icons.png: 32 white glyphs (alpha is coverage) in an 8x4 grid of
  64 px cells, drawn at 4x and box-filtered down; the names, in reading
  order, are ICONS below (src/main.ts passes them to og.ui.configure).
- textures/backdrop.png: a 640x360 dusk landscape (sky bands, two hill
  ranges, a sun) behind the HUD screens, so HUD capsules are seen over art.

Pillow only; no randomness, so the output is reproducible.
"""
import math, os, sys
from PIL import Image, ImageDraw

ICONS = ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
         "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
         "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
         "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person"]
S = 256  # drawing size of one cell
W = 255


def poly(d, pts, s=S):
    d.polygon([(x * s, y * s) for x, y in pts], fill=W)


def circle(d, cx, cy, r, fill=W):
    d.ellipse([(cx - r) * S, (cy - r) * S, (cx + r) * S, (cy + r) * S], fill=fill)


def rect(d, x0, y0, x1, y1, fill=W, r=0.0):
    d.rounded_rectangle([x0 * S, y0 * S, x1 * S, y1 * S], radius=r * S, fill=fill)


def line(d, pts, w):
    d.line([(x * S, y * S) for x, y in pts], fill=W, width=int(w * S), joint="curve")
    for x, y in (pts[0], pts[-1]):
        circle(d, x, y, w / 2)


def star_pts(cx, cy, r0, r1, n=5, rot=-math.pi / 2):
    pts = []
    for k in range(2 * n):
        r = r0 if k % 2 == 0 else r1
        a = rot + k * math.pi / n
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def draw(name, d):
    if name == "heart":
        circle(d, 0.35, 0.38, 0.17); circle(d, 0.65, 0.38, 0.17)
        poly(d, [(0.19, 0.44), (0.81, 0.44), (0.5, 0.82)])
    elif name == "star":
        poly(d, star_pts(0.5, 0.53, 0.38, 0.16))
    elif name == "coin":
        circle(d, 0.5, 0.5, 0.36); circle(d, 0.5, 0.5, 0.26, 0); rect(d, 0.45, 0.33, 0.55, 0.67, r=0.03)
    elif name == "lock":
        rect(d, 0.25, 0.45, 0.75, 0.82, r=0.06)
        d.arc([0.32 * S, 0.18 * S, 0.68 * S, 0.6 * S], 180, 360, fill=W, width=int(0.08 * S))
        rect(d, 0.32, 0.38, 0.4, 0.47); rect(d, 0.6, 0.38, 0.68, 0.47)
    elif name == "pause":
        rect(d, 0.3, 0.24, 0.43, 0.76, r=0.03); rect(d, 0.57, 0.24, 0.7, 0.76, r=0.03)
    elif name == "gear":
        for k in range(8):
            a = k * math.pi / 4
            cx, cy = 0.5 + 0.3 * math.cos(a), 0.5 + 0.3 * math.sin(a)
            circle(d, cx, cy, 0.08)
        circle(d, 0.5, 0.5, 0.3); circle(d, 0.5, 0.5, 0.12, 0)
    elif name == "sword":
        poly(d, [(0.72, 0.18), (0.82, 0.18), (0.82, 0.28), (0.42, 0.68), (0.32, 0.58)])
        line(d, [(0.25, 0.55), (0.45, 0.75)], 0.07); line(d, [(0.33, 0.67), (0.2, 0.8)], 0.07)
    elif name == "shield":
        poly(d, [(0.5, 0.15), (0.8, 0.26), (0.76, 0.56), (0.5, 0.85), (0.24, 0.56), (0.2, 0.26)])
    elif name == "potion":
        circle(d, 0.5, 0.62, 0.24); rect(d, 0.43, 0.2, 0.57, 0.45); rect(d, 0.38, 0.16, 0.62, 0.24, r=0.03)
    elif name == "gem":
        poly(d, [(0.3, 0.25), (0.7, 0.25), (0.85, 0.42), (0.5, 0.82), (0.15, 0.42)])
    elif name == "key":
        circle(d, 0.32, 0.5, 0.16); circle(d, 0.32, 0.5, 0.07, 0)
        rect(d, 0.44, 0.46, 0.84, 0.55); rect(d, 0.72, 0.55, 0.78, 0.66); rect(d, 0.62, 0.55, 0.68, 0.62)
    elif name == "map":
        poly(d, [(0.15, 0.25), (0.38, 0.18), (0.62, 0.27), (0.85, 0.2), (0.85, 0.75), (0.62, 0.82), (0.38, 0.73), (0.15, 0.8)])
        line(d, [(0.38, 0.22), (0.38, 0.72)], 0.03); line(d, [(0.62, 0.3), (0.62, 0.8)], 0.03)
    elif name == "bolt":
        poly(d, [(0.56, 0.12), (0.26, 0.56), (0.48, 0.56), (0.42, 0.88), (0.74, 0.42), (0.52, 0.42)])
    elif name == "clock":
        circle(d, 0.5, 0.5, 0.36); circle(d, 0.5, 0.5, 0.28, 0)
        line(d, [(0.5, 0.5), (0.5, 0.3)], 0.06); line(d, [(0.5, 0.5), (0.64, 0.58)], 0.06)
    elif name == "trophy":
        rect(d, 0.3, 0.18, 0.7, 0.24); poly(d, [(0.3, 0.2), (0.7, 0.2), (0.64, 0.52), (0.36, 0.52)])
        circle(d, 0.5, 0.48, 0.14); rect(d, 0.45, 0.55, 0.55, 0.7); rect(d, 0.32, 0.7, 0.68, 0.8, r=0.02)
        d.arc([0.16 * S, 0.22 * S, 0.38 * S, 0.48 * S], 90, 270, fill=W, width=int(0.05 * S))
        d.arc([0.62 * S, 0.22 * S, 0.84 * S, 0.48 * S], 270, 90, fill=W, width=int(0.05 * S))
    elif name == "home":
        poly(d, [(0.5, 0.16), (0.86, 0.48), (0.14, 0.48)]); rect(d, 0.24, 0.46, 0.76, 0.84)
        rect(d, 0.43, 0.6, 0.57, 0.84, fill=0)
    elif name == "plus":
        rect(d, 0.44, 0.2, 0.56, 0.8, r=0.03); rect(d, 0.2, 0.44, 0.8, 0.56, r=0.03)
    elif name == "close":
        line(d, [(0.26, 0.26), (0.74, 0.74)], 0.11); line(d, [(0.74, 0.26), (0.26, 0.74)], 0.11)
    elif name == "check":
        line(d, [(0.2, 0.52), (0.42, 0.72), (0.8, 0.3)], 0.11)
    elif name == "play":
        poly(d, [(0.3, 0.2), (0.8, 0.5), (0.3, 0.8)])
    elif name == "back":
        line(d, [(0.62, 0.2), (0.32, 0.5), (0.62, 0.8)], 0.11)
    elif name == "bag":
        rect(d, 0.2, 0.36, 0.8, 0.84, r=0.08)
        d.arc([0.33 * S, 0.16 * S, 0.67 * S, 0.52 * S], 180, 360, fill=W, width=int(0.07 * S))
    elif name == "bomb":
        circle(d, 0.45, 0.58, 0.28); rect(d, 0.52, 0.22, 0.66, 0.36, r=0.02); line(d, [(0.6, 0.24), (0.72, 0.12)], 0.04)
    elif name == "crown":
        poly(d, [(0.16, 0.3), (0.34, 0.5), (0.5, 0.24), (0.66, 0.5), (0.84, 0.3), (0.78, 0.74), (0.22, 0.74)])
    elif name == "flag":
        rect(d, 0.24, 0.14, 0.31, 0.86); poly(d, [(0.31, 0.18), (0.8, 0.32), (0.31, 0.5)])
    elif name == "apple":
        circle(d, 0.4, 0.58, 0.23); circle(d, 0.6, 0.58, 0.23); line(d, [(0.5, 0.36), (0.56, 0.18)], 0.05)
    elif name == "leaf":
        d.ellipse([0.2 * S, 0.3 * S, 0.8 * S, 0.7 * S], fill=W); line(d, [(0.2, 0.75), (0.5, 0.5)], 0.05)
    elif name == "fire":
        poly(d, [(0.5, 0.12), (0.72, 0.42), (0.76, 0.62), (0.5, 0.86), (0.24, 0.62), (0.3, 0.4), (0.42, 0.5)])
        circle(d, 0.5, 0.64, 0.22)
    elif name == "drop":
        poly(d, [(0.5, 0.12), (0.74, 0.5), (0.26, 0.5)]); circle(d, 0.5, 0.6, 0.24)
    elif name == "skull":
        circle(d, 0.5, 0.44, 0.3); rect(d, 0.34, 0.6, 0.66, 0.84, r=0.04)
        circle(d, 0.39, 0.46, 0.08, 0); circle(d, 0.61, 0.46, 0.08, 0)
    elif name == "music":
        circle(d, 0.32, 0.72, 0.12); circle(d, 0.7, 0.64, 0.12)
        rect(d, 0.38, 0.22, 0.44, 0.72); rect(d, 0.76, 0.16, 0.82, 0.64); poly(d, [(0.38, 0.22), (0.82, 0.12), (0.82, 0.26), (0.38, 0.36)])
    elif name == "person":
        circle(d, 0.5, 0.32, 0.15); d.pieslice([0.22 * S, 0.52 * S, 0.78 * S, 1.1 * S], 180, 360, fill=W)
    else:
        raise SystemExit(f"no drawing for {name}")


def glyph_mask(name, cell=64):
    """One icon's coverage (L, cell x cell): drawn at S and box-filtered down."""
    big = Image.new("L", (S, S), 0)
    draw(name, ImageDraw.Draw(big))
    return big.resize((cell, cell), Image.BOX)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "templates/ui-kit/assets"
    os.makedirs(os.path.join(out, "textures"), exist_ok=True)
    cell, cols, rows = 64, 8, 4
    atlas = Image.new("LA", (cell * cols, cell * rows), (255, 0))
    for k, name in enumerate(ICONS):
        big = Image.new("L", (S, S), 0)
        draw(name, ImageDraw.Draw(big))
        small = big.resize((cell, cell), Image.BOX)
        glyph = Image.merge("LA", (Image.new("L", (cell, cell), 255), small))
        atlas.paste(glyph, ((k % cols) * cell, (k // cols) * cell))
    atlas.convert("RGBA").save(os.path.join(out, "textures", "icons.png"), optimize=True)

    # The backdrop: sky bands from deep blue to dusk orange, a sun, two hill ranges.
    w, h = 640, 360
    sky = Image.new("RGB", (w, h))
    px = sky.load()
    top, mid, low = (18, 24, 54), (68, 52, 104), (214, 120, 84)
    for y in range(h):
        t = y / (h * 0.62)
        a, b, u = (top, mid, t / 0.6) if t < 0.6 else (mid, low, min(1.0, (t - 0.6) / 0.4))
        c = tuple(round(a[i] + (b[i] - a[i]) * u) for i in range(3))
        for x in range(w):
            px[x, y] = c
    d = ImageDraw.Draw(sky)
    d.ellipse([430, 150, 510, 230], fill=(250, 196, 120))
    far = [(0, 250)] + [(x, 236 + 18 * math.sin(x / 70) + 10 * math.sin(x / 23)) for x in range(0, w + 1, 8)] + [(w, h), (0, h)]
    d.polygon(far, fill=(54, 40, 78))
    near = [(0, 290)] + [(x, 284 + 22 * math.sin(x / 95 + 1.3) + 6 * math.sin(x / 17)) for x in range(0, w + 1, 8)] + [(w, h), (0, h)]
    d.polygon(near, fill=(26, 22, 44))
    sky.save(os.path.join(out, "textures", "backdrop.png"), optimize=True)
    print("wrote", os.path.join(out, "textures"))


if __name__ == "__main__":
    main()
