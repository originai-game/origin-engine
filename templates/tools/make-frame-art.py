#!/usr/bin/env python3
"""Draws the packs' nine-slice frame art (CC0), byte for byte: the default
pack's in templates/styles/shared/art/frames/ and scifi's in
templates/styles/scifi/art/frames/ and cartoon's in
templates/styles/cartoon/art/frames/;
and the UI effects' particle textures in templates/styles/shared/fx/.

    python3 templates/tools/make-frame-art.py [repo]

Needs numpy and Pillow. Every piece is built from signed distance fields at
4 px per point (the packs' art scale): a drop shadow, a dark outline, a lit
rim whose brightness follows the edge's normal against a light from above,
a face in a vertical gradient with an inner highlight, a gloss and an inner
shade, and, on buttons, a lip under the face that a press flattens. The
nine-slice insets each image needs are in FRAMES (points); the pack entry
uses them times 4 as `slice` and as is as `width`. Along a stretched edge
every row or column is the same, so the art stretches cleanly.

Tint pieces (`tint`) are grey shading with alpha, which the element's fill
multiplies: one bevel takes any colour the pack or a rarity gives it.
"""
import os, sys
import numpy as np
from PIL import Image, ImageFilter

S = 4  # pixels per point


def hexc(h, a=1.0):
    h = h.lstrip("#")
    if len(h) == 8:
        a = int(h[6:], 16) / 255
    return np.array([int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)] + [a])


