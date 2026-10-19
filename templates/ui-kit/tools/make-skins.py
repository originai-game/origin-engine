#!/usr/bin/env python3
"""Regenerates the UI kit's six style packs (CC0 art), byte for byte:

    python3 templates/ui-kit/tools/make-skins.py templates/ui-kit/assets

For each of pixel, cartoon, fantasy, scifi, paper and neon it writes
assets/skins/<name>/:

- atlas.png: the nine-slice frames (panel, card, button and its hover,
  pressed and disabled states, field, slot, tooltip, capsule, progress track
  and fill), a soft blurred rectangle for shadows and glows, the ornaments
  (corner, divider, heading flourish) and a tile, each with a 4 px gutter
  whose texels repeat the region's edge (so filtering never bleeds a
  neighbour in). Frames meant to take the palette are drawn in white and
  greys (og-ui multiplies them by the element's colour); "authored" ones
  carry their own colours.
- icons.png: the kit's 32 icons (tools/make-assets.py draws them) restyled
  per pack: pixelated, thickened, outlined, roughened or neon-lined; white,
  alpha is coverage.
- skin.ogt: the skin file (og-ui's `parse_skin` grammar) naming the atlas,
  the slices, the palette, the radii, the type ramp, the fonts, the motion.

The fonts next to them are SIL OFL 1.1 files copied (or subset to Latin-1
with fontTools) by hand; LICENSES.md and OFL.txt in each directory say which.
numpy and Pillow only; no randomness beyond fixed integer hashes, so the
output is reproducible.
"""
import importlib.util
import math
import os
import sys

import numpy as np
from PIL import Image, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("make_assets", os.path.join(HERE, "make-assets.py"))
make_assets = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(make_assets)
ICONS = make_assets.ICONS

ATLAS_W, ATLAS_H = 384, 256
# Region name -> (x, y, w, h) in the atlas.
LAYOUT = {
    "panel": (4, 4, 96, 96),
    "card": (104, 4, 64, 64),
    "button": (172, 4, 48, 48),
    "buttonHover": (224, 4, 48, 48),
    "buttonPressed": (276, 4, 48, 48),
    "buttonDisabled": (328, 4, 48, 48),
    "field": (4, 104, 48, 48),
    "slot": (56, 104, 48, 48),
    "tooltip": (108, 104, 32, 32),
    "capsule": (144, 104, 32, 32),
    "track": (180, 104, 32, 16),
    "fill": (216, 104, 32, 16),
    "soft": (252, 104, 64, 64),
    "corner": (320, 104, 32, 32),
    "divider": (4, 172, 128, 24),
    "flourish": (136, 172, 64, 24),
    "tile": (204, 172, 64, 64),
}
SS = 4  # supersampling of smooth art


# ── geometry ────────────────────────────────────────────────────────────────

def coords(w, h, ss):
    """Sample centres in output pixels, (h*ss, w*ss) arrays."""
    ys, xs = np.mgrid[0:h * ss, 0:w * ss].astype(np.float64)
    return (xs + 0.5) / ss, (ys + 0.5) / ss


def sd_box(px, py, x0, y0, x1, y1, r=0.0):
    """Signed distance to a rounded box (negative inside)."""
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    hx, hy = (x1 - x0) / 2, (y1 - y0) / 2
    r = min(r, hx, hy)
    qx = np.abs(px - cx) - hx + r
    qy = np.abs(py - cy) - hy + r
    out = np.hypot(np.maximum(qx, 0), np.maximum(qy, 0))
    return out + np.minimum(np.maximum(qx, qy), 0) - r


def sd_chamfer(px, py, x0, y0, x1, y1, c, corners=(1, 1, 1, 1)):
    """A box with its corners cut at 45 degrees by `c` (tl, tr, br, bl)."""
    d = sd_box(px, py, x0, y0, x1, y1)
    for k, (sx, sy) in enumerate([(-1, -1), (1, -1), (1, 1), (-1, 1)]):
        if not corners[k] or c <= 0:
            continue
        ex = x1 if sx > 0 else x0
        ey = y1 if sy > 0 else y0
        cut = (sx * (px - ex) + sy * (py - ey) + c) / math.sqrt(2)
        d = np.maximum(d, cut)
    return d


def shrink(a, ss):
    """Box-filters a supersampled (H*ss, W*ss[, C]) array down."""
    h, w = a.shape[0] // ss, a.shape[1] // ss
    return a.reshape(h, ss, w, ss, *a.shape[2:]).mean(axis=(1, 3))


class Canvas:
    """Straight-alpha RGBA float canvas, painted over (source-over)."""

    def __init__(self, w, h, ss=SS):
        self.w, self.h, self.ss = w, h, ss
        self.px, self.py = coords(w, h, ss)
        self.rgba = np.zeros((h * ss, w * ss, 4))

    def paint(self, mask, rgb, a=1.0):
        """mask: coverage 0-1 (or bool) at the supersampled size."""
        m = np.clip(np.asarray(mask, dtype=np.float64), 0, 1) * a
        if np.ndim(rgb) == 1 or isinstance(rgb, tuple):
            rgb = np.broadcast_to(np.asarray(rgb, dtype=np.float64), self.rgba.shape[:2] + (3,))
        src_a = m
        dst = self.rgba
        out_a = src_a + dst[..., 3] * (1 - src_a)
        safe = np.where(out_a > 0, out_a, 1)
        for c in range(3):
            dst[..., c] = (rgb[..., c] * src_a + dst[..., c] * dst[..., 3] * (1 - src_a)) / safe
        dst[..., 3] = out_a

    def inside(self, d):
        return (d < 0).astype(np.float64)

    def image(self):
        a = shrink(self.rgba, self.ss) if self.ss > 1 else self.rgba
        # Premultiply to average, then unpremultiply.
        if self.ss > 1:
            pm = self.rgba.copy()
            pm[..., :3] *= pm[..., 3:4]
            pm = shrink(pm, self.ss)
            alpha = pm[..., 3:4]
            rgb = np.where(alpha > 0, pm[..., :3] / np.where(alpha > 0, alpha, 1), 0)
            a = np.concatenate([rgb, alpha], axis=2)
        return np.clip(a, 0, 1)


def grey(v):
    return (v, v, v)


def hexc(s):
    s = s.lstrip("#")
    return tuple(int(s[i:i + 2], 16) / 255 for i in (0, 2, 4))


def vgrad(cv, top, bottom):
    """A vertical ramp of greys or colours over the canvas."""
    t = np.clip(cv.py / cv.h, 0, 1)[..., None]
    a, b = np.asarray(top, dtype=np.float64), np.asarray(bottom, dtype=np.float64)
    return a + (b - a) * t


def hash01(x, y, seed=0):
    h = (x * 0x9E3779B1 + y * 0x85EBCA6B + seed * 0xC2B2AE35) & 0xFFFFFFFF
    h ^= h >> 15
    h = (h * 0x2C1B3C6D) & 0xFFFFFFFF
    h ^= h >> 12
    return (h & 0xFFFF) / 65536.0


