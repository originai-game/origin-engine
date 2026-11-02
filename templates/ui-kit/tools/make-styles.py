#!/usr/bin/env python3
"""Regenerates the engine's four atlas UI styles (CC0 art), byte for byte,
the icon sheets the seed styles share, and the UI kit's UI sounds:

    python3 templates/ui-kit/tools/make-styles.py templates/ui-kit/assets

For each of pixel, cartoon, fantasy and scifi it writes
templates/styles/<name>/ (og's cook reads them from there or from the
engine package's styles/; a look names one with `ui "<name>"`), and it
writes templates/styles/shared/ink-icons.png and glow-icons.png, the
seed styles' icons (the paper and neon packs' before they were folded into
storybook and cyber, 2026-11-02):

- atlas.png: the nine-slice frames (panel, card, button and its hover,
  pressed and disabled states, field, slot, tooltip, capsule, progress track
  and fill), a soft blurred rectangle for shadows and glows, the ornaments
  (corner, divider, heading flourish, and for fantasy a top-centre crest
  and a mid-edge filigree) and a tile, each with a 4 px gutter
  whose texels repeat the region's edge (so filtering never bleeds a
  neighbour in). Frames meant to take the palette are drawn in white and
  greys (og-ui multiplies them by the element's colour); "authored" ones
  carry their own colours.
- icons.png: the kit's 32 icons (tools/make-assets.py draws them) restyled
  per pack: pixelated, thickened, outlined, roughened or neon-lined; white,
  alpha is coverage.
- style.ogt: the style, version 2 (og-ui's `parse_style` grammar, docs/ui.md
  §12): every palette role, the tokens (colours with their cues, the type
  scale, space, size, radius and stroke scales, elevation, the shape
  language, motion, sounds and haptics; TOKENS below, chosen per style), the
  ramp, the fonts, the atlas, and its parts dressed by selector.
  Chinese comes from the pack's companions, `cjk` (body and numbers) and
  `cjkDisplay` (titles), shared Simplified Chinese faces in
  templates/styles/cjk/ chosen to match each pack's Latin voice.

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
    "hud": (144, 104, 32, 32),
    "track": (180, 104, 32, 16),
    "fill": (216, 104, 32, 16),
    "soft": (252, 104, 64, 64),
    "corner": (320, 104, 32, 32),
    "divider": (4, 172, 128, 24),
    "flourish": (136, 172, 64, 24),
    "tile": (204, 172, 64, 64),
    "crest": (272, 172, 64, 32),
    "edge": (344, 172, 16, 64),
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
    p["hud"] = pixel_frame(32, 32, cut=2, outline=2, bevel=2, light=1.0, dark=0.7, body=0.9)
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
    style = dict(
        name="pixel", base="dark",
        palette=dict(
            backdrop="#14111f", well="#1d1930", panel="#2b2745", panelHover="#3a3459", panelRaised="#2b2745", line="#5b5480",
            lineStrong="#a79ccf", ink="#f4f1de", inkMuted="#c9c2a8", inkFaint="#ada792", inkInverse="#14111f", control="#3f3a5e",
            controlStrong="#4c4570", primary="#ffcd4a", primaryHover="#ffdb75", primaryActive="#e6ad1f", onPrimary="#14111f",
            primarySoft="#ffcd4a25", highlight="#ffd86b", focus="#ffcd4a", knob="#5b5480", hud="#221e38f2", scrim="#0a0812b3",
            shadow="#00000099", sheen="#ffffff52", success="#6bd36b", warn="#ffa53d", danger="#ff8080", info="#5dc2ff",
            health="#6bd36b", mana="#0a84ff", xp="#bf5af2", gold="#ffcd4a",
        ),
        radius=dict(small=0, control=0, bar=0, panel=0),
        ramp=dict(xs=(12, 16), meta=(13, 18), caption=(13, 18), sm=(14, 18), body=(16, 20), reading=(16, 22), base=(16, 22),
                  lg=(18, 24), xl=(22, 28), xl2=(24, 28), xl3=(28, 32), displaySm=(32, 36), displayMd=(36, 40), displayLg=(44, 48)),
        durations=dict(instant=80, fast=120, base=160, slow=320),
        fonts=dict(body="DotGothic16-Regular.ttf", display="DotGothic16-Regular.ttf", numeric="DotGothic16-Regular.ttf", cjk="FusionPixel12-zhHans.ttf", cjkDisplay="FusionPixel12-zhHans.ttf"),
        slices=dict(panel=(8,), card=(4,), button=(6,), buttonHover=(6,), buttonPressed=(6,), buttonDisabled=(6,),
                    field=(4,), slot=(4,), tooltip=(4,), hud=(4,), track=(4,), fill=(4,)),
        scale=1, divider=(128, 24, "line"), flourish=(32, 12, "primary", 0),
        tile=(16, 16, 0.06, 6, ["panel"]),
        shadows=dict(panel=(6, 6, 0, 0, "#000000b0"), card=(4, 4, 0, 0, "#00000080"), button=(0, 4, 0, 0, "#000000a0"),
                     hud=(3, 3, 0, 0, "#00000080"), tooltip=(3, 3, 0, 0, "#00000080")),
        # DotGothic16's one-dot strokes are one pixel wide at the body's 16 px
        # but sit between device pixels, so no pixel took more than about half
        # the ink: the gold primary button's label read 2.9:1 (hollowbell
        # FINDINGS #20). Half a pixel each side fills them.
        text=dict(textShadow=[("display", 3, 3, 0, "#000000c0"), ("body", 1, 1, 0, "#00000080")],
                  textWeight=[("body", 0.5)]),
        press=(1.0, 1.0, 3), hover=(-1, 1.0),
        motion=dict(enter="slideDown", stagger=0, toast="slideUp", progress=240, shake=4, pop=0.0),
        icons="pixel",
    )
    return p, style


def cartoon_pack():
    outline = grey(0.16)
    p = {}
    p["panel"] = frame(96, 96, r=14, outline=5, outline_c=outline, fill=(grey(1.0), grey(0.94)), highlight=0, inner=0, shade=7)
    p["card"] = frame(64, 64, r=12, outline=4, outline_c=outline, fill=(grey(1.0), grey(0.93)), shade=5)
    p["button"] = frame(48, 48, r=10, outline=4, outline_c=outline, fill=(grey(1.0), grey(0.84)), highlight=8, shade=7)
    p["buttonHover"] = frame(48, 48, r=10, outline=4, outline_c=outline, fill=(grey(1.0), grey(0.92)), highlight=9, shade=7)
    p["buttonPressed"] = frame(48, 48, r=10, outline=4, outline_c=outline, fill=(grey(0.82), grey(0.9)), highlight=0, shade=0)
    p["buttonDisabled"] = frame(48, 48, r=10, outline=4, outline_c=grey(0.45), fill=(grey(0.9), grey(0.82)), shade=4)
    p["field"] = frame(48, 48, r=8, outline=3, outline_c=outline, fill=(grey(0.9), grey(1.0)))
    p["slot"] = frame(48, 48, r=8, outline=3, outline_c=outline, fill=(grey(0.86), grey(0.98)), shade=0)
    p["tooltip"] = frame(32, 32, r=8, outline=3, outline_c=outline, fill=(grey(1.0), grey(0.95)))
    p["hud"] = frame(32, 32, r=8, outline=3, outline_c=outline, fill=(grey(1.0), grey(0.9)), shade=3)
    p["track"] = frame(32, 16, r=5, outline=3, outline_c=outline, fill=(grey(0.8), grey(0.95)))
    p["fill"] = frame(32, 16, r=5, outline=3, outline_c=outline, fill=(grey(1.0), grey(0.85)), highlight=3)
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
    style = dict(
        name="cartoon", base="light",
        palette=dict(
            backdrop="#2a1d5c", well="#33246e", panel="#fff4dc", panelHover="#ffe9bd", panelRaised="#fff4dc", line="#2b1a4a",
            lineStrong="#2b1a4a", ink="#2b1a4a", inkMuted="#5a4880", inkFaint="#61527c", inkInverse="#fff4dc", control="#fff3cc",
            controlStrong="#ffecb5", primary="#8c360f", primaryHover="#933a10", primaryActive="#772d0b", onPrimary="#fff4dc",
            primarySoft="#8c360f40", highlight="#9b3a09", focus="#3fa7ff", knob="#ffffff", hud="#fff4dcf5", scrim="#1a0f3ccc",
            shadow="#2b1a4a99", sheen="#ffffff52", success="#06681b", warn="#7b5102", danger="#6a0a16", info="#2256af",
            health="#06681b", mana="#007aff", xp="#af52de", gold="#a24112",
        ),
        radius=dict(small=6, control=6, bar=6, panel=10),
        ramp=dict(body=(15, 20), meta=(13, 18), caption=(13, 18), lg=(19, 26), xl=(22, 28), xl2=(26, 30), xl3=(30, 34),
                  displaySm=(32, 36), displayMd=(40, 44), displayLg=(52, 56)),
        durations=dict(instant=110, fast=160, base=280, slow=420),
        fonts=dict(body="Fredoka-SemiBold.ttf", display="LilitaOne-Regular.ttf", numeric="LilitaOne-Regular.ttf", cjk="ZCOOLKuaiLe-Regular.ttf", cjkDisplay="ZCOOLKuaiLe-Regular.ttf"),
        slices=dict(panel=(16,), card=(14,), button=(12,), buttonHover=(12,), buttonPressed=(12,), buttonDisabled=(12,),
                    field=(10,), slot=(10,), tooltip=(10,), hud=(10,), track=(7,), fill=(7,)),
        scale=1, divider=(48, 16, "primary"), flourish=(22, 10, "primary", 2),
        tile=None,
        shadows=dict(panel=(0, 8, 0, 0, "#2b1a4acc"), card=(0, 5, 0, 0, "#2b1a4a99"), button=(0, 5, 0, 0, "#2b1a4acc"),
                     hud=(0, 4, 0, 0, "#2b1a4a99"), slot=(0, 3, 0, 0, "#2b1a4a66")),
        text=dict(textShadow=[("display", 0, 4, 0, "#2b1a4a")]),
        press=(1.08, 0.88, 3), hover=(-3, 1.04),
        motion=dict(enter="pop", stagger=50, toast="pop", progress=360, shake=8, pop=0.22),
        icons="cartoon",
    )
    return p, style


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
    p["hud"] = frame(32, 32, r=8, outline=2, outline_c=grey(0.25), fill=(grey(1.0), grey(0.86)), inner=1.5, inner_c=grey(0.66))
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

    def crest(cv):
        # A crest for a panel's top centre: a gem in a gold mount between
        # two scrolled wings, the band's line running out to each side.
        t = np.clip(1 - np.abs(cv.px - 32) / 31, 0, 1)
        cv.paint(cv.inside(np.abs(cv.py - 20) - 1.0) * t, GOLD_DARK)
        for sx in (-1, 1):
            r = np.hypot(cv.px - (32 + sx * 15), cv.py - 16)
            cv.paint(cv.inside(np.abs(r - 6) - 1.4) * (cv.py < 19), GOLD_DARK)
            disc(cv, 32 + sx * 21, 17, 2.2, GOLD_DARK)
            disc(cv, 32 + sx * 26, 20, 1.6, GOLD)
        diamond(cv, 32, 16, 13, GOLD_DARK)
        diamond(cv, 32, 16, 10.5, GOLD)
        diamond(cv, 32, 16, 6.5, hexc("#6e1b10"))
        diamond(cv, 30.5, 14.5, 2.2, hexc("#c94a36"))
        disc(cv, 32, 3.5, 2.4, GOLD_DARK)
        disc(cv, 32, 3.5, 1.4, GOLD)

    def edge(cv):
        # Filigree for the middle of a panel's left edge (mirrored on the
        # right): a gold rule with a diamond boss and a curl above and below.
        t = np.clip(1 - np.abs(cv.py - 32) / 31, 0, 1)
        cv.paint(cv.inside(np.abs(cv.px - 6) - 1.0) * t, GOLD_DARK)
        for sy in (-1, 1):
            # Two leaves on each side of the boss, leaning away from it.
            for k, (cx, cy) in enumerate([(9.5, 32 + sy * 12), (9.0, 32 + sy * 20)]):
                ang = math.radians(sy * (55 if k == 0 else 40))
                u = (cv.px - cx) * math.cos(ang) + (cv.py - cy) * math.sin(ang)
                v = -(cv.px - cx) * math.sin(ang) + (cv.py - cy) * math.cos(ang)
                ax, ay = (4.6, 1.9) if k == 0 else (3.6, 1.5)
                cv.paint(cv.inside(np.hypot(u / ax, v / ay) - 1.0), GOLD_DARK)
                cv.paint(cv.inside(np.hypot(u / (ax - 1.1), v / max(ay - 0.8, 0.6)) - 1.0), GOLD)
            disc(cv, 6, 32 + sy * 27, 1.8, GOLD)
        diamond(cv, 6, 32, 6.5, GOLD_DARK)
        diamond(cv, 6, 32, 4.5, GOLD)

    p["corner"] = ornament(32, 32, corner)
    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = ornament(64, 24, flourish)
    p["crest"] = ornament(64, 32, crest)
    p["edge"] = ornament(16, 64, edge)
    n = noise(64, 64, 11, 1)
    fib = noise(64, 64, 12, 4)
    tile = np.zeros((64, 64, 4))
    tile[..., :3] = hexc("#6b4a22")
    tile[..., 3] = np.clip((n - 0.55) * 0.9 + (fib - 0.5) * 0.25, 0, 1)
    p["tile"] = tile
    style = dict(
        name="fantasy", base="dark",
        palette=dict(
            backdrop="#1a120b", well="#241910", panel="#efdfb8", panelHover="#f4e8c8", panelRaised="#efdfb8", line="#9b7a4a",
            lineStrong="#5c4127", ink="#2a1a0c", inkMuted="#3d2a18", inkFaint="#654b31", inkInverse="#f4e8c8", control="#e2cc98",
            controlStrong="#d6bc82", primary="#7a520f", primaryHover="#74500e", primaryActive="#6a470c", onPrimary="#fff4dc",
            primarySoft="#7a520f40", highlight="#6e470d", focus="#8f2d1f", knob="#f4e8c8", hud="#efdfb8", scrim="#0d0805cc",
            shadow="#000000aa", sheen="#ffffff52", success="#2d5a1e", warn="#704713", danger="#6e1b10", info="#22517f",
            health="#2d5a1e", mana="#0a84ff", xp="#bf5af2", gold="#7a520f",
        ),
        radius=dict(small=4, control=4, bar=4, panel=4),
        ramp=dict(body=(15, 22), meta=(13, 18), caption=(13, 18), reading=(16, 24), lg=(19, 26), xl=(22, 28), xl2=(24, 30),
                  xl3=(28, 32), displaySm=(28, 34), displayMd=(34, 40), displayLg=(42, 48)),
        durations=dict(fast=200, base=300, slow=480),
        fonts=dict(body="Newsreader-Medium.ttf", display="Cinzel-Bold.ttf", numeric="Cinzel-Bold.ttf", cjk="NotoSerifSC-SemiBold.ttf", cjkDisplay="NotoSerifSC-SemiBold.ttf"),
        slices=dict(panel=(16, "authored"), card=(10, "authored"), button=(10,), buttonHover=(10,), buttonPressed=(10,),
                    buttonDisabled=(10,), field=(8,), slot=(10,), tooltip=(8,), hud=(12,), track=(6,), fill=(6,)),
        scale=1, corner=(28, 28, "authored", 6), corners=["panel"], divider=(112, 20, "authored"), flourish=(48, 18, "authored", 0),
        crest=(56, 28, "authored", 12), edge=(14, 56, "authored", 7), crests=["panel"], edges=["panel"],
        tile=(64, 64, 0.5, 6, ["panel", "card"]),
        shadows=dict(panel=(0, 10, 18, 2, "#000000b3"), card=(0, 4, 8, 0, "#00000066"), button=(0, 3, 4, 0, "#00000073"),
                     hud=(0, 3, 6, 0, "#00000080")),
        text=dict(),
        press=(0.98, 0.96, 1), hover=(0, 1.0),
        motion=dict(enter="scale", stagger=35, toast="slideDown", progress=420, shake=5, pop=0.1),
        icons="fantasy",
    )
    return p, style


def scifi_pack():
    # Real-world spaceflight operations (the owner's direction, 2026-11-02;
    # the approved sheet ui-art-2 59079f25d): flat near-black glass, a 1 px
    # hairline, small radii, one ice-blue accent; no brackets, scanlines or
    # glow. The frames take the element's colour: the edge at full strength,
    # the body a darker shade of it (a button's nearly the same, so a primary
    # stays flat ice blue).
    p = {}

    def flat(w, h, r, body):
        return frame(w, h, r=r, outline=1.0, outline_c=grey(1.0), fill=(grey(body), grey(body)))

    p["panel"] = flat(96, 96, 4, 0.45)
    p["card"] = flat(64, 64, 4, 0.55)
    p["button"] = flat(48, 48, 3, 0.9)
    p["buttonHover"] = flat(48, 48, 3, 1.0)
    p["buttonPressed"] = flat(48, 48, 3, 0.78)
    p["buttonDisabled"] = flat(48, 48, 3, 0.6)
    p["field"] = flat(48, 48, 3, 0.5)
    p["slot"] = flat(48, 48, 3, 0.5)
    p["tooltip"] = flat(32, 32, 3, 0.6)
    p["hud"] = flat(32, 32, 3, 0.55)
    p["track"] = frame(32, 16, r=1.5, fill=(grey(1.0), grey(1.0)))
    p["fill"] = frame(32, 16, r=1.5, fill=(grey(1.0), grey(1.0)))
    p["soft"] = soft_rect(64, 64, 20)

    def divider(cv):
        # A hairline, faint at its ends.
        for x in range(4, 125):
            a = min(1.0, (x - 4) / 24, (124 - x) / 24)
            bars(cv, [(x, 12, x + 1, 12)], 1.0, grey(1.0), max(a, 0.0))

    p["corner"] = np.zeros((32, 32, 4))
    p["divider"] = ornament(128, 24, divider)
    p["flourish"] = np.zeros((24, 64, 4))
    p["tile"] = np.zeros((64, 64, 4))
    style = dict(
        name="scifi", base="dark",
        palette=dict(
            backdrop="#05080c", well="#0a0e13", panel="#252d36", panelHover="#2c3540", panelRaised="#29323c", line="#2a323c",
            lineStrong="#46525f", ink="#eef2f6", inkMuted="#a8b2bd", inkFaint="#808b97", inkInverse="#0a0e13", control="#2f3842",
            controlStrong="#3c4652", primary="#9ccdf5", primaryHover="#b5dbf9", primaryActive="#86bdea", onPrimary="#0b1520",
            primarySoft="#9ccdf526", highlight="#9ccdf5", focus="#d2e8fb", knob="#9ccdf5", hud="#0f1318e6", scrim="#000000b3",
            shadow="#000000aa", sheen="#ffffff1f", success="#5fd38a", warn="#e8b84a", danger="#ea6b5f", info="#9ccdf5",
            health="#5fd38a", mana="#6aa8ff", xp="#b49cf0", gold="#e8c46a",
        ),
        radius=dict(small=2, control=3, bar=2, panel=4),
        ramp=dict(body=(14, 20), meta=(12, 16), caption=(12, 16), lg=(17, 24), xl=(19, 26), xl2=(22, 28), xl3=(26, 30),
                  displaySm=(28, 34), displayMd=(34, 40), displayLg=(42, 48)),
        durations=dict(instant=90, fast=140, base=220, slow=360),
        fonts=dict(body="IBMPlexSans-Regular.ttf", display="IBMPlexSans-Light.ttf", numeric="IBMPlexMono-Regular.ttf", cjk="NotoSansSC-Regular.ttf", cjkDisplay="NotoSansSC-Regular.ttf"),
        slices=dict(panel=(8,), card=(8,), button=(8,), buttonHover=(8,), buttonPressed=(8,), buttonDisabled=(8,),
                    field=(6,), slot=(6,), tooltip=(6,), hud=(6,), track=(4,), fill=(4,)),
        scale=1, divider=(128, 24, "line"),
        tile=None,
        shadows=dict(panel=(0, 12, 32, 0, "#00000080")),
        # Glass: the scene behind blurred 20 px under a near-opaque panel.
        glass=dict(panel=(20, 0.92, 0.1)),
        text=dict(),
        press=(0.98, 0.98, 0), hover=(0, 1.0),
        motion=dict(enter="fade", stagger=30, toast="slideLeft", progress=300, shake=4, pop=0.06),
        icons="scifi",
        # The sheet's toggles: a white knob on the ice track, a grey one off;
        # slider thumbs and the selected segment keep `knob`'s ice.
        dresses={"Toggle.knob[on]": ['fill "#f4f7fa"'], "Toggle.knob[off]": ['fill "#8e98a3"']},
    )
    return p, style


PACKS = [pixel_pack, cartoon_pack, fantasy_pack, scifi_pack]


# ── version 2 tokens (docs/ui.md §12.2) ────────────────────────────────────
#
# Each style's tokens are chosen for it: the rarity, team, damage and chart
# colours start from seeds in the style's own voice and are moved along their
# lightness until they read 3:1 on the style's raised panel (WCAG 1.4.11, the
# bar for a colour that is not text); every status and rarity colour carries a
# cue, an icon or a pattern or pips, so no meaning is colour alone (XAG 103).
# The four element classes version 1 had a radius for (slot: small, button:
# control, bar, panel) keep it, so the style draws as before.

def _lin(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def _lum(rgb):
    r, g, b = (_lin(v) for v in rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def _contrast(a, b):
    x, y = _lum(a), _lum(b)
    return (max(x, y) + 0.05) / (min(x, y) + 0.05)


def _rgba(s):
    s = s.lstrip("#")
    rgb = tuple(int(s[i:i + 2], 16) / 255 for i in (0, 2, 4))
    a = int(s[6:8], 16) / 255 if len(s) == 8 else 1.0
    return rgb, a


def _over(top, a, below):
    return tuple(round((t * a + b * (1 - a)) * 255) / 255 for t, b in zip(top, below))


def _hex(rgb):
    return "#" + "".join(f"{round(v * 255):02x}" for v in rgb)


def readable(seed, ground, bar=3.0):
    """The seed, or the least mix of it towards its own end of the scale
    (white when lighter than the ground, black when darker) that reads
    `bar`:1 on it: its hue kept as far as the bar allows."""
    c = _rgba(seed)[0]
    if _contrast(c, ground) >= bar:
        return _hex(c)
    end = (1.0, 1.0, 1.0) if _lum(c) >= _lum(ground) else (0.0, 0.0, 0.0)
    for k in range(1, 101):
        t = k / 100
        m = tuple(round((a + (b - a) * t) * 255) / 255 for a, b in zip(c, end))
        if _contrast(m, ground) >= bar:
            return _hex(m)
    return _hex(end)


RARITY = ["common", "uncommon", "rare", "epic", "legendary", "mythic"]
DAMAGE = ["physical", "fire", "ice", "lightning", "poison", "holy", "shadow", "arcane"]
EVENTS = ["hover", "press", "focus", "select", "confirm", "cancel", "back", "open", "close", "tab", "toggleOn",
          "toggleOff", "sliderTick", "scrollTick", "typewriter", "counterTick", "notification", "warning", "error",
          "success", "reward", "revealCommon", "revealUncommon", "revealRare", "revealEpic", "revealLegendary",
          "revealMythic"]
CLASSES = ["panel", "dialog", "card", "tooltip", "button", "field", "chip", "badge", "tab", "slot", "bar", "hud", "avatar"]
DURATIONS = dict(instant=120, fast=180, base=240, slow=400, spring=520, bounce=640)  # the prototype's (og-ui theme.rs)
SPACE = dict(xxs=2, xs=4, sm=8, md=12, lg=16, xl=24, xxl=32, xl3=48)

# Haptics shared by most styles (a style overrides what is its own).
HAPTICS = dict(press="tick", select="tick", confirm="tap", cancel="tick", back="tick", tab="tick", toggleOn="tick",
               toggleOff="tick", sliderTick="tick", notification="tap", warning="warning", error="error",
               success="success", reward="double", revealUncommon="tick", revealRare="tap", revealEpic="double",
               revealLegendary="thud", revealMythic="buzz")

TOKENS = {
    "pixel": dict(
        # 8-bit tiers in the NES palette's voice; pips are the pixel stars of a loot drop.
        status=dict(success='icon "check"', warn='icon "flag"', danger='icon "skull"', info="pattern checks"),
        rarity=[("#bcbcbc", "pips 1"), ("#58d854", "pips 2"), ("#3cbcfc", "pips 3"), ("#d878f8", "pips 4"),
                ("#fca044", "pips 5"), ("#f83800", "pips 6")],
        teams=["#3cbcfc", "#f83800", "#58d854", "#f8b800", "#d878f8", "#fca044", "#00e8d8", "#f878f8"],
        damage=[("#bcbcbc", 'icon "sword"'), ("#f83800", 'icon "fire"'), ("#3cbcfc", 'icon "drop"'),
                ("#f8b800", 'icon "bolt"'), ("#58d854", "pattern checks"), ("#fcfcfc", 'icon "star"'),
                ("#9878f8", "pattern grid"), ("#f878f8", "pattern diamonds")],
        series=["#3cbcfc", "#f8b800", "#f83800", "#58d854", "#d878f8", "#00e8d8", "#fca044", "#bcbcbc"],
        weights=dict(display=400, title=400, heading=400, body=400, label=400, caption=400, numeric=400),
        tracking=dict(display=0.04, title=0.02, label=0.02),
        sizes=dict(xs=16, sm=24, md=32, lg=48, xl=64, xxl=96, touch=44),
        radius=dict(xs=0, sm=0, md=0, lg=0, xl=0, pill=0),
        stroke=dict(hairline=2, thin=2, regular=2, thick=4, focus=2),
        # Hard, offset shadows on the pixel grid.
        elevation=dict(raised=[(4, 4, 0, 0, "#00000080")], floating=[(3, 3, 0, 0, "#00000080")],
                       overlay=[(6, 6, 0, 0, "#000000b0")], top=[(8, 8, 0, 0, "#000000c0")]),
        shape=dict(panel="square 0", dialog="square 0", card="square 0", tooltip="square 0", button="square 0",
                   field="square 0", chip="notch 2", badge="notch 2", tab="square 0", slot="square 0",
                   bar="square 0", hud="square 0", avatar="notch 4"),
        springs=dict(ui=(100, 0), panel=(160, 0), emphasis=(240, 0), playful=(320, 0.1)),
        # Motion held on steps, as hand-timed pixel animation is.
        eases=dict(standard="linear", enter="steps 4", exit="steps 3", emphasis="steps 6"),
        stagger_cap=0, overshoot=0.05,
        haptics=dict(HAPTICS, revealLegendary="thud", typewriter="tick"),
        sounds=dict(typewriter="pitch 0.9 1.1", counterTick="pitch 1 1.5"),
    ),
    "cartoon": dict(
        # Candy-bright tiers; the reveal of each lands harder.
        status=dict(success='icon "check"', warn='icon "flag"', danger='icon "close"', info="pattern dots"),
        rarity=[("#9a9aa8", "pips 1"), ("#2fbf4a", "pips 2"), ("#2e8bff", "pips 3"), ("#b44cff", "pips 4"),
                ("#ff9a1f", "pips 5"), ("#ff3d7f", "pips 6")],
        teams=["#2e8bff", "#ff4d4d", "#2fbf4a", "#ffc21a", "#b44cff", "#ff8a1f", "#1fc7c7", "#ff5cb0"],
        damage=[("#a08a70", 'icon "sword"'), ("#ff6a2b", 'icon "fire"'), ("#45c8ff", 'icon "drop"'),
                ("#ffd21f", 'icon "bolt"'), ("#7ed957", "pattern dots"), ("#ffe58a", 'icon "star"'),
                ("#7a5cf6", "pattern waves"), ("#ff4fd8", "pattern stars")],
        series=["#2e8bff", "#ff8a1f", "#ff4d6a", "#2fbf4a", "#b44cff", "#ffc21a", "#1fc7c7", "#ff5cb0"],
        weights=dict(display=400, title=400, heading=600, body=600, label=600, caption=600, numeric=400),
        tracking=dict(display=0.01, title=0.01),
        sizes=dict(xs=16, sm=24, md=36, lg=52, xl=72, xxl=104, touch=48),
        radius=dict(xs=4, sm=6, md=10, lg=14, xl=20, pill=9999),
        stroke=dict(hairline=2, thin=3, regular=4, thick=5, focus=3),
        # A thick ink drop shadow straight down, the panel's weight.
        elevation=dict(raised=[(0, 5, 0, 0, "#2b1a4a99")], floating=[(0, 4, 0, 0, "#2b1a4a99")],
                       overlay=[(0, 8, 0, 0, "#2b1a4acc"), (0, 18, 24, 0, "#1a0f3c40")],
                       top=[(0, 10, 0, 0, "#2b1a4acc"), (0, 24, 32, 0, "#1a0f3c4d")]),
        shape=dict(panel="round 10", dialog="round 14", card="round 12", tooltip="round 8", button="squircle 6",
                   field="squircle 6", chip="round pill", badge="round pill", tab="squircle 6", slot="round 6",
                   bar="round 6", hud="round 8", avatar="round pill"),
        springs=dict(ui=(160, 0.15), panel=(320, 0.25), emphasis=(480, 0.4), playful=(560, 0.5)),
        eases=dict(standard="0.2 0 0 1", enter="0.34 1.56 0.64 1", exit="0.36 0 0.66 -0.56", emphasis="0.34 1.8 0.64 1"),
        stagger_cap=400, overshoot=0.2,
        haptics=dict(HAPTICS, reward="0 20 0.2 0.6 80 20 0.2 0.7 160 40 0.4 0.9", revealMythic="thud"),
        sounds=dict(press="volume 0.8 1 pitch 0.95 1.08", counterTick="pitch 1 1.6", reward="pitch 1 1.25"),
    ),
    "fantasy": dict(
        # Jewel tones of an illuminated manuscript; legendary is gilt, mythic crimson.
        status=dict(success='icon "check"', warn='icon "flag"', danger='icon "skull"', info='icon "map"'),
        rarity=[("#8a8170", "pips 1"), ("#3f8a2f", "pips 2"), ("#2a5fa8", "pips 3"), ("#7d3aa0", "pips 4"),
                ("#b8781a", 'icon "crown"'), ("#a8182c", 'icon "gem"')],
        teams=["#2a5fa8", "#a8182c", "#3f8a2f", "#b8781a", "#7d3aa0", "#c0561a", "#1f7f7a", "#a83a7a"],
        damage=[("#7a6a55", 'icon "sword"'), ("#c0461a", 'icon "fire"'), ("#2f7fb0", 'icon "drop"'),
                ("#a8861a", 'icon "bolt"'), ("#4f7a1f", 'icon "potion"'), ("#b8941a", 'icon "star"'),
                ("#4a2a6a", 'icon "skull"'), ("#8a2a9a", "pattern stars")],
        series=["#2a5fa8", "#b8781a", "#a8182c", "#3f8a2f", "#7d3aa0", "#1f7f7a", "#c0561a", "#6a5a48"],
        weights=dict(display=700, title=700, heading=700, body=500, label=500, caption=500, numeric=700),
        tracking=dict(display=0.06, title=0.05, heading=0.03, numeric=0.02),
        sizes=dict(xs=16, sm=24, md=32, lg=48, xl=64, xxl=96, touch=44),
        radius=dict(xs=2, sm=4, md=6, lg=8, xl=12, pill=9999),
        stroke=dict(hairline=1, thin=1.5, regular=2, thick=3, focus=2),
        elevation=dict(raised=[(0, 4, 8, 0, "#00000066")], floating=[(0, 3, 6, 0, "#00000080")],
                       overlay=[(0, 10, 18, 2, "#000000b3")], top=[(0, 14, 28, 2, "#000000c0")]),
        shape=dict(panel="round 4", dialog="round 6", card="round 4", tooltip="notch 4", button="round 4",
                   field="round 4", chip="bevel 4", badge="notch 6", tab="round 4", slot="bevel 4",
                   bar="round 4", hud="round 8", avatar="round pill"),
        springs=dict(ui=(220, 0), panel=(360, 0.05), emphasis=(520, 0.15), playful=(600, 0.2)),
        eases=dict(standard="0.25 0.1 0.25 1", enter="0.16 1 0.3 1", exit="0.7 0 0.84 0", emphasis="0.22 1.2 0.36 1"),
        stagger_cap=420, overshoot=0.05,
        haptics=dict(HAPTICS, open="tap", close="tick", revealLegendary="thud", revealMythic="thud"),
        sounds=dict(press="volume 0.85 1 pitch 0.97 1.03", counterTick="pitch 1 1.3"),
    ),
    "scifi": dict(
        # Mission control: calm status colours, cues as plain marks.
        status=dict(success='icon "check"', warn="pattern stripes", danger="pattern chevrons", info="pattern grid"),
        rarity=[("#8a96a3", "pips 1"), ("#5fd38a", "pips 2"), ("#9ccdf5", "pips 3"), ("#b49cf0", "pips 4"),
                ("#e8c46a", "pips 5"), ("#d0584c", "pips 6")],
        teams=["#9ccdf5", "#d0584c", "#5fd38a", "#e8c46a", "#b49cf0", "#e89a5a", "#6ad6c8", "#e88ab8"],
        damage=[("#a8b2bd", "pattern hatch"), ("#e8845a", 'icon "fire"'), ("#9ccdf5", 'icon "drop"'),
                ("#e8d06a", 'icon "bolt"'), ("#9fd36a", "pattern dots"), ("#f4ecc8", 'icon "star"'),
                ("#a896f0", "pattern grid"), ("#e88ab8", "pattern diamonds")],
        series=["#9ccdf5", "#e8c46a", "#d0584c", "#5fd38a", "#b49cf0", "#e89a5a", "#6ad6c8", "#a8b2bd"],
        weights=dict(display=300, title=300, heading=400, body=400, label=400, caption=400, numeric=400),
        tracking=dict(display=0.0, title=0.0, heading=0.01, label=0.08, caption=0.04),
        sizes=dict(xs=16, sm=24, md=32, lg=48, xl=64, xxl=96, touch=44),
        radius=dict(xs=1, sm=2, md=3, lg=4, xl=6, pill=9999),
        stroke=dict(hairline=1, thin=1, regular=1.5, thick=2, focus=2),
        # A soft falloff under floating glass, no coloured rim.
        elevation=dict(raised=[(0, 4, 12, 0, "#00000066")], floating=[(0, 8, 24, 0, "#00000080")],
                       overlay=[(0, 12, 32, 0, "#00000099")], top=[(0, 16, 40, 0, "#000000aa")]),
        shape=dict(panel="round 4", dialog="round 6", card="round 4", tooltip="round 3", button="round 3",
                   field="round 3", chip="round 3", badge="round 2", tab="round 3", slot="round 2",
                   bar="round 2", hud="round 3", avatar="round pill"),
        springs=dict(ui=(160, 0), panel=(260, 0), emphasis=(320, 0.05), playful=(360, 0.08)),
        eases=dict(standard="0.4 0 0.2 1", enter="0 0 0.2 1", exit="0.4 0 1 1", emphasis="0.2 0 0 1"),
        stagger_cap=240, overshoot=0.0,
        haptics=dict(HAPTICS, focus="tick", scrollTick="tick", typewriter="tick", counterTick="tick"),
        sounds=dict(focus="volume 0.5 0.6", typewriter="volume 0.4 0.5 pitch 0.95 1.05", counterTick="pitch 1 1.4"),
    ),
}


def tokens_lines(s):
    """The version 2 token blocks of a style (everything but motion and sounds)."""
    t = TOKENS[s["name"]]
    p = s["palette"]
    raised, a = _rgba(p["panelRaised"])
    ground = _over(raised, a, _rgba(p["backdrop"])[0])
    L = ["colors {"]
    for k, cue in t["status"].items():
        L.append(f"  {k} {cue}")
    for k, (c, cue) in zip(RARITY, t["rarity"]):
        L.append(f'  {k} "{readable(c, ground)}" {cue}')
    for i, c in enumerate(t["teams"]):
        L.append(f'  team{i + 1} "{readable(c, ground)}"')
    for k, (c, cue) in zip(DAMAGE, t["damage"]):
        L.append(f'  {k} "{readable(c, ground)}" {cue}')
    for i, c in enumerate(t["series"]):
        L.append(f'  series{i + 1} "{readable(c, ground)}"')
    L.append("}")
    ramp = s["ramp"]
    fonts = dict(display="display", title="display", heading="display", body="body", label="body", caption="body",
                 numeric="numeric")
    steps = dict(display="displayLg", title="displaySm", heading="xl", body="body", label="body", caption="caption",
                 numeric="lg")
    L.append("typography {")
    for role, step in steps.items():
        size, lh = ramp[step]
        head = f"  {role} {size} {lh} {t['tracking'].get(role, 0)} {t['weights'][role]} {fonts[role]}"
        # Phones take display steps down: a title screen's word still fits a 360 px column.
        if role in ("display", "title") and size >= 28:
            k = 0.8 if role == "display" else 0.875
            pp = (max(12, round(size * k)), max(16, round(lh * k)))
            pl = (max(12, round(size * (1 + k) / 2)), max(16, round(lh * (1 + k) / 2)))
            L += [head + " {", f"    phonePortrait {pp[0]} {pp[1]}", f"    phoneLandscape {pl[0]} {pl[1]}", "  }"]
        else:
            L.append(head)
    L.append("}")
    for key, vals in (("space", SPACE), ("size", t["sizes"]), ("radius", t["radius"]), ("stroke", t["stroke"])):
        L += [key + " {"] + [f"  {k} {v}" for k, v in vals.items()] + ["}"]
    L.append("elevation {")
    for lvl, shadows in t["elevation"].items():
        L += [f"  {lvl} {{"] + [f'    shadow {dx} {dy} {b} {sp} "{c}"' for dx, dy, b, sp, c in shadows] + ["  }"]
    L.append("}")
    assert list(t["shape"]) == CLASSES
    r = s["radius"]
    # The classes version 1 gave a radius keep it (the widgets read these).
    for cls, v1 in (("slot", "small"), ("button", "control"), ("bar", "bar"), ("panel", "panel")):
        assert float(t["shape"][cls].split()[1]) == r[v1], (s["name"], cls)
    L += ["shape {"] + [f"  {k} {v}" for k, v in t["shape"].items()] + ["}"]
    return L


def motion_lines(s):
    t = TOKENS[s["name"]]
    m = s["motion"]
    d = dict(DURATIONS, **s["durations"])
    L = ["motion {", f"  enter {m['enter']}", f"  toast {m['toast']}", f"  progress {m['progress']}",
         f"  shake {m['shake']}", f"  pop {m['pop']}"]
    L += [f"  spring {k} {dur} {b}" for k, (dur, b) in t["springs"].items()]
    L += [f"  ease {k} {v}" for k, v in t["eases"].items()]
    L += [f"  duration {k} {d[k]}" for k in ("instant", "fast", "base", "slow", "spring", "bounce")]
    L += [f"  stagger {m['stagger']}", f"  staggerCap {t['stagger_cap']}", f"  overshoot {t['overshoot']}", "}"]
    return L


def sounds_lines(s):
    """Every UI event of docs/ui.md §7.8: the built-ins ship no UI sounds of
    their own (a game hands og.ui.configure its own), so each is `none` with
    the volume and pitch its set will play at, and the style's haptic."""
    t = TOKENS[s["name"]]
    L = ["sounds {"]
    for e in EVENTS:
        extra = t["sounds"].get(e)
        L.append(f"  {e} none{' ' + extra if extra else ''} haptic {t['haptics'].get(e, 'none')}")
    L.append("}")
    return L


