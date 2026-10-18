#!/usr/bin/env python3
"""Regenerates the effects showcase's self-made assets (CC0), byte for byte:

    python3 templates/fx-showcase/tools/make-assets.py templates/fx-showcase/assets

- textures/spark.png: a soft round dot (particles, ribbons).
- textures/flame.png: a 4x2 sprite sheet of a flame flickering (fire).
- textures/smoke.png: a soft noisy puff (smoke).
- textures/decals.png: a 2x2 decal atlas: a scorch ring, a glowing rune, an
  arrow, a splat (RGBA; alpha is coverage).
- textures/beam.png: bright stripes for the scrolling beam.
- textures/toon.png: a 4-band toon ramp (Material.gradientMap).
- textures/crate.png, textures/tile.png: 2D sprites for the lit 2D station.
- luts/teal-orange.cube: a 17^3 grading table, shadows to teal, highlights
  to orange.

Pillow only; no randomness beyond a fixed hash, so the output is reproducible.
"""
import math, os, sys
from PIL import Image, ImageDraw, ImageFilter

out = sys.argv[1] if len(sys.argv) > 1 else "templates/fx-showcase/assets"
for d in ("textures", "luts"):
    os.makedirs(os.path.join(out, d), exist_ok=True)


def save(img, name):
    img.save(os.path.join(out, "textures", name), optimize=False)


def hash01(x, y, s=0):
    h = (x * 374761393 + y * 668265263 + s * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h ^ (h >> 16)) / 0xFFFFFFFF


def radial(size, power=2.0, color=(255, 255, 255)):
    img = Image.new("RGBA", (size, size))
    px = img.load()
    c = (size - 1) / 2
    for y in range(size):
        for x in range(size):
            d = min(1.0, math.hypot(x - c, y - c) / c)
            a = int(255 * (1 - d) ** power)
            px[x, y] = (*color, a)
    return img


save(radial(64, 2.2), "spark.png")

# Flame sheet: 8 frames of a teardrop whose tip sways.
sheet = Image.new("RGBA", (256, 128))
for f in range(8):
    cell = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(cell)
    sway = 6 * math.sin(f / 8 * 2 * math.pi)
    for k in range(10, 0, -1):
        r = 3 + k * 2.2
        a = int(40 + 21 * (10 - k))
        cx, cy = 32 + sway * (1 - k / 10), 44 - (10 - k) * 2.6
        d.ellipse([cx - r, cy - r * 1.3, cx + r, cy + r], fill=(255, 255, 255, a))
    cell = cell.filter(ImageFilter.GaussianBlur(2))
    sheet.paste(cell, ((f % 4) * 64, (f // 4) * 64))
save(sheet, "flame.png")

smoke = Image.new("RGBA", (64, 64))
px = smoke.load()
for y in range(64):
    for x in range(64):
        d = min(1.0, math.hypot(x - 31.5, y - 31.5) / 31.5)
        n = 0.6 + 0.4 * hash01(x // 4, y // 4, 3)
        px[x, y] = (230, 230, 230, int(200 * (1 - d) ** 1.5 * n))
save(smoke.filter(ImageFilter.GaussianBlur(1.5)), "smoke.png")

atlas = Image.new("RGBA", (256, 256))
d = ImageDraw.Draw(atlas)
# Scorch ring (top left).
for k in range(40):
    r = 60 - k
    a = int(255 * min(1, k / 20) * (0.6 if k > 30 else 1))
    d.ellipse([64 - r, 64 - r, 64 + r, 64 + r], outline=(30, 20, 15, a), width=2)
d.ellipse([34, 34, 94, 94], fill=(20, 12, 8, 230))
# Rune (top right): a star in a circle.
d.ellipse([140, 12, 244, 116], outline=(255, 255, 255, 255), width=8)
pts = [(192 + 44 * math.cos(-math.pi / 2 + k * 4 * math.pi / 5), 64 + 44 * math.sin(-math.pi / 2 + k * 4 * math.pi / 5)) for k in range(5)]
d.line(pts + [pts[0]], fill=(255, 255, 255, 255), width=7)
# Arrow (bottom left).
d.polygon([(20, 192), (70, 150), (70, 175), (110, 175), (110, 209), (70, 209), (70, 234)], fill=(250, 220, 40, 255))
# Splat (bottom right).
for k in range(14):
    a = k * 2 * math.pi / 14
    r = 20 + 30 * hash01(k, 1, 7)
    cx, cy = 192 + r * math.cos(a), 192 + r * math.sin(a)
    rr = 8 + 6 * hash01(k, 2, 7)
    d.ellipse([cx - rr, cy - rr, cx + rr, cy + rr], fill=(140, 10, 20, 255))
d.ellipse([162, 162, 222, 222], fill=(140, 10, 20, 255))
save(atlas, "decals.png")

beam = Image.new("RGBA", (64, 16))
px = beam.load()
for y in range(16):
    for x in range(64):
        v = 0.5 + 0.5 * math.cos(x / 64 * 2 * math.pi * 2)
        e = 1 - abs(y - 7.5) / 8
        px[x, y] = (255, 255, 255, int(255 * e * (0.4 + 0.6 * v)))
save(beam, "beam.png")

ramp = Image.new("RGBA", (4, 1))
for i, v in enumerate((40, 110, 190, 255)):
    ramp.putpixel((i, 0), (v, v, v, 255))
save(ramp, "toon.png")

crate = Image.new("RGBA", (64, 64), (150, 100, 55, 255))
d = ImageDraw.Draw(crate)
d.rectangle([0, 0, 63, 63], outline=(80, 50, 25, 255), width=5)
d.line([(4, 4), (59, 59)], fill=(95, 60, 30, 255), width=6)
d.line([(4, 59), (59, 4)], fill=(95, 60, 30, 255), width=6)
save(crate, "crate.png")

tile = Image.new("RGBA", (64, 64))
px = tile.load()
for y in range(64):
    for x in range(64):
        n = hash01(x // 8, y // 8, 11)
        g = int(120 + 50 * n)
        edge = x % 32 < 2 or y % 32 < 2
        px[x, y] = (g - 30, g - 20, g, 255) if not edge else (60, 60, 70, 255)
save(tile, "tile.png")

# Teal-orange: shadows lean teal, highlights orange, saturation kept.
n = 17
lines = ['TITLE "teal-orange"', f"LUT_3D_SIZE {n}", "DOMAIN_MIN 0 0 0", "DOMAIN_MAX 1 1 1"]
for b in range(n):
    for g in range(n):
        for r in range(n):
            R, G, B = r / (n - 1), g / (n - 1), b / (n - 1)
            l = 0.2126 * R + 0.7152 * G + 0.0722 * B
            t = (1 - l) ** 2 * 0.18
            o = l ** 2 * 0.14
            R2 = min(1, max(0, R - t * 0.6 + o))
            G2 = min(1, max(0, G + t * 0.25 + o * 0.35))
            B2 = min(1, max(0, B + t - o * 0.7))
            lines.append(f"{R2:.6f} {G2:.6f} {B2:.6f}")
with open(os.path.join(out, "luts", "teal-orange.cube"), "w") as fh:
    fh.write("\n".join(lines) + "\n")
print("wrote", out)