def noise(w, h, seed, scale=1):
    xs, ys = np.meshgrid(np.arange(w) // scale, np.arange(h) // scale)
    v = np.vectorize(lambda x, y: hash01(int(x), int(y), seed))(xs, ys)
    return v


# ── frame painters ──────────────────────────────────────────────────────────

def frame(w, h, r=8, outline=0.0, outline_c=grey(0.2), fill=(grey(1.0), grey(0.86)),
          highlight=0.0, highlight_c=grey(1.0), inner=0.0, inner_c=grey(0.7), chamfer=None,
          fill_a=1.0, shade=0.0, ss=SS):
    """A rounded (or chamfered) frame: an outline ring, a gradient fill, a
    top highlight band, an inner ring and a bottom shade band."""
    cv = Canvas(w, h, ss)

    def shape(inset, rr):
        if chamfer is not None:
            return sd_chamfer(cv.px, cv.py, inset, inset, w - inset, h - inset, max(0.0, chamfer - inset * 0.4))
        return sd_box(cv.px, cv.py, inset, inset, w - inset, h - inset, max(0.0, rr))

    if outline > 0:
        cv.paint(cv.inside(shape(0.5, r)), outline_c)
    body = shape(0.5 + outline, r - outline)
    cv.paint(cv.inside(body), vgrad(cv, fill[0], fill[1]), fill_a)
    if shade > 0:
        band = cv.inside(body) * (cv.py > h - outline - 0.5 - shade)
        cv.paint(band, grey(0.0), 0.18)
    if highlight > 0:
        band = cv.inside(body) * (cv.py < outline + 0.5 + highlight)
        cv.paint(band, highlight_c, 0.55)
    if inner > 0:
        ring = cv.inside(shape(0.5 + outline + inner, r - outline - inner))
        edge = cv.inside(body) * (1 - ring)
        cv.paint(edge, inner_c, 1.0)
    return cv.image()


def pixel_frame(w, h, cut=2, outline=2, light=1.0, dark=0.62, body=0.86, bevel=2, outline_c=grey(0.08),
                sunk=False):
    """A crisp 1x pixel frame: a dark outline with cut corners, a light top
    and left bevel, a dark bottom and right bevel (swapped when sunk)."""
    img = np.zeros((h, w, 4))
    ys, xs = np.mgrid[0:h, 0:w]
    corner = ((xs < cut) | (xs >= w - cut)) & ((ys < cut) | (ys >= h - cut))
    img[..., :3] = outline_c
    img[..., 3] = 1.0
    inner = (xs >= outline) & (xs < w - outline) & (ys >= outline) & (ys < h - outline)
    img[inner, :3] = body
    top = inner & ((ys < outline + bevel) | (xs < outline + bevel))
    bottom = inner & ((ys >= h - outline - bevel) | (xs >= w - outline - bevel))
    hi, lo = (dark, light) if sunk else (light, dark)
    img[top, :3] = hi
    img[bottom & ~top, :3] = lo
    # The light and dark bevels meet on the diagonal at the two odd corners.
    img[corner] = 0.0
    return img


def soft_rect(w, h, margin, ss=1):
    """A white rectangle blurred into its margin (alpha only)."""
    px, py = coords(w, h, ss)
    d = sd_box(px, py, margin, margin, w - margin, h - margin, 4)
    a = np.where(d <= 0, 1.0, np.exp(-(d / (margin * 0.45)) ** 2))
    img = np.ones((h * ss, w * ss, 4))
    img[..., 3] = a
    return img


# ── ornaments ───────────────────────────────────────────────────────────────

def ornament(w, h, draw, ss=SS):
    cv = Canvas(w, h, ss)
    draw(cv)
    return cv.image()


def bars(cv, segs, width, rgb=grey(1.0), a=1.0):
    """Line segments of a width (capsules) in output px."""
    for (x0, y0, x1, y1) in segs:
        dx, dy = x1 - x0, y1 - y0
        L2 = dx * dx + dy * dy
        t = np.clip(((cv.px - x0) * dx + (cv.py - y0) * dy) / (L2 if L2 > 0 else 1), 0, 1)
        d = np.hypot(cv.px - (x0 + t * dx), cv.py - (y0 + t * dy)) - width / 2
        cv.paint(cv.inside(d), rgb, a)


def disc(cv, x, y, r, rgb=grey(1.0), a=1.0):
    cv.paint(cv.inside(np.hypot(cv.px - x, cv.py - y) - r), rgb, a)


def diamond(cv, x, y, r, rgb=grey(1.0), a=1.0):
    cv.paint(cv.inside(np.abs(cv.px - x) + np.abs(cv.py - y) - r), rgb, a)


def glow_line(cv, segs, width, halo, rgb=grey(1.0)):
    for (x0, y0, x1, y1) in segs:
        dx, dy = x1 - x0, y1 - y0
        L2 = dx * dx + dy * dy
        t = np.clip(((cv.px - x0) * dx + (cv.py - y0) * dy) / (L2 if L2 > 0 else 1), 0, 1)
        d = np.hypot(cv.px - (x0 + t * dx), cv.py - (y0 + t * dy)) - width / 2
        cv.paint(np.where(d < 0, 1.0, np.exp(-(np.maximum(d, 0) / halo) ** 2) * 0.55), rgb)


# ── icons ───────────────────────────────────────────────────────────────────

def icon_sheet(style):
    cell, cols, rows = 64, 8, 4
    sheet = Image.new("LA", (cell * cols, cell * rows), (255, 0))
    for k, name in enumerate(ICONS):
        m = make_assets.glyph_mask(name, cell)
        if style == "pixel":
            small = make_assets.glyph_mask(name, 16)
            small = small.point(lambda v: 255 if v >= 110 else 0)
            m = small.resize((cell, cell), Image.NEAREST)
        elif style == "cartoon":
            m = m.filter(ImageFilter.MaxFilter(5)).filter(ImageFilter.GaussianBlur(0.8))
            m = m.point(lambda v: min(255, int(v * 1.6)))
        elif style in ("scifi", "neon"):
            inner = m.filter(ImageFilter.MinFilter(7 if style == "scifi" else 9))
            a = np.asarray(m, dtype=np.float64)
            b = np.asarray(inner, dtype=np.float64)
            rim = np.clip(a - b, 0, 255)
            fill = 0.28 if style == "scifi" else 0.42
            m = Image.fromarray(np.clip(rim + b * fill, 0, 255).astype(np.uint8), "L")
            if style == "neon":
                m = Image.fromarray(np.maximum(np.asarray(m), np.asarray(m.filter(ImageFilter.GaussianBlur(3))) * 0.9).astype(np.uint8), "L")
        elif style == "paper":
            a = np.asarray(m, dtype=np.float64) / 255
            ys, xs = np.mgrid[0:cell, 0:cell]
            j = np.vectorize(lambda x, y: hash01(int(x) // 2, int(y) // 2, k + 7))(xs, ys)
            a = np.clip((a - 0.5) * 3 + 0.5 + (j - 0.5) * 0.7, 0, 1)
            m = Image.fromarray((a * 255).astype(np.uint8), "L").filter(ImageFilter.GaussianBlur(0.5))
        elif style == "fantasy":
            # A raised glyph: the edge a little deeper than the face.
            inner = m.filter(ImageFilter.MinFilter(5))
            a = np.asarray(m, dtype=np.float64)
            b = np.asarray(inner, dtype=np.float64)
            m = Image.fromarray(np.clip(a * 0.85 + b * 0.15, 0, 255).astype(np.uint8), "L")
        glyph = Image.merge("LA", (Image.new("L", (cell, cell), 255), m))
        sheet.paste(glyph, ((k % cols) * cell, (k // cols) * cell))
    return sheet.convert("RGBA")


# ── the packs ───────────────────────────────────────────────────────────────

def pixel_pack():
    p = {}
    p["panel"] = pixel_frame(96, 96, cut=4, outline=4, bevel=4, light=0.98, dark=0.7, body=0.88)
    p["card"] = pixel_frame(64, 64, cut=2, outline=2, bevel=2, light=1.0, dark=0.72, body=0.9)
    p["button"] = pixel_frame(48, 48, cut=2, outline=2, bevel=4, light=1.0, dark=0.62, body=0.86)
    p["buttonHover"] = pixel_frame(48, 48, cut=2, outline=2, bevel=4, light=1.0, dark=0.7, body=0.96)
    p["buttonPressed"] = pixel_frame(48, 48, cut=2, outline=2, bevel=4, light=1.0, dark=0.62, body=0.8, sunk=True)
    p["buttonDisabled"] = pixel_frame(48, 48, cut=2, outline=2, bevel=2, light=0.8, dark=0.7, body=0.75, outline_c=grey(0.3))
    p["field"] = pixel_frame(48, 48, cut=0, outline=2, bevel=2, light=1.0, dark=0.7, body=0.94, sunk=True)
    p["slot"] = pixel_frame(48, 48, cut=0, outline=2, bevel=2, light=1.0, dark=0.66, body=0.9, sunk=True)
    p["tooltip"] = pixel_frame(32, 32, cut=2, outline=2, bevel=0, body=1.0)
    p["capsule"] = pixel_frame(32, 32, cut=2, outline=2, bevel=2, light=1.0, dark=0.7, body=0.9)
    p["track"] = pixel_frame(32, 16, cut=0, outline=2, bevel=2, light=0.7, dark=1.0, body=0.85)
    p["fill"] = pixel_frame(32, 16, cut=0, outline=2, bevel=2, light=1.0, dark=0.7, body=0.9)
    p["soft"] = soft_rect(64, 64, 20)

    def divider(cv):
        for x in range(8, 120, 8):
            cv.paint(cv.inside(sd_box(cv.px, cv.py, x, 10, x + 4, 14)), grey(1.0))
        cv.paint(cv.inside(sd_box(cv.px, cv.py, 58, 6, 70, 18)), grey(1.0))

    def flourish(cv):
        # A pixel arrow pointing right (at the heading), mirrored on its right.
        for k, (y0, y1) in enumerate([(4, 20), (6, 18), (8, 16), (10, 14)]):
            x = 40 + 4 * k
            cv.paint(cv.inside(sd_box(cv.px, cv.py, x, y0, x + 4, y1)), grey(1.0))
        cv.paint(cv.inside(sd_box(cv.px, cv.py, 24, 10, 36, 14)), grey(1.0))

    p["divider"] = ornament(128, 24, divider, ss=1)
    p["flourish"] = ornament(64, 24, flourish, ss=1)
    p["corner"] = np.zeros((32, 32, 4))
    tile = np.zeros((64, 64, 4))
    ys, xs = np.mgrid[0:64, 0:64]
    tile[..., :3] = 1.0
    tile[..., 3] = (((xs // 2) + (ys // 2)) % 2 == 0) * 0.5
    p["tile"] = tile
    skin = dict(
        name="pixel", base="dark",
        palette=dict(
            bg="#14111f", bgSubtle="#1d1930", surface="#2b2745", surfaceRaised="#2b2745", surfaceHover="#3a3459",
            materialThick="#221e38f2", materialRegular="#221e38e6", materialThin="#221e38cc",
            fg="#f4f1de", fgMuted="#c9c2a8", fgSubtle="#8c8670", fgInverse="#14111f",
            fill="#5b5480", fillStrong="#746aa3", border="#5b5480", borderStrong="#a79ccf",
            accent="#ffcd4a", accentHover="#ffdb75", accentActive="#e6ad1f", accentFg="#14111f", accentInk="#ffd86b",
            accentSoft="#ffcd4a33", focus="#ffcd4a", thumb="#5b5480", scrim="#0a0812b3",
            success="#6bd36b", warn="#ffa53d", danger="#ff5d5d", info="#5dc2ff", shadow="#00000099",
        ),
        radius=dict(control=0, field=0, block=0, card=0, panel=0, sheet=0, button=0),
        ramp=dict(xs=(12, 16), meta=(13, 18), caption=(13, 18), sm=(14, 18), body=(16, 20), reading=(16, 22), base=(16, 22),
                  lg=(18, 24), xl=(22, 28), xl2=(24, 28), xl3=(28, 32), displaySm=(32, 36), displayMd=(36, 40), displayLg=(44, 48)),
        durations=dict(instant=80, fast=120, base=160, slow=320),
        fonts=dict(body="DotGothic16-Regular.ttf", display="DotGothic16-Regular.ttf", numeric="DotGothic16-Regular.ttf"),
        slices=dict(panel=(8,), card=(4,), button=(6,), buttonHover=(6,), buttonPressed=(6,), buttonDisabled=(6,),
                    field=(4,), slot=(4,), tooltip=(4,), capsule=(4,), track=(4,), fill=(4,)),
        scale=1, divider=(128, 24, "border"), flourish=(32, 12, "accent", 0),
        tile=(16, 16, 0.06, 6, ["panel"]),
        shadows=dict(panel=(6, 6, 0, 0, "#000000b0"), card=(4, 4, 0, 0, "#00000080"), button=(0, 4, 0, 0, "#000000a0"),
                     capsule=(3, 3, 0, 0, "#00000080"), tooltip=(3, 3, 0, 0, "#00000080")),
        text=dict(textShadow=[("display", 3, 3, 0, "#000000c0"), ("body", 1, 1, 0, "#00000080")]),
        press=(1.0, 1.0, 3), hover=(-1, 1.0),
        motion=dict(enter="slideDown", stagger=0, toast="slideUp", progress=240, shake=4, pop=0.0),
        icons="pixel",
    )
    return p, skin


def cartoon_pack():
    outline = grey(0.16)
    p = {}
    p["panel"] = frame(96, 96, r=28, outline=5, outline_c=outline, fill=(grey(1.0), grey(0.94)), highlight=0, inner=0, shade=7)
    p["card"] = frame(64, 64, r=18, outline=4, outline_c=outline, fill=(grey(1.0), grey(0.93)), shade=5)
    p["button"] = frame(48, 48, r=18, outline=4, outline_c=outline, fill=(grey(1.0), grey(0.84)), highlight=8, shade=7)
    p["buttonHover"] = frame(48, 48, r=18, outline=4, outline_c=outline, fill=(grey(1.0), grey(0.92)), highlight=9, shade=7)
    p["buttonPressed"] = frame(48, 48, r=18, outline=4, outline_c=outline, fill=(grey(0.82), grey(0.9)), highlight=0, shade=0)
    p["buttonDisabled"] = frame(48, 48, r=18, outline=4, outline_c=grey(0.45), fill=(grey(0.9), grey(0.82)), shade=4)
    p["field"] = frame(48, 48, r=14, outline=3, outline_c=outline, fill=(grey(0.9), grey(1.0)))
    p["slot"] = frame(48, 48, r=14, outline=3, outline_c=outline, fill=(grey(0.86), grey(0.98)), shade=0)
    p["tooltip"] = frame(32, 32, r=12, outline=3, outline_c=outline, fill=(grey(1.0), grey(0.95)))
    p["capsule"] = frame(32, 32, r=16, outline=3, outline_c=outline, fill=(grey(1.0), grey(0.9)), shade=3)
    p["track"] = frame(32, 16, r=8, outline=3, outline_c=outline, fill=(grey(0.8), grey(0.95)))
    p["fill"] = frame(32, 16, r=8, outline=3, outline_c=outline, fill=(grey(1.0), grey(0.85)), highlight=3)
    p["soft"] = soft_rect(64, 64, 20)

    def divider(cv):
        for k, x in enumerate([52, 64, 76]):
            disc(cv, x, 12, 4 if k == 1 else 3)

    def flourish(cv):
        # A four-point sparkle.
        cx, cy = 32, 12
        d = np.minimum(np.abs(cv.px - cx) * 0.35 + np.abs(cv.py - cy) * 1.2, np.abs(cv.px - cx) * 1.2 + np.abs(cv.py - cy) * 0.35) - 3.4
        cv.paint(cv.inside(d), grey(1.0))

    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = ornament(64, 24, flourish)
    p["corner"] = np.zeros((32, 32, 4))
    p["tile"] = np.zeros((64, 64, 4))
    skin = dict(
        name="cartoon", base="light",
        palette=dict(
            bg="#2a1d5c", bgSubtle="#33246e", surface="#fff4dc", surfaceRaised="#fff4dc", surfaceHover="#ffe9bd",
            materialThick="#fff4dcf5", materialRegular="#fff4dcee", materialThin="#fff4dce0",
            fg="#2b1a4a", fgMuted="#5a4880", fgSubtle="#8a7aa8", fgInverse="#fff4dc",
            fill="#ffd166", fillStrong="#ffc23d", border="#2b1a4a", borderStrong="#2b1a4a",
            accent="#ff7a2f", accentHover="#ff8f4f", accentActive="#ee6418", accentFg="#2b1a4a", accentInk="#b4430a",
            accentSoft="#ff7a2f40", focus="#3fa7ff", thumb="#ffffff", scrim="#1a0f3ccc",
            success="#2fb85a", warn="#f2a516", danger="#e8364e", info="#2f8ff0", shadow="#2b1a4a99",
        ),
        radius=dict(control=12, field=14, block=14, card=18, panel=28, sheet=28, button=9999),
        ramp=dict(body=(15, 20), meta=(13, 18), caption=(13, 18), lg=(19, 26), xl=(22, 28), xl2=(26, 30), xl3=(30, 34),
                  displaySm=(32, 36), displayMd=(40, 44), displayLg=(52, 56)),
        durations=dict(instant=110, fast=160, base=280, slow=420),
        fonts=dict(body="Fredoka-SemiBold.ttf", display="LilitaOne-Regular.ttf", numeric="LilitaOne-Regular.ttf"),
        slices=dict(panel=(30,), card=(20,), button=(20,), buttonHover=(20,), buttonPressed=(20,), buttonDisabled=(20,),
                    field=(16,), slot=(16,), tooltip=(13,), capsule=(15,), track=(7,), fill=(7,)),
        scale=1, divider=(48, 16, "accent"), flourish=(22, 10, "accent", 2),
        tile=None,
        shadows=dict(panel=(0, 8, 0, 0, "#2b1a4acc"), card=(0, 5, 0, 0, "#2b1a4a99"), button=(0, 5, 0, 0, "#2b1a4acc"),
                     capsule=(0, 4, 0, 0, "#2b1a4a99"), slot=(0, 3, 0, 0, "#2b1a4a66")),
        text=dict(textShadow=[("display", 0, 4, 0, "#2b1a4a")]),
        press=(1.08, 0.88, 3), hover=(-3, 1.04),
        motion=dict(enter="pop", stagger=50, toast="pop", progress=360, shake=8, pop=0.22),
        icons="cartoon",
    )
    return p, skin


GOLD = hexc("#d9a640")
GOLD_DARK = hexc("#7a5414")
PARCH = hexc("#efdfb8")


def fantasy_pack():
    p = {}
    # Parchment panels, drawn as authored: a warm sheet, darker toward the
    # edges, inside a thin brown rule and a gold one.
    def parchment(w, h, r, border_gap):
        cv = Canvas(w, h)
        d = sd_box(cv.px, cv.py, 0.5, 0.5, w - 0.5, h - 0.5, r)
        cv.paint(cv.inside(d), hexc("#5a3a1a"))
        d2 = sd_box(cv.px, cv.py, 2, 2, w - 2, h - 2, max(0, r - 2))
        edge = np.clip(-d2 / (border_gap * 1.6), 0, 1)
        base = np.asarray(PARCH)[None, None, :] * (0.86 + 0.14 * edge[..., None])
        cv.paint(cv.inside(d2), base)
        ring = cv.inside(sd_box(cv.px, cv.py, border_gap, border_gap, w - border_gap, h - border_gap, 2)) - \
            cv.inside(sd_box(cv.px, cv.py, border_gap + 1.5, border_gap + 1.5, w - border_gap - 1.5, h - border_gap - 1.5, 2))
        cv.paint(np.clip(ring, 0, 1), GOLD_DARK, 0.85)
        return cv.image()

    p["panel"] = parchment(96, 96, 6, 10)
    p["card"] = parchment(64, 64, 4, 6)
    p["button"] = frame(48, 48, r=6, outline=2, outline_c=grey(0.25), fill=(grey(1.0), grey(0.8)), highlight=2, inner=2, inner_c=grey(0.62), shade=3)
    p["buttonHover"] = frame(48, 48, r=6, outline=2, outline_c=grey(0.25), fill=(grey(1.0), grey(0.9)), highlight=3, inner=2, inner_c=grey(0.7), shade=3)
    p["buttonPressed"] = frame(48, 48, r=6, outline=2, outline_c=grey(0.25), fill=(grey(0.78), grey(0.92)), inner=2, inner_c=grey(0.6))
    p["buttonDisabled"] = frame(48, 48, r=6, outline=2, outline_c=grey(0.45), fill=(grey(0.85), grey(0.78)), inner=2, inner_c=grey(0.72))
    p["field"] = frame(48, 48, r=4, outline=2, outline_c=grey(0.35), fill=(grey(0.92), grey(1.0)))
    p["slot"] = frame(48, 48, r=4, outline=2, outline_c=grey(0.3), fill=(grey(0.84), grey(0.98)), inner=2, inner_c=grey(0.72))
    p["tooltip"] = frame(32, 32, r=4, outline=2, outline_c=grey(0.3), fill=(grey(1.0), grey(0.94)))
    p["capsule"] = frame(32, 32, r=8, outline=2, outline_c=grey(0.25), fill=(grey(1.0), grey(0.86)), inner=1.5, inner_c=grey(0.66))
    p["track"] = frame(32, 16, r=4, outline=2, outline_c=grey(0.3), fill=(grey(0.75), grey(0.95)))
    p["fill"] = frame(32, 16, r=4, outline=2, outline_c=grey(0.3), fill=(grey(1.0), grey(0.8)), highlight=2)
    p["soft"] = soft_rect(64, 64, 20)

    def corner(cv):
        # Filigree: two gold arms, a curl and a diamond, outlined in dark gold.
        segs = [(4, 4, 28, 4), (4, 4, 4, 28)]
        bars(cv, segs, 5, GOLD_DARK)
        bars(cv, segs, 2.6, GOLD)
        disc(cv, 12, 12, 5.5, GOLD_DARK)
        disc(cv, 12, 12, 3.8, GOLD)
        diamond(cv, 4, 4, 6.5, GOLD_DARK)
        diamond(cv, 4, 4, 4.5, GOLD)
        disc(cv, 28, 4, 2.6, GOLD_DARK)
        disc(cv, 4, 28, 2.6, GOLD_DARK)

    def divider(cv):
        t = np.clip(1 - np.abs(cv.px - 64) / 62, 0, 1)
        line = cv.inside(np.abs(cv.py - 12) - 0.9) * t
        cv.paint(line, GOLD_DARK)
        diamond(cv, 64, 12, 7, GOLD_DARK)
        diamond(cv, 64, 12, 5, GOLD)
        disc(cv, 50, 12, 2.2, GOLD)
        disc(cv, 78, 12, 2.2, GOLD)

    def flourish(cv):
        # A scroll swirl: an arc and a tail toward the heading.
        r = np.hypot(cv.px - 20, cv.py - 12)
        ring = cv.inside(np.abs(r - 7) - 1.4) * (cv.py < 14)
        cv.paint(ring, GOLD_DARK)
        bars(cv, [(26, 13, 58, 13)], 2.0, GOLD_DARK)
        disc(cv, 13, 13, 2.4, GOLD_DARK)
        diamond(cv, 58, 13, 3.5, GOLD)

    p["corner"] = ornament(32, 32, corner)
    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = ornament(64, 24, flourish)
    n = noise(64, 64, 11, 1)
    fib = noise(64, 64, 12, 4)
    tile = np.zeros((64, 64, 4))
    tile[..., :3] = hexc("#6b4a22")
    tile[..., 3] = np.clip((n - 0.55) * 0.9 + (fib - 0.5) * 0.25, 0, 1)
    p["tile"] = tile
    skin = dict(
        name="fantasy", base="dark",
        palette=dict(
            bg="#1a120b", bgSubtle="#241910", surface="#efdfb8", surfaceRaised="#efdfb8", surfaceHover="#f4e8c8",
            materialThick="#efdfb8", materialRegular="#efdfb8f2", materialThin="#efdfb8e6",
            fg="#2a1a0c", fgMuted="#5c4127", fgSubtle="#8a6d4c", fgInverse="#f4e8c8",
            fill="#d8c08d", fillStrong="#c9ab6f", border="#9b7a4a", borderStrong="#5c4127",
            accent="#c99a3a", accentHover="#d8ac4c", accentActive="#b0832a", accentFg="#2a1a0c", accentInk="#7a4f0e",
            accentSoft="#c99a3a40", focus="#8f2d1f", thumb="#f4e8c8", scrim="#0d0805cc",
            success="#3f7a2a", warn="#a86a12", danger="#9b2a1c", info="#2d5c8a", shadow="#000000aa",
            stage="#1a120b",
        ),
        radius=dict(control=4, field=4, block=4, card=4, panel=6, sheet=6, button=6),
        ramp=dict(body=(15, 22), meta=(13, 18), caption=(13, 18), reading=(16, 24), lg=(19, 26), xl=(22, 28), xl2=(24, 30),
                  xl3=(28, 32), displaySm=(28, 34), displayMd=(34, 40), displayLg=(42, 48)),
        durations=dict(fast=200, base=300, slow=480),
        fonts=dict(body="Newsreader-Medium.ttf", display="Cinzel-Bold.ttf", numeric="Cinzel-Bold.ttf"),
        slices=dict(panel=(16, "authored"), card=(10, "authored"), button=(10,), buttonHover=(10,), buttonPressed=(10,),
                    buttonDisabled=(10,), field=(8,), slot=(10,), tooltip=(8,), capsule=(12,), track=(6,), fill=(6,)),
        scale=1, corner=(28, 28, "authored", 6), corners=["panel"], divider=(112, 20, "authored"), flourish=(48, 18, "authored", 0),
        tile=(64, 64, 0.5, 6, ["panel", "card"]),
        shadows=dict(panel=(0, 10, 18, 2, "#000000b3"), card=(0, 4, 8, 0, "#00000066"), button=(0, 3, 4, 0, "#00000073"),
                     capsule=(0, 3, 6, 0, "#00000080")),
        text=dict(),
        press=(0.98, 0.96, 1), hover=(0, 1.0),
        motion=dict(enter="scale", stagger=35, toast="slideDown", progress=420, shake=5, pop=0.1),
        icons="fantasy",
    )
    return p, skin


def scifi_pack():
    p = {}
    # Tinted frames: a white hairline edge with corner cuts over a dark,
    # translucent body (the tint makes the edge the element's colour and
    # the body a deep shade of it).
    def hud(w, h, cut, edge=1.5, body=0.32, body_a=0.92, notch=True):
        cv = Canvas(w, h)
        outer = sd_chamfer(cv.px, cv.py, 0.5, 0.5, w - 0.5, h - 0.5, cut, (1, 0, 1, 0))
        cv.paint(cv.inside(outer), grey(1.0))
        inner = sd_chamfer(cv.px, cv.py, 0.5 + edge, 0.5 + edge, w - 0.5 - edge, h - 0.5 - edge, cut - edge * 0.4, (1, 0, 1, 0))
        cv.rgba[..., :3] *= 1
        body_mask = cv.inside(inner)
        # Replace the body (not over: the edge stays only on the rim).
        for c in range(3):
            cv.rgba[..., c] = np.where(body_mask > 0, body, cv.rgba[..., c])
        cv.rgba[..., 3] = np.where(body_mask > 0, body_a, cv.rgba[..., 3])
        if notch:
            tick = cv.inside(sd_box(cv.px, cv.py, w - 10, h - 4, w - 3, h - 2.5))
            cv.paint(tick, grey(1.0))
        return cv.image()

    p["panel"] = hud(96, 96, 16, edge=1.5, body=0.18, body_a=0.94)
    p["card"] = hud(64, 64, 10, edge=1.2, body=0.22, body_a=0.94)
    p["button"] = hud(48, 48, 10, edge=1.6, body=0.3, body_a=0.95)
    p["buttonHover"] = hud(48, 48, 10, edge=2.0, body=0.42, body_a=0.96)
    p["buttonPressed"] = hud(48, 48, 10, edge=2.0, body=0.6, body_a=1.0, notch=False)
    p["buttonDisabled"] = hud(48, 48, 10, edge=1.0, body=0.26, body_a=0.85, notch=False)
    p["field"] = hud(48, 48, 6, edge=1.2, body=0.14, body_a=0.94, notch=False)
    p["slot"] = hud(48, 48, 8, edge=1.2, body=0.2, body_a=0.9, notch=False)
    p["tooltip"] = hud(32, 32, 6, edge=1.2, body=0.2, body_a=0.96, notch=False)
    p["capsule"] = hud(32, 32, 8, edge=1.2, body=0.22, body_a=0.94, notch=False)
    p["track"] = hud(32, 16, 4, edge=1.0, body=0.3, body_a=0.9, notch=False)
    p["fill"] = frame(32, 16, r=0, fill=(grey(1.0), grey(0.8)))
    p["soft"] = soft_rect(64, 64, 20)

    def corner(cv):
        bars(cv, [(2, 2, 18, 2), (2, 2, 2, 18)], 2.4)
        disc(cv, 7.5, 7.5, 1.6)

    def divider(cv):
        bars(cv, [(4, 12, 124, 12)], 1.0, grey(1.0), 0.5)
        for x in (4, 34, 64, 94, 124):
            bars(cv, [(x, 8, x, 16)], 1.6)
        cv.paint(cv.inside(sd_box(cv.px, cv.py, 56, 10, 72, 14)), grey(1.0))

    def flourish(cv):
        # Two chevrons pointing at the heading.
        for x in (36, 46):
            bars(cv, [(x, 5, x + 7, 12), (x + 7, 12, x, 19)], 2.4)

    p["corner"] = ornament(32, 32, corner)
    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = ornament(64, 24, flourish)
    tile = np.zeros((64, 64, 4))
    ys, xs = np.mgrid[0:64, 0:64]
    tile[..., :3] = hexc("#7fe8ff")
    tile[..., 3] = ((ys % 3) == 0) * 0.5
    p["tile"] = tile
    skin = dict(
        name="scifi", base="dark",
        palette=dict(
            bg="#03080f", bgSubtle="#071220", surface="#1a4a66", surfaceRaised="#1a4a66", surfaceHover="#225a7a",
            materialThick="#1a4a66", materialRegular="#1a4a66e6", materialThin="#1a4a66cc",
            fg="#e2f7ff", fgMuted="#a6cde0", fgSubtle="#6f93a6", fgInverse="#03080f",
            fill="#3f8fb3", fillStrong="#5aaed4", border="#2c6a88", borderStrong="#6fd3f5",
            accent="#38e1ff", accentHover="#6ae9ff", accentActive="#16c4e6", accentFg="#e2f7ff", accentInk="#7fecff",
            accentSoft="#38e1ff33", focus="#ffd23f", thumb="#2f7194", scrim="#010409cc",
            success="#3dffa0", warn="#ffc53d", danger="#ff4d6a", info="#38e1ff", shadow="#00000099",
        ),
        radius=dict(control=2, field=2, block=2, card=2, panel=2, sheet=2, button=2),
        ramp=dict(body=(14, 20), meta=(12, 16), caption=(12, 16), lg=(17, 24), xl=(19, 26), xl2=(22, 28), xl3=(26, 30),
                  displaySm=(26, 32), displayMd=(32, 38), displayLg=(38, 44)),
        durations=dict(instant=90, fast=140, base=220, slow=360),
        fonts=dict(body="ChakraPetch-Medium.ttf", display="Orbitron-Bold.ttf", numeric="JetBrainsMono-Regular.ttf"),
        slices=dict(panel=(18,), card=(12,), button=(12,), buttonHover=(12,), buttonPressed=(12,), buttonDisabled=(12,),
                    field=(8,), slot=(10,), tooltip=(8,), capsule=(10,), track=(5,), fill=(2,)),
        scale=1, corner=(18, 18, "accent", 4), corners=["panel", "card", "slot"], divider=(128, 24, "accent"), flourish=(32, 14, "accent", 0),
        tile=(64, 64, 0.08, 3, ["panel"]),
        shadows=dict(panel=(0, 0, 18, 0, "#000000aa")),
        glows=dict(button=(5, "accent", 0.22), panel=(10, "accent", 0.12), capsule=(4, "accent", 0.18)),
        text=dict(textGlow=[("display", 1.5, "accent", 0.55)]),
        press=(0.98, 0.98, 0), hover=(0, 1.02),
        motion=dict(enter="fade", stagger=30, toast="slideLeft", progress=300, shake=6, pop=0.1),
        icons="scifi",
    )
    return p, skin


def paper_pack():
    p = {}
    INK = grey(0.1)

    def sheet(w, h, r, ink=1.6, wobble=0.6, seed=0, fill=(grey(1.0), grey(0.97)), fill_a=1.0):
        cv = Canvas(w, h)
        ang = np.arctan2(cv.py - h / 2, cv.px - w / 2)
        jitter = wobble * (np.sin(ang * 5 + seed) * 0.6 + np.sin(ang * 11 + seed * 2.1) * 0.4)
        d = sd_box(cv.px, cv.py, 1, 1, w - 1, h - 1, r) + jitter
        cv.paint(cv.inside(d), INK)
        cv.paint(cv.inside(d + ink), vgrad(cv, fill[0], fill[1]), fill_a)
        return cv.image()

    p["panel"] = sheet(96, 96, 8, ink=1.4, wobble=0.5, seed=1)
    p["card"] = sheet(64, 64, 6, ink=1.3, wobble=0.45, seed=2)
    p["button"] = sheet(48, 48, 8, ink=1.6, wobble=0.5, seed=3)
    p["buttonHover"] = sheet(48, 48, 8, ink=2.2, wobble=0.5, seed=3)
    p["buttonPressed"] = sheet(48, 48, 8, ink=2.2, wobble=0.5, seed=4, fill=(grey(0.9), grey(0.96)))
    p["buttonDisabled"] = sheet(48, 48, 8, ink=1.0, wobble=0.4, seed=5, fill=(grey(0.96), grey(0.96)))
    p["field"] = sheet(48, 48, 4, ink=1.2, wobble=0.3, seed=6)
    p["slot"] = sheet(48, 48, 5, ink=1.1, wobble=0.35, seed=7, fill=(grey(0.97), grey(0.94)))
    p["tooltip"] = sheet(32, 32, 4, ink=1.2, wobble=0.3, seed=8)
    p["capsule"] = sheet(32, 32, 14, ink=1.3, wobble=0.35, seed=9)
    p["track"] = sheet(32, 16, 7, ink=1.1, wobble=0.25, seed=10)
    p["fill"] = sheet(32, 16, 7, ink=1.1, wobble=0.25, seed=11)
    p["soft"] = soft_rect(64, 64, 20)

    def divider(cv):
        # A brush stroke: thick in the middle, tapering, slightly wavy.
        t = (cv.px - 4) / 120
        thick = 2.6 * np.sin(np.clip(t, 0, 1) * math.pi) ** 0.7
        y = 12 + 1.2 * np.sin(t * 7)
        cv.paint(cv.inside(np.abs(cv.py - y) - thick / 2) * (t >= 0) * (t <= 1), grey(1.0))

    def flourish(cv):
        bars(cv, [(30, 12, 54, 12)], 2.2)

    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = ornament(64, 24, flourish)
    p["corner"] = np.zeros((32, 32, 4))
    n = noise(64, 64, 21, 1)
    tile = np.zeros((64, 64, 4))
    tile[..., :3] = hexc("#7a6a50")
    tile[..., 3] = np.clip((n - 0.62) * 0.7, 0, 1)
    p["tile"] = tile
    skin = dict(
        name="paper", base="light",
        palette=dict(
            bg="#e8e2d4", bgSubtle="#ded7c6", surface="#fbf8f0", surfaceRaised="#fbf8f0", surfaceHover="#f3eee2",
            materialThick="#fbf8f0f5", materialRegular="#fbf8f0eb", materialThin="#fbf8f0dd",
            fg="#1c1a17", fgMuted="#57524a", fgSubtle="#8c867b", fgInverse="#fbf8f0",
            fill="#efe9dc", fillStrong="#e2dac8", border="#cfc6b2", borderStrong="#1c1a17",
            accent="#c8402b", accentHover="#b5372a", accentActive="#ad3220", accentFg="#fffaf3", accentInk="#a8321f",
            accentSoft="#c8402b26", focus="#2b5cc8", thumb="#fbf8f0", scrim="#2a261f73",
            success="#2f7d4a", warn="#a8650f", danger="#b3261e", info="#2b5cc8", shadow="#2a261f40",
        ),
        radius=dict(control=6, field=4, block=6, card=6, panel=8, sheet=8, button=8),
        ramp=dict(body=(15, 22), meta=(13, 18), caption=(13, 18), reading=(16, 25), lg=(19, 26), xl=(22, 30),
                  xl2=(26, 32), xl3=(30, 36), displaySm=(30, 36), displayMd=(38, 44), displayLg=(48, 54)),
        durations=dict(fast=180, base=260, slow=420),
        fonts=dict(body="InstrumentSans-Regular.ttf", display="Newsreader-SemiBold.ttf", numeric="PTMono-Regular.ttf"),
        slices=dict(panel=(12,), card=(10,), button=(12,), buttonHover=(12,), buttonPressed=(12,), buttonDisabled=(12,),
                    field=(8,), slot=(8,), tooltip=(8,), capsule=(14,), track=(7,), fill=(7,)),
        scale=1, divider=(120, 18, "fg"), flourish=(22, 10, "accent", 2),
        tile=(64, 64, 0.6, 4, ["panel", "card"]),
        shadows=dict(panel=(0, 6, 16, 0, "#2a261f3a"), card=(0, 3, 8, 0, "#2a261f2e"), button=(0, 2, 4, 0, "#2a261f33"),
                     capsule=(0, 2, 6, 0, "#2a261f33"), tooltip=(0, 2, 6, 0, "#2a261f33")),
        text=dict(),
        press=(0.98, 0.98, 1), hover=(-1, 1.0),
        motion=dict(enter="slideUp", stagger=40, toast="slideUp", progress=380, shake=5, pop=0.08),
        icons="paper",
    )
    return p, skin


MAGENTA = hexc("#ff3df2")
CYAN = hexc("#36e8ff")


def neon_pack():
    p = {}

    def tube(w, h, r, line=2.0, halo=4.0, body=0.16, body_a=0.9, rgb=grey(1.0), inset=None):
        """A glowing tube around a dark body (white: takes the tint)."""
        cv = Canvas(w, h)
        m = halo + 1 if inset is None else inset
        d = sd_box(cv.px, cv.py, m, m, w - m, h - m, r)
        inside = cv.inside(d + line / 2)
        for c in range(3):
            cv.rgba[..., c] = body * (rgb[c] if isinstance(rgb, tuple) else 1)
        cv.rgba[..., 3] = inside * body_a
        ring = np.abs(d)
        glow = np.where(ring < line / 2, 1.0, np.exp(-((ring - line / 2) / halo) ** 2) * 0.7)
        cv.paint(glow, rgb)
        return cv.image()

    def two_tone(w, h, r):
        # The panel, authored: a magenta tube outside, a thin cyan one inside.
        cv = Canvas(w, h)
        m = 6
        d = sd_box(cv.px, cv.py, m, m, w - m, h - m, r)
        cv.paint(cv.inside(d), hexc("#120a24"), 0.93)
        ring = np.abs(d)
        cv.paint(np.where(ring < 1.2, 1.0, np.exp(-((ring - 1.2) / 4) ** 2) * 0.75), MAGENTA)
        d2 = sd_box(cv.px, cv.py, m + 5, m + 5, w - m - 5, h - m - 5, max(0, r - 4))
        cv.paint(cv.inside(np.abs(d2) - 0.5), CYAN, 0.75)
        return cv.image()

    p["panel"] = two_tone(96, 96, 14)
    p["card"] = tube(64, 64, 10, line=1.6, halo=3.0, body=0.1, body_a=0.86)
    p["button"] = tube(48, 48, 12, line=2.0, halo=3.5, body=0.22, body_a=0.9)
    p["buttonHover"] = tube(48, 48, 12, line=2.6, halo=4.5, body=0.32, body_a=0.94)
    p["buttonPressed"] = tube(48, 48, 12, line=3.0, halo=5.0, body=0.55, body_a=1.0)
    p["buttonDisabled"] = tube(48, 48, 12, line=1.2, halo=1.5, body=0.18, body_a=0.8)
    p["field"] = tube(48, 48, 6, line=1.4, halo=2.5, body=0.08, body_a=0.94)
    p["slot"] = tube(48, 48, 8, line=1.4, halo=2.5, body=0.12, body_a=0.88)
    p["tooltip"] = tube(32, 32, 6, line=1.4, halo=2.5, body=0.12, body_a=0.94)
    p["capsule"] = tube(32, 32, 10, line=1.6, halo=3.0, body=0.12, body_a=0.9)
    p["track"] = tube(32, 16, 4, line=1.2, halo=1.5, body=0.2, body_a=0.9, inset=2.5)
    p["fill"] = tube(32, 16, 4, line=1.6, halo=1.5, body=0.9, body_a=1.0, inset=2.5)
    p["soft"] = soft_rect(64, 64, 20)

    def divider(cv):
        glow_line(cv, [(6, 12, 122, 12)], 1.6, 3)
        diamond(cv, 64, 12, 4.5)

    def flourish(cv):
        glow_line(cv, [(34, 7, 54, 7), (30, 12, 54, 12), (38, 17, 54, 17)], 1.6, 2.2)

    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = ornament(64, 24, flourish)

    def corner(cv):
        glow_line(cv, [(5, 5, 5, 16), (5, 5, 16, 5)], 2.0, 2.5)

    p["corner"] = ornament(32, 32, corner)
    tile = np.zeros((64, 64, 4))
    ys, xs = np.mgrid[0:64, 0:64]
    tile[..., :3] = MAGENTA
    tile[..., 3] = (((xs % 16) == 0) | ((ys % 16) == 0)) * 0.35
    p["tile"] = tile
    skin = dict(
        name="neon", base="dark",
        palette=dict(
            bg="#07020f", bgSubtle="#0e0520", surface="#1b0f36", surfaceRaised="#1b0f36", surfaceHover="#26164a",
            materialThick="#1b0f36", materialRegular="#1b0f36eb", materialThin="#1b0f36d9",
            fg="#fbefff", fgMuted="#cdb6ea", fgSubtle="#8d76ad", fgInverse="#07020f",
            fill="#8a5cff", fillStrong="#a27dff", border="#4a2f86", borderStrong="#b18cff",
            accent="#ff3df2", accentHover="#ff6ff6", accentActive="#e01fd3", accentFg="#fbefff", accentInk="#ff8af8",
            accentSoft="#ff3df240", focus="#36e8ff", thumb="#5a3aa6", scrim="#030008cc",
            success="#3dffb0", warn="#ffd23d", danger="#ff3d6e", info="#36e8ff", shadow="#000000aa",
        ),
        radius=dict(control=8, field=6, block=8, card=10, panel=14, sheet=14, button=12),
        ramp=dict(body=(15, 20), meta=(13, 18), caption=(13, 18), lg=(19, 24), xl=(22, 28), xl2=(26, 30), xl3=(30, 34),
                  displaySm=(32, 36), displayMd=(40, 44), displayLg=(50, 54)),
        durations=dict(instant=90, fast=140, base=220, slow=380),
        fonts=dict(body="Rajdhani-Bold.ttf", display="ChakraPetch-BoldItalic.ttf", numeric="Orbitron-Medium.ttf"),
        slices=dict(panel=(22, "authored"), card=(14,), button=(16,), buttonHover=(16,), buttonPressed=(16,), buttonDisabled=(16,),
                    field=(10,), slot=(12,), tooltip=(10,), capsule=(14,), track=(6,), fill=(6,)),
        scale=1, corner=(20, 20, "accent", 2), corners=["card"], divider=(128, 24, "accent"), flourish=(36, 16, "accent", 0),
        tile=(48, 48, 0.12, 10, ["panel"]),
        shadows=dict(panel=(0, 0, 0, 0, "#00000000")),
        glows=dict(button=(6, "accent", 0.35), capsule=(4, "accent", 0.25), panel=(14, "accent", 0.18), slot=(3, "focus", 0.2)),
        text=dict(textGlow=[("display", 2.0, "accent", 0.7), ("numeric", 1.2, "focus", 0.5)]),
        press=(0.96, 0.96, 0), hover=(0, 1.04),
        motion=dict(enter="pop", stagger=35, toast="slideLeft", progress=260, shake=7, pop=0.16),
        icons="neon",
    )
    return p, skin


PACKS = [pixel_pack, cartoon_pack, fantasy_pack, scifi_pack, paper_pack, neon_pack]


# ── output ──────────────────────────────────────────────────────────────────

def to8(img):
    return (np.clip(img, 0, 1) * 255 + 0.5).astype(np.uint8)


def build_atlas(parts):
    atlas = np.zeros((ATLAS_H, ATLAS_W, 4), dtype=np.uint8)
    for name, (x, y, w, h) in LAYOUT.items():
        img = to8(parts[name])
        assert img.shape[:2] == (h, w), (name, img.shape, (w, h))
        # Edge texels repeat 2 px into the gutter (no neighbour bleeds in).
        padded = np.pad(img, ((2, 2), (2, 2), (0, 0)), mode="edge")
        atlas[y - 2:y + h + 2, x - 2:x + w + 2] = padded
    return Image.fromarray(atlas, "RGBA")


def region(name):
    x, y, w, h = LAYOUT[name]
    return f"{x} {y} {w} {h}"


def skin_text(s, d):
    n = s["name"]
    L = [
        "type skin",
        "version 1",
        "---",
        f"# The {n} style pack: generated by templates/ui-kit/tools/make-skins.py (CC0 art, OFL fonts; LICENSES.md).",
        f'name "{n}"',
        f"base {s['base']}",
        f'atlas "skins/{n}/atlas.png" {ATLAS_W} {ATLAS_H}',
        f"scale {s['scale']}",
        "palette {",
    ]
    L += [f'  {k} "{v}"' for k, v in s["palette"].items()]
    L += ["}", "radius {"] + [f"  {k} {v}" for k, v in s["radius"].items()] + ["}"]
    L += ["ramp {"] + [f"  {k} {a} {b}" for k, (a, b) in s["ramp"].items()] + ["}"]
    L += ["durations {"] + [f"  {k} {v}" for k, v in s["durations"].items()] + ["}"]
    L += [
        "fonts {",
        f'  body "skins/{n}/{s["fonts"]["body"]}"',
        f'  display "skins/{n}/{s["fonts"]["display"]}"',
        f'  numeric "skins/{n}/{s["fonts"]["numeric"]}"',
        '  fallback "fonts/NotoSansSC-Regular-subset.ttf"',
        "}",
        f'icons "skins/{n}/icons.png" 8 4 {{',
        "  names " + " ".join(f'"{i}"' for i in ICONS),
        "}",
    ]
    for part, spec in s["slices"].items():
        border = spec[0]
        mode = spec[1] if len(spec) > 1 else "tint"
        L.append(f"slice {part} {region(part)} {border} {mode}")
    L.append(f"soft {region('soft')} 24")
    if s.get("corner"):
        w, h, ink, off = s["corner"]
        L.append(f"corner {region('corner')} {w} {h} {ink} {off}")
        L.append("corners " + " ".join(s["corners"]))
    if s.get("divider"):
        w, h, ink = s["divider"]
        L.append(f"divider {region('divider')} {w} {h} {ink}")
    if s.get("flourish"):
        w, h, ink, off = s["flourish"]
        L.append(f"flourish {region('flourish')} {w} {h} {ink} {off}")
    if s.get("tile"):
        w, h, a, inset, parts = s["tile"]
        L.append(f"tile {region('tile')} {w} {h} {a} {inset} " + " ".join(parts))
    for part, (dx, dy, blur, spread, c) in s["shadows"].items():
        L.append(f'shadow {part} {dx} {dy} {blur} {spread} "{c}"')
    for part, (size, ink, a) in s.get("glows", {}).items():
        L.append(f"glow {part} {size} {ink} {a}")
    for role, dx, dy, blur, c in s["text"].get("textShadow", []):
        L.append(f'textShadow {role} {dx} {dy} {blur} "{c}"')
    for role, size, ink, a in s["text"].get("textGlow", []):
        L.append(f"textGlow {role} {size} {ink} {a}")
    L.append("press {} {} {}".format(*s["press"]))
    L.append("hover {} {}".format(*s["hover"]))
    m = s["motion"]
    L += [
        "motion {",
        f"  enter {m['enter']}",
        f"  stagger {m['stagger']}",
        f"  toast {m['toast']}",
        f"  progress {m['progress']}",
        f"  shake {m['shake']}",
        f"  pop {m['pop']}",
        "}",
    ]
    return "\n".join(L) + "\n"


def font_import(display, glows=False):
    em = 48 if display else 32
    # A role with a textGlow reads its glow from the distance field, which
    # reaches half px_range atlas pixels: 12 lets it spread 6 (4 lets 2).
    rng = 12 if glows else 4
    why = "; px_range 12 for its text glow" if glows else ""
    return (
        "type import\nversion 1\n---\n"
        f"# Latin-1 and the typographic punctuation the kit shows; {em} px per em in the atlas{why}.\n"
        "font {\n  charset latin punct\n"
        f"  em_px {em}\n  px_range {rng}\n}}\n"
    )


def ui_sounds(out):
    """audio/ui-<event>.wav: short synthesized blips the demo's `sounds:on`
    hands og.ui.configure (16-bit mono, 44.1 kHz, sines under an envelope)."""
    import wave
    rate = 44100
    notes = {  # event: [(start s, length s, from hz, to hz, gain)]
        "hover": [(0.0, 0.035, 1800, 2000, 0.12)],
        "press": [(0.0, 0.06, 660, 520, 0.35)],
        "open": [(0.0, 0.08, 520, 660, 0.25), (0.06, 0.1, 780, 990, 0.22)],
        "close": [(0.0, 0.08, 780, 620, 0.22), (0.06, 0.1, 520, 400, 0.2)],
        "success": [(0.0, 0.09, 660, 660, 0.25), (0.08, 0.09, 880, 880, 0.25), (0.16, 0.16, 1320, 1320, 0.22)],
        "error": [(0.0, 0.16, 220, 180, 0.35), (0.0, 0.16, 233, 190, 0.25)],
    }
    os.makedirs(os.path.join(out, "audio"), exist_ok=True)
    for event, parts in notes.items():
        n = int(rate * max(a + b for a, b, *_ in parts)) + 1
        buf = np.zeros(n)
        for start, length, f0, f1, gain in parts:
            k = np.arange(int(rate * length))
            t = k / rate
            f = f0 + (f1 - f0) * t / length
            phase = 2 * np.pi * np.cumsum(f) / rate
            env = np.minimum(1, t / 0.004) * (1 - t / length) ** 2
            i = int(rate * start)
            buf[i:i + len(k)] += np.sin(phase) * env * gain
        pcm = (np.clip(buf, -1, 1) * 32767).round().astype("<i2")
        with wave.open(os.path.join(out, "audio", f"ui-{event}.wav"), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(rate)
            w.writeframes(pcm.tobytes())


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "templates/ui-kit/assets"
    ui_sounds(out)
    for pack in PACKS:
        parts, s = pack()
        d = os.path.join(out, "skins", s["name"])
        os.makedirs(d, exist_ok=True)
        build_atlas(parts).save(os.path.join(d, "atlas.png"), optimize=True)
        icon_sheet(s["icons"]).save(os.path.join(d, "icons.png"), optimize=True)
        with open(os.path.join(d, "skin.ogt"), "w") as f:
            f.write(skin_text(s, d))
        for name in ("atlas.png", "icons.png"):
            with open(os.path.join(d, name + ".ogt"), "w") as f:
                f.write("type import\nversion 1\n---\n# Sharp art: UASTC.\ntexture {\n  kind "
                        + ("linear" if name == "icons.png" else "color") + "\n}\n")
        roles = s["fonts"]
        imports = {}
        for role in ("body", "display", "numeric"):
            imports[roles[role]] = imports.get(roles[role], False) or role != "body"
        glowing = {roles[role] for role, *_ in s["text"].get("textGlow", [])}
        for name, display in imports.items():
            with open(os.path.join(d, name + ".ogt"), "w") as f:
                f.write(font_import(display, name in glowing))
        print("wrote", d)


if __name__ == "__main__":
    main()