# ── output ──────────────────────────────────────────────────────────────────

def to8(img):
    return (np.clip(img, 0, 1) * 255 + 0.5).astype(np.uint8)


def build_atlas(parts):
    atlas = np.zeros((ATLAS_H, ATLAS_W, 4), dtype=np.uint8)
    for name, (x, y, w, h) in LAYOUT.items():
        if name not in parts:
            continue  # a pack without that ornament leaves its region empty
        img = to8(parts[name])
        assert img.shape[:2] == (h, w), (name, img.shape, (w, h))
        # Edge texels repeat 2 px into the gutter (no neighbour bleeds in).
        padded = np.pad(img, ((2, 2), (2, 2), (0, 0)), mode="edge")
        atlas[y - 2:y + h + 2, x - 2:x + w + 2] = padded
    return Image.fromarray(atlas, "RGBA")


def region(name):
    x, y, w, h = LAYOUT[name]
    return f"{x} {y} {w} {h}"


# The version 1 parts as version 2 dressings (og-ui's StylePart::selector).
SELECTORS = dict(panel="Panel.frame", card="Card.frame", button="Button.base", buttonHover="Button.base[hover]",
                 buttonPressed="Button.base[pressed]", buttonDisabled="Button.base[disabled]", field="Field.frame",
                 slot="Slot.frame", tooltip="Tooltip.frame", hud="Hud.frame", track="Meter.track", fill="Meter.fill")