class Canvas:
    def __init__(self, w, h):
        self.w, self.h = w, h
        ys, xs = np.mgrid[0:h * S, 0:w * S]
        self.x = (xs + 0.5) / S
        self.y = (ys + 0.5) / S
        self.rgba = np.zeros((h * S, w * S, 4))

    def over(self, color, alpha):
        """Lays `color` (rgba, or a per-pixel array) over the canvas at coverage `alpha`."""
        c = np.broadcast_to(np.asarray(color, float), self.rgba.shape)
        a = np.clip(alpha, 0, 1)[..., None] * c[..., 3:4]
        base = self.rgba
        out_a = a + base[..., 3:4] * (1 - a)
        rgb = (c[..., :3] * a + base[..., :3] * base[..., 3:4] * (1 - a)) / np.maximum(out_a, 1e-6)
        self.rgba = np.concatenate([rgb, out_a], axis=-1)

    def save(self, path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        px = np.round(np.clip(self.rgba, 0, 1) * 255).astype(np.uint8)
        Image.fromarray(px, "RGBA").save(path, optimize=True)


def shape(cv, box, radius=0.0, cuts=(), cut=0.0):
    """Signed distance (points) to a rounded box, its corners in `cuts`
    ("tl", "tr", "br", "bl") chamfered by `cut` instead."""
    x0, y0, x1, y1 = box
    cx, cy, hw, hh = (x0 + x1) / 2, (y0 + y1) / 2, (x1 - x0) / 2, (y1 - y0) / 2
    px, py = cv.x - cx, cv.y - cy
    r = min(radius, hw, hh)
    qx, qy = np.abs(px) - hw + r, np.abs(py) - hh + r
    d = np.hypot(np.maximum(qx, 0), np.maximum(qy, 0)) + np.minimum(np.maximum(qx, qy), 0) - r
    for c in cuts:
        sx = -1 if "l" in c else 1
        sy = -1 if "t" in c else 1
        d = np.maximum(d, (sx * px + sy * py - (hw + hh - cut)) / np.sqrt(2))
    return d


def cover(d):
    """Antialiased coverage of the inside of a distance field."""
    return np.clip(0.5 - d * S, 0, 1)


def lit(cv, d):
    """How much an edge faces the light (above, a little left): 0 to 1."""
    gy, gx = np.gradient(d)
    n = np.hypot(gx, gy) + 1e-9
    return np.clip(0.5 + 0.5 * (-(gy / n) * 0.92 - (gx / n) * 0.38), 0, 1)[..., None]


def vgrad(cv, top, bottom, y0, y1):
    t = np.clip((cv.y - y0) / max(y1 - y0, 1e-6), 0, 1)[..., None]
    return top * (1 - t) + bottom * t


def shadow(cv, d, dy, blur, alpha, color="#000000"):
    a = cover(d)
    img = Image.fromarray(np.round(a * 255).astype(np.uint8), "L")
    img = img.transform(img.size, Image.AFFINE, (1, 0, 0, 0, 1, -dy * S))
    img = img.filter(ImageFilter.GaussianBlur(blur * S))
    cv.over(hexc(color), np.asarray(img) / 255 * alpha)


def glow(cv, d, blur, alpha, color):
    """A halo of `color` outside the shape, its blurred silhouette."""
    a = cover(d)
    img = Image.fromarray(np.round(a * 255).astype(np.uint8), "L").filter(ImageFilter.GaussianBlur(blur * S))
    cv.over(hexc(color), np.asarray(img) / 255 * alpha * (1 - cover(d)))


def plate(cv, d, face_top, face_bottom, y0, y1, rim_lo, rim_hi, rim, outline, outline_w=1.0,
          gloss=0.16, highlight=0.5, inner_shade=0.25):
    """A raised piece: outline, a lit rim, a gradient face, its highlight, gloss and inner shade."""
    if outline is not None:
        cv.over(hexc(outline), cover(d - outline_w))
    L = lit(cv, d)
    rim_c = hexc(rim_lo) * (1 - L) + hexc(rim_hi) * L
    cv.over(rim_c, cover(d))
    face = shape_inset(d, rim)
    cv.over(vgrad(cv, hexc(face_top), hexc(face_bottom), y0, y1), cover(face))
    if highlight:
        hl = np.clip(cover(face) - cover(face + 1.0), 0, 1) * np.clip((L[..., 0] - 0.55) * 2.2, 0, 1)
        cv.over(hexc("#ffffff"), hl * highlight)
    if gloss:
        mid = y0 + (y1 - y0) * 0.5
        g = cover(face + 1.5) * np.clip((mid - cv.y) / max(mid - y0, 1e-6), 0, 1) ** 0.7
        cv.over(hexc("#ffffff"), g * gloss)
    if inner_shade:
        sh = cover(face) * np.clip((cv.y - (y1 - (y1 - y0) * 0.3)) / ((y1 - y0) * 0.3), 0, 1)
        cv.over(hexc("#000000"), sh * inner_shade)
    return face


def shape_inset(d, by):
    return d + by


def mix(a, b, t):
    return "#" + "".join(f"{round((int(a[i:i+2],16)*(1-t)+int(b[i:i+2],16)*t)):02x}" for i in (1, 3, 5))


# ---------------------------------------------------------------------------
# The default pack: clean, polished and soft-cornered (the RPG menus of
# Genshin Impact and Honkai: Star Rail): slate glass panels with a
# champagne rim, chunky glossy buttons with a lip, dark currency plates.

GOLD_RIM = ("#8c7650", "#f3e3bd")


def button(cv, face, lip, rim, outline, state, radius, cuts=(), cut=0.0, glow_c=None):
    W, H = cv.w, cv.h
    m, lip_h = 2.0, 4.0
    if state == "pressed":
        top, lip_h = m + 2.5, 1.5
    else:
        top = m
    body = shape(cv, (m, top, W - m, H - m), radius, cuts, cut)
    fbox = (m, top, W - m, H - m - lip_h)
    fd = shape(cv, fbox, radius, cuts, cut)
    shadow(cv, body, 1.5 if state != "pressed" else 0.6, 1.4, 0.45)
    if glow_c:
        glow(cv, body, 2.4, 0.9 if state == "hover" else 0.6, glow_c)
    cv.over(hexc(outline), cover(body - 1.0))
    cv.over(vgrad(cv, hexc(lip[0]), hexc(lip[1]), fbox[3] - 4, H - m), cover(body))
    bright = 0.1 if state == "hover" else 0.0
    ft, fb = (mix(face[0], "#ffffff", bright), mix(face[1], "#ffffff", bright))
    plate(cv, fd, ft, fb, fbox[1], fbox[3], rim[0], rim[1], 1.2, None,
          gloss=0.22 if state != "pressed" else 0.1, highlight=0.6 if state != "pressed" else 0.25,
          inner_shade=0.18)


DEFAULT_BUTTONS = {
    "button": dict(face=("#5b6886", "#323b52"), lip=("#1f2536", "#151a26"), rim=("#262d40", "#a9b6d6"), outline="#0a0d15"),
    "button-primary": dict(face=("#ffe08a", "#f2a92e"), lip=("#b8661c", "#8a4510"), rim=("#c47a1e", "#fff6cf"), outline="#3d2206"),
    "button-danger": dict(face=("#d8404c", "#961424"), lip=("#7a1020", "#5a0a16"), rim=("#8a1c2c", "#ffc2ba"), outline="#2e0610"),
    "button-disabled": dict(face=("#4c525e", "#363a44"), lip=("#24272e", "#1b1d22"), rim=("#30343c", "#7c828f"), outline="#0c0d10"),
}


def default_art(out):
    made = {}
    # Buttons: 96 × 52, rounded 14, a 4-point lip.
    for name, p in DEFAULT_BUTTONS.items():
        states = ["", "-hover", "-pressed"] if name != "button-disabled" else [""]
        for st in states:
            cv = Canvas(96, 52)
            button(cv, p["face"], p["lip"], p["rim"], p["outline"], st[1:] or "rest", 14)
            made[f"{name}{st}"] = cv
    # Panel: slate glass, a champagne rim, an inner hairline with gold corner ticks.
    cv = Canvas(128, 128)
    body = shape(cv, (4, 3, 124, 123), 16)
    shadow(cv, body, 2.0, 3.0, 0.55)
    cv.over(hexc("#06080e"), cover(body - 1.0))
    L = lit(cv, body)
    cv.over(hexc(GOLD_RIM[0]) * (1 - L) + hexc(GOLD_RIM[1]) * L, cover(body))
    face = body + 2.0
    cv.over(vgrad(cv, hexc("#2a3550"), hexc("#151b2b"), 5, 121), cover(face))
    cv.over(hexc("#ffffff"), (cover(face) - cover(face + 1.0)) * np.clip((L[..., 0] - 0.55) * 2, 0, 1) * 0.35)
    cv.over(hexc("#9fc2ff"), cover(face) * np.clip((22 - cv.y) / 17, 0, 1) * 0.10)
    hair = shape(cv, (10, 9, 118, 117), 10)
    cv.over(hexc("#e9d6a8"), (cover(hair + 0.0) - cover(hair + 0.75)) * 0.28)
    corner = np.zeros_like(hair)
    for (cx, cy) in [(10, 9), (118, 9), (118, 117), (10, 117)]:
        near = (np.abs(cv.x - cx) < 16) & (np.abs(cv.y - cy) < 16)
        corner = np.maximum(corner, near)
    cv.over(hexc("#f6e2b0"), (cover(hair + 0.0) - cover(hair + 1.25)) * corner * 0.9)
    made["panel"] = cv
    # Plate: a dark glass pill for currencies and HUD read-outs.
    cv = Canvas(96, 36)
    body = shape(cv, (2, 2, 94, 34), 16)
    shadow(cv, body, 1.0, 1.2, 0.4)
    plate(cv, body, "#1c2333", "#0b0f18", 2, 34, "#0a0d15", "#7d8aa8", 1.1, "#05070c", 0.9,
          gloss=0.10, highlight=0.45, inner_shade=0.0)
    made["plate"] = cv
    # Field: an inset well.
    cv = Canvas(96, 44)
    body = shape(cv, (2, 2, 94, 42), 10)
    cv.over(hexc("#ffffff"), (cover(body - 1.0) - cover(body)) * np.clip((cv.y - 30) / 12, 0, 1) * 0.18)
    cv.over(hexc("#04060b"), cover(body))
    face = body + 1.0
    cv.over(vgrad(cv, hexc("#0b0f18"), hexc("#161c2a"), 3, 41), cover(face))
    cv.over(hexc("#000000"), cover(face) * np.clip((12 - cv.y) / 9, 0, 1) * 0.45)
    made["field"] = cv
    # Slot: a sunk square with a lit metal rim.
    cv = Canvas(64, 64)
    body = shape(cv, (2, 2, 62, 62), 12)
    shadow(cv, body, 1.2, 1.5, 0.45)
    cv.over(hexc("#06080e"), cover(body - 1.0))
    L = lit(cv, body)
    cv.over(hexc("#2a3144") * (1 - L) + hexc("#a2b0cf") * L, cover(body))
    face = body + 2.25
    cv.over(vgrad(cv, hexc("#0e121c"), hexc("#222a3d"), 4, 60), cover(face))
    cv.over(hexc("#000000"), cover(face) * np.clip((16 - cv.y) / 12, 0, 1) * 0.5)
    cv.over(hexc("#ffffff"), cover(face) * np.clip((cv.y - 44) / 16, 0, 1) * 0.06)
    made["slot"] = cv
    # Slot rim (tint): a bright ring with an inward glow, its centre open: a rarity's colour.
    cv = Canvas(64, 64)
    body = shape(cv, (2, 2, 62, 62), 12)
    ring = cover(body) - cover(body + 2.25)
    inner = np.clip(1 + (body + 2.25) / 7, 0, 1) * cover(body + 2.25)
    L = lit(cv, body)[..., 0]
    grey = 0.78 + 0.22 * L
    cv.over(np.stack([grey, grey, grey, np.ones_like(grey)], -1), np.clip(ring + inner * 0.45, 0, 1))
    made["slot-rim"] = cv
    # Chip (tint): a bevelled tag the fill colours.
    cv = Canvas(48, 28)
    body = shape(cv, (1, 1, 47, 27), 8)
    cv.over(np.array([0.16, 0.16, 0.16, 1.0]), cover(body))
    L = lit(cv, body)
    rim = 0.17 + 0.36 * L
    cv.over(np.concatenate([np.repeat(rim, 3, -1), np.ones_like(rim)], -1), cover(body + 1.0))
    face = body + 2.0
    cv.over(vgrad(cv, np.array([0.33, 0.33, 0.33, 1]), np.array([0.24, 0.24, 0.24, 1]), 3, 25), cover(face))
    made["chip-bevel"] = cv
    # Badge: a notification pill, red with a white rim.
    cv = Canvas(32, 28)
    body = shape(cv, (2, 1, 30, 26), 12.5)
    shadow(cv, body, 1.0, 0.9, 0.5)
    cv.over(hexc("#ffffff"), cover(body))
    face = body + 1.75
    cv.over(vgrad(cv, hexc("#d83834"), hexc("#a00c1a"), 3, 24), cover(face))
    cv.over(hexc("#ffffff"), cover(face + 1.2) * np.clip((11 - cv.y) / 7, 0, 1) * 0.35)
    made["badge"] = cv
    # Divider: a champagne line fading out at both ends, over a dark hairline.
    cv = Canvas(192, 8)
    ends = np.clip(np.minimum(cv.x, 192 - cv.x) / 80, 0, 1) ** 1.5
    line = np.clip(1 - np.abs(cv.y - 3.6) / 0.75, 0, 1)
    under = np.clip(1 - np.abs(cv.y - 5.0) / 0.6, 0, 1)
    halo = np.clip(1 - np.abs(cv.y - 3.6) / 3.0, 0, 1) ** 2
    cv.over(hexc("#000000"), under * ends * 0.45)
    cv.over(hexc("#f3dfae"), halo * ends * 0.18)
    cv.over(hexc("#f3dfae"), line * ends)
    made["divider"] = cv
    for name, cv in made.items():
        cv.save(os.path.join(out, f"{name}.png"))
    return made


# ---------------------------------------------------------------------------
# Scifi: precise and quiet (Honkai: Star Rail, Destiny 2, Zenless Zone
# Zero, Arknights). Dark translucent glass faces with a subtle lit top
# edge, one hairline stroke, a soft drop instead of a glow, one accent
# per piece (a short cyan bar on the leading edge or a notch in the top
# left corner), radii from one rule (6 on controls, 10 on panels, 4 on
# chips), and a flat lit-cyan primary with a light inner gradient.

ACCENT = "#5fe3ff"
HAIR = "#ffffff"


def glass(cv, d, top, bottom, y0, y1, hair_a=0.16, lit_a=0.22, alpha=0.88):
    """A glass face over `d`: a vertical gradient at `alpha`, a hairline
    around it, the top edge lit."""
    cv.over(vgrad(cv, hexc(top, alpha), hexc(bottom, alpha), y0, y1), cover(d))
    cv.over(hexc(HAIR), (cover(d) - cover(d + 1.0)) * hair_a)
    toplit = np.clip((y0 + 6 - cv.y) / 6, 0, 1)
    cv.over(hexc(HAIR), (cover(d) - cover(d + 1.0)) * toplit * lit_a)
    cv.over(hexc(HAIR), cover(d + 1.0) * np.clip((y0 + 10 - cv.y) / 10, 0, 1) * 0.05)


def notch(cv, x0, y0, size, color, alpha=1.0):
    """A small right triangle in a top left corner (the accent notch)."""
    a = ((cv.x - x0) + (cv.y - y0) < size) & (cv.x >= x0) & (cv.y >= y0)
    cv.over(hexc(color), a * alpha)


def edge_bar(cv, x, y0, y1, width, color, alpha=1.0):
    """A vertical accent bar on a leading edge, the same on every row it spans."""
    a = np.clip(np.minimum(cv.x - x, x + width - cv.x) * S + 0.5, 0, 1) * np.clip(np.minimum(cv.y - y0, y1 - cv.y) * S + 0.5, 0, 1)
    cv.over(hexc(color), a * alpha)


def scifi_button(cv, kind, state):
    W, H = cv.w, cv.h
    m = 2.0
    dy = 0.75 if state == "pressed" else 0.0
    body = shape(cv, (m, m + dy, W - m, H - m + dy), 6)
    shadow(cv, body, 1.5 - dy, 1.6, 0.5)
    if kind == "primary":
        top, bottom = {"rest": ("#8cf0ff", "#3cc9ec"), "hover": ("#a8f5ff", "#55d6f4"), "pressed": ("#5ad7f5", "#2aaed3")}[state]
        cv.over(vgrad(cv, hexc(top), hexc(bottom), m, H - m), cover(body))
        cv.over(hexc("#ffffff"), (cover(body + 1.0) - cover(body + 1.75)) * np.clip((m + dy + 5 - cv.y) / 5, 0, 1) * 0.75)
        cv.over(hexc("#0a4a60"), (cover(body) - cover(body + 0.75)) * 0.55)
    elif kind == "disabled":
        glass(cv, body, "#1d252d", "#151b21", m, H - m, hair_a=0.10, lit_a=0.08)
    else:
        lift = {"rest": 0.0, "hover": 0.08, "pressed": -0.04}[state]
        glass(cv, body, mix("#1a2a38", "#ffffff", max(lift, 0)), "#0b131b", m, H - m,
              hair_a=0.22 if state == "hover" else 0.16)
        edge_bar(cv, m + 2.5, m + dy + 13, H - m + dy - 13, 1.5, ACCENT, 1.0 if state != "pressed" else 0.7)


def scifi_art(out):
    made = {}
    for name, kind in [("button", "plain"), ("button-primary", "primary")]:
        for st in ["", "-hover", "-pressed"]:
            cv = Canvas(96, 52)
            scifi_button(cv, kind, st[1:] or "rest")
            made[f"{name}{st}"] = cv
    cv = Canvas(96, 52)
    scifi_button(cv, "disabled", "rest")
    made["button-disabled"] = cv
    # Panel: glass, a hairline, the lit top, a cyan notch and a short bar after it.
    cv = Canvas(128, 128)
    body = shape(cv, (4, 3, 124, 123), 10)
    shadow(cv, body, 2.5, 4.0, 0.55)
    glass(cv, body, "#14202c", "#0a1118", 3, 123, alpha=0.9)
    tab = np.clip(np.minimum(cv.x - 14, 38 - cv.x) * S + 0.5, 0, 1) * np.clip(np.minimum(cv.y - 3, 4.5 - cv.y) * S + 0.5, 0, 1)
    cv.over(hexc(ACCENT), tab)
    made["panel"] = cv
    # Plate: a glass read-out with the accent bar on its leading edge.
    cv = Canvas(96, 36)
    body = shape(cv, (2, 2, 94, 34), 6)
    shadow(cv, body, 1.0, 1.4, 0.45)
    glass(cv, body, "#16222e", "#0b1219", 2, 34)
    edge_bar(cv, 4.5, 10, 26, 1.5, ACCENT)
    made["plate"] = cv
    # Chip: small glass with a cyan hairline.
    cv = Canvas(48, 24)
    body = shape(cv, (2, 2, 46, 22), 4)
    glass(cv, body, "#163042", "#0d1d29", 2, 22, hair_a=0.0, lit_a=0.15)
    cv.over(hexc(ACCENT), (cover(body) - cover(body + 1.0)) * 0.55)
    made["chip"] = cv
    # Slot: sunk glass, a hairline and a notch in its top left corner.
    cv = Canvas(64, 64)
    body = shape(cv, (2, 2, 62, 62), 6)
    shadow(cv, body, 1.0, 1.4, 0.4)
    glass(cv, body, "#0a1118", "#14202b", 2, 62, hair_a=0.18, lit_a=0.0)
    cv.over(hexc("#000000"), cover(body + 1.0) * np.clip((12 - cv.y) / 10, 0, 1) * 0.35)
    corner = ((cv.x - 5) + (cv.y - 5) < 7) & (cv.x >= 5) & (cv.y >= 5)
    cv.over(hexc(ACCENT), corner * cover(body + 2.0))
    made["slot"] = cv
    # Divider: a hairline fading out at its ends, a short cyan segment in its middle.
    cv = Canvas(192, 8)
    ends = np.clip(np.minimum(cv.x, 192 - cv.x) / 72, 0, 1)
    line = np.clip(1 - np.abs(cv.y - 4) / 0.5, 0, 1)
    cv.over(hexc(HAIR), line * ends * 0.28)
    mid = np.clip(np.minimum(cv.x - 84, 108 - cv.x) * S + 0.5, 0, 1) * np.clip(1 - np.abs(cv.y - 4) / 0.9, 0, 1)
    cv.over(hexc(ACCENT), mid)
    made["divider"] = cv
    for name, cv in made.items():
        cv.save(os.path.join(out, f"{name}.png"))
    return made


# ---------------------------------------------------------------------------
# Cartoon: soft, chunky and confident (Brawl Stars, Clash Royale, Royal
# Match). A thick clean navy outline, warm two-tone faces split by a
# glossy top band, a deep 3D lip a press sinks into, and a palette past
# flat blue: sky-blue and sunny-yellow buttons, a warm cream panel under
# a tangerine header band, navy read-out plates.

INK = "#1c1a3d"

CARTOON_BUTTONS = {
    "button": dict(top="#7fd7ff", bottom="#3aa6f5", lip=("#1f6fc7", "#17559e")),
    "button-primary": dict(top="#ffe36b", bottom="#ffb51f", lip=("#d9781a", "#b05c10")),
    "button-danger": dict(top="#d63842", bottom="#b41c2c", lip=("#86121f", "#6a0c18")),
    "button-disabled": dict(top="#6c7088", bottom="#555a70", lip=("#3c4054", "#2e3142")),
}


def chunky(cv, box, radius, top, bottom, lip, lip_h, outline=2.5, band=0.46, drop=True):
    """A cartoon piece: an outline, a lip under the face, a two-tone face
    (lighter above its middle) and a glossy band across its top."""
    x0, y0, x1, y1 = box
    body = shape(cv, box, radius)
    if drop:
        shadow(cv, body, 1.5, 0.8, 0.35, INK)
    cv.over(hexc(INK), cover(body - outline))
    cv.over(vgrad(cv, hexc(lip[0]), hexc(lip[1]), y1 - lip_h - 2, y1), cover(body))
    fd = shape(cv, (x0, y0, x1, y1 - lip_h), radius)
    fy1 = y1 - lip_h
    split = y0 + (fy1 - y0) * 0.55
    face = np.where((cv.y < split)[..., None], hexc(top), vgrad(cv, hexc(mix(top, bottom, 0.55)), hexc(bottom), split, fy1))
    cv.over(face, cover(fd))
    # the band: a rounded white gloss inset in the top of the face
    gb = shape(cv, (x0 + 3.5, y0 + 2.5, x1 - 3.5, y0 + 2.5 + (fy1 - y0) * band * 0.62), max(radius - 4, 2))
    cv.over(hexc("#ffffff"), cover(gb) * vgrad(cv, np.array([1, 1, 1, 0.55]), np.array([1, 1, 1, 0.12]), y0 + 2.5, y0 + (fy1 - y0) * band)[..., 3])
    # a light rim inside the face's top, a darker one at its bottom
    cv.over(hexc("#ffffff"), (cover(fd) - cover(fd + 1.0)) * np.clip((y0 + 8 - cv.y) / 8, 0, 1) * 0.6)
    return fd


def cartoon_art(out):
    made = {}
    for name, p in CARTOON_BUTTONS.items():
        states = [""] if name == "button-disabled" else ["", "-hover", "-pressed"]
        for st in states:
            cv = Canvas(96, 56)
            pressed = st == "-pressed"
            bright = 0.14 if st == "-hover" else 0.0
            top = mix(p["top"], "#ffffff", bright)
            bottom = mix(p["bottom"], "#ffffff", bright)
            chunky(cv, (2.5, 2.5 + (4 if pressed else 0), 93.5, 52.5), 16, top, bottom, p["lip"], 2 if pressed else 6)
            made[f"{name}{st}"] = cv
    # Panel: a warm cream face under a tangerine header band, a deep lip.
    cv = Canvas(128, 128)
    box = (3, 3, 125, 123)
    body = shape(cv, box, 20)
    shadow(cv, body, 3, 2.0, 0.35, INK)
    cv.over(hexc(INK), cover(body - 3.0))
    cv.over(hexc("#d8c095"), cover(body))
    fd = shape(cv, (3, 3, 125, 118), 20)
    cv.over(vgrad(cv, hexc("#fff8ea"), hexc("#f6e6c8"), 30, 118), cover(fd))
    header = cover(fd) * (cv.y < 26)
    cv.over(vgrad(cv, hexc("#ffb347"), hexc("#ff8a24"), 3, 26), header)
    cv.over(hexc("#ffffff"), cover(shape(cv, (8, 6, 120, 13), 4)) * 0.35)
    cv.over(hexc("#c9611a"), header * np.clip(1 - np.abs(cv.y - 25.5) / 0.9, 0, 1))
    cv.over(hexc("#ffffff"), (cover(fd) - cover(fd + 1.25)) * (cv.y > 27) * 0.8)
    made["panel"] = cv
    # Plate: a navy read-out pill, outlined, a gloss band.
    cv = Canvas(96, 40)
    chunky(cv, (2, 2, 94, 37), 15, "#3b3f9a", "#2a2c78", ("#191a52", "#121340"), 3, outline=2.25)
    made["plate"] = cv
    # Chip: a small chunky tag.
    cv = Canvas(48, 28)
    chunky(cv, (2, 2, 46, 25.5), 9, "#4b55c8", "#3540a8", ("#222a7a", "#1a2066"), 2.5, outline=2.0, drop=False)
    made["chip"] = cv
    # Slot: a sunk navy well with a thick outline, its inner top shaded.
    cv = Canvas(64, 64)
    body = shape(cv, (2.5, 2.5, 61.5, 61.5), 14)
    cv.over(hexc(INK), cover(body - 2.5))
    cv.over(vgrad(cv, hexc("#232870"), hexc("#363c98"), 3, 61), cover(body))
    inner = body + 1.5
    cv.over(hexc("#000000"), cover(inner) * np.clip((14 - cv.y) / 10, 0, 1) * 0.18)
    cv.over(hexc("#ffffff"), (cover(body) - cover(body + 1.25)) * np.clip((cv.y - 50) / 10, 0, 1) * 0.8)
    made["slot"] = cv
    for name, cv in made.items():
        cv.save(os.path.join(out, f"{name}.png"))
    return made
# The UI effects' particle textures (templates/styles/shared/fx/, 128 px):
# white and grey with alpha, so a `<Particles>` node's ink colours them.
# A `sparkle` (a four-point star with a glowing core), a `glint` (a hot
# dot in a soft halo), a faceted `shard`, a `coin` seen face on (a rim, a
# sunk face and a lit stamp), a `ray` (a soft beam, tall) and a `ring` (a
# shock ring with a glow).

def fx_canvas():
    n = 128
    ys, xs = np.mgrid[0:n, 0:n]
    return (xs + 0.5) / n * 2 - 1, (ys + 0.5) / n * 2 - 1


def fx_save(path, grey, alpha):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    g = np.clip(grey, 0, 1)
    a = np.clip(alpha, 0, 1)
    px = np.round(np.stack([g, g, g, a], -1) * 255).astype(np.uint8)
    Image.fromarray(px, "RGBA").save(path, optimize=True)


def fx_art(out):
    x, y = fx_canvas()
    r = np.hypot(x, y)
    edge = np.clip((1 - r) / 0.08, 0, 1)
    # sparkle: two crossed tapering beams, a smaller diagonal pair, a core
    beam = lambda u, v: np.exp(-np.abs(v) / (0.035 + 0.05 * np.clip(1 - np.abs(u), 0, 1))) * np.clip(1 - np.abs(u), 0, 1) ** 1.6
    star = np.maximum(beam(x, y), beam(y, x))
    d1, d2 = (x + y) / np.sqrt(2), (x - y) / np.sqrt(2)
    star = np.maximum(star, 0.45 * np.maximum(beam(d1 * 1.8, d2), beam(d2 * 1.8, d1)))
    core = np.exp(-(r / 0.16) ** 2)
    halo = np.exp(-(r / 0.42) ** 2) * 0.35
    a = np.clip(star + core + halo, 0, 1) * edge
    fx_save(os.path.join(out, "sparkle.png"), np.ones_like(a), a)
    # glint
    a = np.clip(np.exp(-(r / 0.22) ** 2) + np.exp(-(r / 0.6) ** 2) * 0.45, 0, 1) * edge
    fx_save(os.path.join(out, "glint.png"), np.ones_like(a), a)
    # shard: a tall rhombus, its left facets lit
    q = np.abs(x) / 0.42 + np.abs(y) / 0.92
    inside = np.clip((1 - q) * 30, 0, 1)
    lit = np.where(x < 0, 1.0, 0.62) * np.where(y < 0, 1.0, 0.85)
    rimline = np.clip(1 - np.abs(q - 0.93) * 18, 0, 1)
    grey = np.clip(lit + rimline * 0.5, 0, 1)
    fx_save(os.path.join(out, "shard.png"), grey, inside)
    # coin: a rim, a sunk face, a lit stamp and a gloss
    disc = np.clip((0.94 - r) * 40, 0, 1)
    rim = np.clip((r - 0.74) * 40, 0, 1)
    shade = 0.78 - 0.28 * y
    stamp = np.clip(1 - (np.abs(x) / 0.28 + np.abs(y) / 0.42), 0, 1) > 0
    grey = np.where(rim > 0.5, 0.96 - 0.3 * (y + 1) / 2, shade)
    grey = np.where(stamp, 1.0, grey)
    grey = grey + np.exp(-(((x + 0.32) / 0.22) ** 2 + ((y + 0.38) / 0.16) ** 2)) * 0.35
    fx_save(os.path.join(out, "coin.png"), grey, disc)
    # ray: a soft vertical beam, brightest in the middle
    a = np.exp(-(x / 0.16) ** 2) * np.clip(1 - np.abs(y), 0, 1) ** 0.8 + np.exp(-(x / 0.04) ** 2) * np.clip(1 - np.abs(y), 0, 1) * 0.6
    fx_save(os.path.join(out, "ray.png"), np.ones_like(a), np.clip(a, 0, 1))
    # ring
    a = np.exp(-((r - 0.82) / 0.035) ** 2) + np.exp(-((r - 0.82) / 0.14) ** 2) * 0.4
    fx_save(os.path.join(out, "ring.png"), np.ones_like(a), np.clip(a, 0, 1) * edge)


if __name__ == "__main__":
    repo = sys.argv[1] if len(sys.argv) > 1 else "."
    default_art(os.path.join(repo, "templates/styles/shared/art/frames"))
    scifi_art(os.path.join(repo, "templates/styles/scifi/art/frames"))
    cartoon_art(os.path.join(repo, "templates/styles/cartoon/art/frames"))
    fx_art(os.path.join(repo, "templates/styles/shared/fx"))