def style_text(s, d, parts=None):
    n = s["name"]
    L = [
        "type style",
        "version 2",
        "---",
        f"# The {n} style pack: generated by templates/ui-kit/tools/make-styles.py (CC0 art, OFL fonts; LICENSES.md).",
        f'name "{n}"',
        f"scheme {s['base']}",
        f'atlas "atlas.png" {ATLAS_W} {ATLAS_H}',
        f"scale {s['scale']}",
        "palette {",
    ]
    L += [f'  {k} "{v}"' for k, v in s["palette"].items()]
    L += ["}"]
    L += tokens_lines(s)
    L += ["ramp {"] + [f"  {k} {a} {b}" for k, (a, b) in s["ramp"].items()] + ["}"]
    L += [
        "fonts {",
        f'  body "{s["fonts"]["body"]}"',
        f'  display "{s["fonts"]["display"]}"',
        f'  numeric "{s["fonts"]["numeric"]}"',
        # Chinese in the style's voice: the shared companions (templates/styles/cjk).
        f'  cjk "@styles/cjk/{s["fonts"]["cjk"]}"',
        f'  cjkDisplay "@styles/cjk/{s["fonts"]["cjkDisplay"]}"',
        "}",
        f'icons "icons.png" 8 4 {{',
        "  names " + " ".join(f'"{i}"' for i in ICONS),
        "}",
    ]
    L.append(f"soft {region('soft')} 24")
    if s.get("corner"):
        w, h, ink, off = s["corner"]
        L.append(f"corner {region('corner')} {w} {h} {ink} {off}")
    if s.get("divider"):
        w, h, ink = s["divider"]
        L.append(f"divider {region('divider')} {w} {h} {ink}")
    if s.get("flourish"):
        w, h, ink, off = s["flourish"]
        L.append(f"flourish {region('flourish')} {w} {h} {ink} {off}")
    if s.get("crest"):
        w, h, ink, off = s["crest"]
        L.append(f"crest {region('crest')} {w} {h} {ink} {off}")
    if s.get("edge"):
        w, h, ink, off = s["edge"]
        L.append(f"edge {region('edge')} {w} {h} {ink} {off}")
    tiled = []
    if s.get("tile"):
        w, h, a, inset, tiled = s["tile"]
        L.append(f"tile {region('tile')} {w} {h} {a} {inset}")
    for role, dx, dy, blur, c in s["text"].get("textShadow", []):
        L.append(f'textShadow {role} {dx} {dy} {blur} "{c}"')
    for role, size, ink, a in s["text"].get("textGlow", []):
        L.append(f"textGlow {role} {size} {ink} {a}")
    for role, w in s["text"].get("textWeight", []):
        L.append(f"textWeight {role} {w}")
    L += motion_lines(s)
    L += sounds_lines(s)
    # One dressing per part: its frame, then (on a class) its shadow, glow,
    # glass, corner ornaments and tile, and the button's press and hover.
    for part, spec in s["slices"].items():
        border = spec[0]
        mode = spec[1] if len(spec) > 1 else "tint"
        D = [f"  slice {region(part)} {border} {mode}"]
        # A frame whose centre is not white and opaque: its mean, linear
        # RGB and alpha, so og-ui picks label ink that reads on what it
        # draws (og cook measures it the same way for a style without it).
        if parts is not None:
            c = to8(parts[part])[border:-border or None, border:-border or None] / 255.0
            lin = np.where(c[..., :3] <= 0.04045, c[..., :3] / 12.92, ((c[..., :3] + 0.055) / 1.055) ** 2.4)
            a = c[..., 3:4]
            rgb = (lin * a).reshape(-1, 3).sum(0) / max(float(a.sum()), 1e-6)
            alpha = float(a.mean())
            if min(*rgb, alpha) < 0.98:
                D.append(f"  tone {rgb[0]:.3f} {rgb[1]:.3f} {rgb[2]:.3f} {alpha:.2f}")
        if part in s["shadows"]:
            dx, dy, blur, spread, c = s["shadows"][part]
            D.append(f'  shadow {dx} {dy} {blur} {spread} "{c}"')
        if part in s.get("glows", {}):
            size, ink, a = s["glows"][part]
            D.append(f"  glow {size} {ink} {a}")
        if part in s.get("glass", {}):
            blur, fill, tint = s["glass"][part]
            D.append(f"  glass {blur} {fill} {tint}")
        if s.get("corner") and part in s.get("corners", []):
            D.append("  corners")
        if s.get("crest") and part in s.get("crests", []):
            D.append("  crest")
        if s.get("edge") and part in s.get("edges", []):
            D.append("  edges")
        if part in tiled:
            D.append("  tile")
        if part == "button":
            D.append("  press {} {} {}".format(*s["press"]))
            D.append("  hover {} {}".format(*s["hover"]))
        L += [f'dress "{SELECTORS[part]}" {{'] + D + ["}"]
    # Parts no atlas slice draws (a toggle's knob by state): their lines as given.
    for sel, lines in s.get("dresses", {}).items():
        L += [f'dress "{sel}" {{'] + [f"  {l}" for l in lines] + ["}"]
    return "\n".join(L) + "\n"


def font_import(path, display, glows=False, weighted=False):
    em = 48 if display else 32
    # The marks the kit draws beyond Latin-1 and punct (minus, arrows), as
    # far as the font maps them (fontTools reads its cmap).
    from fontTools.ttLib import TTFont
    cmap = TTFont(path).getBestCmap()
    marks = "".join(c for c in "\u2212\u2190\u2192" if ord(c) in cmap)
    # A role with a textGlow reads its glow from the distance field, which
    # reaches half px_range atlas pixels: 12 lets it spread 6 (4 lets 2). A
    # textWeight thickens inside that reach less 0.75 px (og-render's
    # overlay.wgsl): at 16 px from a 48 px em, 8 allows 0.58 px, 4 none.
    rng = 12 if glows else 8 if weighted else 4
    why = "; px_range 12 for its text glow" if glows else "; px_range 8 for its textWeight" if weighted else ""
    return (
        "type import\nversion 1\n---\n"
        f"# Latin-1 and the typographic punctuation the kit shows; {em} px per em in the atlas{why}.\n"
        "font {\n  charset latin punct\n"
        + (f'  chars "{marks}"\n' if marks else "")
        + f"  em_px {em}\n  px_range {rng}\n}}\n"
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


# The built-in styles live beside the templates.
STYLES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "styles")


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "templates/ui-kit/assets"
    ui_sounds(out)
    for pack in PACKS:
        parts, s = pack()
        d = os.path.join(STYLES, s["name"])
        os.makedirs(d, exist_ok=True)
        # A painted pack (art/, docs/art.md §2.7) is og style art's: its
        # procedural atlas becomes the base the painted pieces pack below,
        # and `og style art <pack>` rewrites its atlases and style file.
        if os.path.isdir(os.path.join(d, "art")):
            build_atlas(parts).save(os.path.join(d, "art", "base.png"), optimize=True)
            icon_sheet(s["icons"]).save(os.path.join(d, "icons.png"), optimize=True)
            print("wrote", os.path.join(d, "art", "base.png"), "(painted: now og style art", d + ")")
            continue
        build_atlas(parts).save(os.path.join(d, "atlas.png"), optimize=True)
        icon_sheet(s["icons"]).save(os.path.join(d, "icons.png"), optimize=True)
        with open(os.path.join(d, "style.ogt"), "w") as f:
            f.write(style_text(s, d, parts))
        for name in ("atlas.png", "icons.png"):
            with open(os.path.join(d, name + ".ogt"), "w") as f:
                f.write("type import\nversion 1\n---\n# Sharp art: UASTC.\ntexture {\n  kind "
                        + ("linear" if name == "icons.png" else "color") + "\n}\n")
        roles = s["fonts"]
        imports = {}
        for role in ("body", "display", "numeric"):
            imports[roles[role]] = imports.get(roles[role], False) or role != "body"
        glowing = {roles[role] for role, *_ in s["text"].get("textGlow", [])}
        weighted = {roles[role] for role, _ in s["text"].get("textWeight", [])}
        for name, display in imports.items():
            with open(os.path.join(d, name + ".ogt"), "w") as f:
                f.write(font_import(os.path.join(d, name), display, name in glowing, name in weighted))
        print("wrote", d)
    # The icon sheets the seed styles share (templates/styles/shared), from
    # the paper and neon packs folded into storybook and cyber (2026-11-02).
    shared = os.path.join(STYLES, "shared")
    for variant, name in (("paper", "ink-icons.png"), ("neon", "glow-icons.png")):
        icon_sheet(variant).save(os.path.join(shared, name), optimize=True)
        print("wrote", os.path.join(shared, name))


if __name__ == "__main__":
    main()
