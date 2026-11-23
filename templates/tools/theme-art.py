#!/usr/bin/env python3
"""The theme art generator's reference: draws a theme's nine-slice frame art
from its art spec (a material, a shape, ornaments and the resolved palette).
og-cook's `cook/ui_art.rs` is its port and must draw the same pixels: the
spec is this file, the cook is the port (`--parity` writes the fixture the
port's test checks).

    python3 templates/tools/theme-art.py SPEC.json OUT_DIR    the PNGs and images.json
    python3 templates/tools/theme-art.py --sheet SPEC.json... OUT.png   a contact sheet
    python3 templates/tools/theme-art.py --parity [repo]      writes crates/og-cook/tests/ui_art_parity.json

Needs numpy and Pillow.

The spec (one JSON object; every colour is a colour ref: "#rrggbb", a
palette role, "top" or "bottom" (the piece's face), or {"mix": [a, b, t]}):

    scale     pixels per point (4)
    palette   roles: button, primary, danger, disabled, panel, plate, chip,
              slot, field ([top, bottom] each), and outline, shadow, orn,
              accent, header (one colour each)
    material  the surface program's layers, each optional (see MATERIAL_KEYS)
    shape     {family: rounded | cut | cut4 | pixel | brush, radius:
              {control, panel, chip, slot}, cut, step, wobble}
    ornament  {button | panel | plate | slot | chip: {kind, color, size}}
    panel     {header: {height, color, rule}} (optional)

Every piece is drawn on its collapsed canvas: its nine-slice insets with no
middle, pixel i at i / scale points, so the column at the left inset and
the row at the top inset lie exactly on the stretch line. That column and
row are then repeated to fill the middle band. Every stretched row and
column of the output is therefore identical, whatever the material's
texture: grain, brushing and ornament live in the corners and edges and
never smear.

Portability: only + - * / sqrt, floor and comparisons, in f64, in the
order written here (no exp, sin, pow, hypot or library blur), so the Rust
port draws the same bytes; noise is an integer hash; a blur is three box
passes; a byte is floor(v * 255 + 0.5).
"""
import json, os, sys
import numpy as np

MATERIAL_KEYS = ("drop", "glow", "outline", "lip", "rim", "face", "grain", "edge", "bevel",
                 "gloss", "toplight", "hairline", "shade", "sunk", "hover", "press")


# --------------------------------------------------------------------------
# colours

def hexc(h):
    h = h.lstrip("#")
    a = int(h[6:8], 16) / 255 if len(h) == 8 else 1.0
    return np.array([int(h[0:2], 16) / 255, int(h[2:4], 16) / 255, int(h[4:6], 16) / 255, a])


class Colors:
    """Resolves colour refs against the palette and the current piece's face."""

    def __init__(self, palette):
        self.palette = palette
        self.top = self.bottom = None

    def __call__(self, ref):
        if isinstance(ref, dict):
            a, b, t = ref["mix"]
            ca, cb = self(a), self(b)
            return ca * (1 - t) + cb * t
        if ref == "top":
            return self.top
        if ref == "bottom":
            return self.bottom
        if isinstance(ref, str) and ref.startswith("#"):
            return hexc(ref)
        v = self.palette[ref]
        return hexc(v if isinstance(v, str) else v[0])

    def pair(self, role):
        v = self.palette[role]
        return hexc(v[0]), hexc(v[1])


def mix(a, b, t):
    return a * (1 - t) + b * t


# --------------------------------------------------------------------------
# the collapsed canvas

class Canvas:
    def __init__(self, scale, insets, band=(4, 4)):
        insets = [int(np.ceil(v)) for v in insets]
        t, r, b, l = insets
        self.S = scale
        self.insets = insets
        self.band = band
        self.w, self.h = l + r, t + b
        nx, ny = self.w * scale + 1, self.h * scale + 1
        xs = np.arange(nx, dtype=np.float64) / scale
        ys = np.arange(ny, dtype=np.float64) / scale
        self.x = np.broadcast_to(xs[None, :], (ny, nx)).copy()
        self.y = np.broadcast_to(ys[:, None], (ny, nx)).copy()
        self.rgba = np.zeros((ny, nx, 4))

    def over(self, color, alpha):
        """Lays `color` (rgba, or per pixel) over the canvas at coverage `alpha`."""
        c = np.broadcast_to(np.asarray(color, np.float64), self.rgba.shape)
        a = clamp(alpha)[..., None] * c[..., 3:4]
        base = self.rgba
        ba = base[..., 3:4]
        out_a = a + ba * (1 - a)
        rgb = (c[..., :3] * a + base[..., :3] * ba * (1 - a)) / np.maximum(out_a, 1e-6)
        self.rgba = np.concatenate([rgb, out_a], axis=-1)

    def bytes(self):
        """The finished image: the stretch column and row repeated over the band, as RGBA bytes."""
        t, r, b, l = self.insets
        S = self.S
        px = np.floor(clamp(self.rgba) * 255 + 0.5).astype(np.uint8)
        cols = np.concatenate([np.arange(l * S), np.full(self.band[0] * S, l * S, int), np.arange(l * S + 1, self.w * S + 1)])
        rows = np.concatenate([np.arange(t * S), np.full(self.band[1] * S, t * S, int), np.arange(t * S + 1, self.h * S + 1)])
        return px[rows][:, cols]


def clamp(v, lo=0.0, hi=1.0):
    return np.minimum(np.maximum(v, lo), hi)


def cover(cv, d):
    """Antialiased coverage of the inside of a distance field."""
    return clamp(0.5 - d * cv.S)


def ramp(v):
    """clamp(v) eased by smoothstep."""
    t = clamp(v)
    return t * t * (3 - 2 * t)


def vgrad(cv, top, bottom, y0, y1):
    t = clamp((cv.y - y0) / max(y1 - y0, 1e-6))[..., None]
    return top * (1 - t) + bottom * t


# --------------------------------------------------------------------------
# shapes

def rounded(x, y, box, radius):
    x0, y0, x1, y1 = box
    cx, cy, hw, hh = (x0 + x1) / 2, (y0 + y1) / 2, (x1 - x0) / 2, (y1 - y0) / 2
    r = min(radius, hw, hh)
    qx, qy = np.abs(x - cx) - hw + r, np.abs(y - cy) - hh + r
    mx, my = np.maximum(qx, 0), np.maximum(qy, 0)
    return np.sqrt(mx * mx + my * my) + np.minimum(np.maximum(qx, qy), 0) - r


def chamfer(d, x, y, box, cuts, cut):
    x0, y0, x1, y1 = box
    cx, cy, hw, hh = (x0 + x1) / 2, (y0 + y1) / 2, (x1 - x0) / 2, (y1 - y0) / 2
    for c in cuts:
        sx = -1 if "l" in c else 1
        sy = -1 if "t" in c else 1
        d = np.maximum(d, (sx * (x - cx) + sy * (y - cy) - (hw + hh - cut)) * 0.7071067811865476)
    return d


def hash01(ix, iy, seed):
    """A value in [0, 1) for each integer lattice point (u32 arithmetic)."""
    h = (ix.astype(np.int64) * 374761393 + iy.astype(np.int64) * 668265263 + seed * 2246822519) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    h = h ^ (h >> 16)
    return h.astype(np.float64) / 4294967296.0


def noise(x, y, cell, seed):
    """Smooth value noise, 0 to 1, one lattice point every `cell` points."""
    u, v = x / cell, y / cell
    fx0, fy0 = np.floor(u), np.floor(v)
    ix, iy = fx0.astype(np.int64), fy0.astype(np.int64)
    fx, fy = u - fx0, v - fy0
    fx = fx * fx * (3 - 2 * fx)
    fy = fy * fy * (3 - 2 * fy)
    a, b = hash01(ix, iy, seed), hash01(ix + 1, iy, seed)
    c, d = hash01(ix, iy + 1, seed), hash01(ix + 1, iy + 1, seed)
    top = a + (b - a) * fx
    bot = c + (d - c) * fx
    return top + (bot - top) * fy


class Shape:
    def __init__(self, spec):
        self.family = spec.get("family", "rounded")
        self.radii = spec.get("radius", {})
        self.cut = spec.get("cut", 0.0)
        self.step = spec.get("step", 1.0)
        self.wobble = spec.get("wobble", 0.0)
        self.seed = spec.get("seed", 7)

    def radius(self, role):
        return self.radii.get(role, self.radii.get("control", 6.0))

    def corner(self, role):
        """How far a corner's shaping reaches in: what a nine-slice inset must clear."""
        return max(self.radius(role), self.cut if self.family in ("cut", "cut4") else 0.0)

    def coords(self, cv):
        if self.family != "pixel":
            return cv.x, cv.y
        s = self.step
        return (np.floor(cv.x / s) + 0.5) * s, (np.floor(cv.y / s) + 0.5) * s

    def dist(self, cv, box, role, scale_cut=1.0):
        x, y = self.coords(cv)
        r = self.radius(role)
        if self.family == "cut":
            d = rounded(x, y, box, min(r, 2.0))
            return chamfer(d, x, y, box, ("tl", "br"), self.cut * scale_cut)
        if self.family == "cut4":
            d = rounded(x, y, box, 0.0)
            return chamfer(d, x, y, box, ("tl", "tr", "br", "bl"), self.cut * scale_cut)
        d = rounded(x, y, box, r)
        if self.family == "brush":
            # an uneven hand-drawn edge: the boundary pushed in and out by noise
            d = d + (noise(x, y, 5.0, self.seed) - 0.5) * 2 * self.wobble
        return d


# --------------------------------------------------------------------------
# blur and light

def box_blur(a, r, axis):
    if r <= 0:
        return a
    n = a.shape[axis]
    pad = [(0, 0), (0, 0)]
    pad[axis] = (r, r)
    p = np.pad(a, pad)
    acc = np.zeros_like(a)
    for k in range(2 * r + 1):
        acc = acc + (p[k:k + n] if axis == 0 else p[:, k:k + n])
    return acc / (2 * r + 1)


def blur(a, sigma_px):
    """Three box passes along each axis: about a gaussian of `sigma_px`."""
    if sigma_px <= 0:
        return a
    w = np.sqrt(4 * sigma_px * sigma_px + 1)
    r = int(np.floor((w - 1) / 2 + 0.5))
    for axis in (1, 0):
        for _ in range(3):
            a = box_blur(a, r, axis)
    return a


def shifted(a, dy):
    """`a` moved down by `dy` whole pixels, zeros above."""
    if dy <= 0:
        return a
    out = np.zeros_like(a)
    out[dy:] = a[:-dy]
    return out


def grad(d):
    """Central differences along each axis (one-sided at the edges)."""
    gx = np.zeros_like(d)
    gy = np.zeros_like(d)
    gx[:, 1:-1] = (d[:, 2:] - d[:, :-2]) / 2
    gx[:, 0] = d[:, 1] - d[:, 0]
    gx[:, -1] = d[:, -1] - d[:, -2]
    gy[1:-1] = (d[2:] - d[:-2]) / 2
    gy[0] = d[1] - d[0]
    gy[-1] = d[-1] - d[-2]
    return gx, gy


def lit(d):
    """How much an edge faces the light (above, a little left): 0 to 1."""
    gx, gy = grad(d)
    n = np.sqrt(gx * gx + gy * gy) + 1e-9
    return clamp(0.5 + 0.5 * ((0 - gy / n) * 0.92 - (gx / n) * 0.38))


def shadow(cv, d, dy, sigma, alpha, color):
    a = cover(cv, d)
    a = shifted(a, int(np.floor(dy * cv.S + 0.5)))
    a = blur(a, sigma * cv.S)
    cv.over(color, a * alpha)


def glow(cv, d, sigma, alpha, color):
    a = blur(cover(cv, d), sigma * cv.S)
    cv.over(color, a * alpha * (1 - cover(cv, d)))


# --------------------------------------------------------------------------
# the surface program

def grain_field(cv, g, box):
    """The grain's signed shading, -1 to 1, by its kind."""
    kind, cell, seed = g["kind"], g.get("cell", 16.0), g.get("seed", 1)
    x, y = cv.x, cv.y
    if kind == "noise":
        return noise(x, y, cell, seed) * 2 - 1
    if kind == "brushed":
        # streaks along x: fine noise across rows only
        return noise(x * 0 + 0.5, y, cell, seed) * 2 - 1
    if kind == "wood":
        # rings along x (a horizontal stretch keeps them): a triangle wave
        # of y bent by noise across rows, eased
        t = y / cell + noise(x * 0 + 0.5, y, cell * 3, seed) * 1.5
        f = t - np.floor(t)
        tri = np.abs(f - 0.5) * 2
        return tri * tri * (3 - 2 * tri) * 2 - 1
    if kind == "weave":
        cx = (x / cell - np.floor(x / cell)) - 0.5
        cy = (y / cell - np.floor(y / cell)) - 0.5
        return (np.abs(cx) - np.abs(cy)) * 2
    if kind == "stone":
        n = noise(x, y, cell, seed) * 0.65 + noise(x, y, cell * 0.37, seed + 1) * 0.35
        return n * 2 - 1
    if kind == "wash":
        # ink: soft blotches that pool towards the bottom
        n = noise(x, y, cell, seed)
        return n * 2 - 1 + vgrad(cv, np.array([-0.3]), np.array([0.3]), box[1], box[3])[..., 0]
    raise SystemExit(f"unknown grain {kind}")


def surface(cv, C, M, d, box, sunk=False):
    """Paints a face of the material over distance field `d` (inside the outline and lip)."""
    x0, y0, x1, y1 = box
    face_d = d
    rim = M.get("rim")
    if rim:
        L = lit(d)[..., None]
        cv.over(C(rim["lo"]) * (1 - L) + C(rim["hi"]) * L, cover(cv, d))
        face_d = d + rim["width"]
    f = M.get("face", {})
    alpha = f.get("alpha", 1.0)
    top, bottom = C.top.copy(), C.bottom.copy()
    top[3] = top[3] * alpha
    bottom[3] = bottom[3] * alpha
    split = f.get("split")
    if split is None:
        col = vgrad(cv, top, bottom, y0, y1)
    else:
        sy = y0 + (y1 - y0) * split
        below = vgrad(cv, mix(top, bottom, split), bottom, sy, y1)
        col = np.where((cv.y < sy)[..., None], top, below)
    cv.over(col, cover(cv, face_d))
    g = M.get("grain")
    if g:
        n = grain_field(cv, g, box)
        cv.over(C(g.get("light", "#ffffff")), cover(cv, face_d) * clamp(n) * g["strength"])
        cv.over(C(g.get("color", "#000000")), cover(cv, face_d) * clamp(0 - n) * g["strength"])
    e = M.get("edge")
    if e:
        cv.over(C(e["color"]), cover(cv, face_d) * clamp(1 + (face_d + 0.5) / e["width"]) * e["alpha"])
    bv = M.get("bevel")
    if bv:
        w = bv["width"]
        ring = cover(cv, face_d) - cover(cv, face_d + w)
        L = lit(face_d)
        light = (L > 0.5).astype(np.float64)
        cv.over(hexc("#ffffff"), ring * light * bv["hi"])
        cv.over(hexc("#000000"), ring * (1 - light) * bv["lo"])
        cv.over(C(bv.get("inner", "top")), cover(cv, face_d + w) * bv.get("inner_alpha", 0.0))
    gl = M.get("gloss")
    if gl and not sunk:
        ins = gl.get("inset", 3.5)
        h = (y1 - y0) * gl["band"]
        gb = rounded(cv.x, cv.y, (x0 + ins, y0 + ins * 0.7, x1 - ins, y0 + ins * 0.7 + h * 0.62),
                     max(gl.get("radius", 12.0) - ins, 2.0))
        a = vgrad(cv, np.array([gl["alpha"][0]]), np.array([gl["alpha"][1]]), y0 + ins * 0.7, y0 + h)[..., 0]
        cv.over(hexc("#ffffff"), cover(cv, gb) * a * cover(cv, face_d))
    tl = M.get("toplight")
    if tl:
        ring = cover(cv, face_d) - cover(cv, face_d + tl.get("width", 1.0))
        cv.over(hexc("#ffffff"), ring * clamp((y0 + tl["depth"] - cv.y) / tl["depth"]) * tl["alpha"])
    hl = M.get("hairline")
    if hl:
        ring = cover(cv, face_d) - cover(cv, face_d + hl.get("width", 1.0))
        cv.over(C(hl.get("color", "#ffffff")), ring * hl["alpha"])
    sh = M.get("shade")
    if sh and not sunk:
        fr = (y1 - y0) * sh["frac"]
        cv.over(hexc("#000000"), cover(cv, face_d) * clamp((cv.y - (y1 - fr)) / fr) * sh["alpha"])
    sk = M.get("sunk")
    if sk and sunk:
        cv.over(hexc("#000000"), cover(cv, face_d) * clamp((y0 + sk["depth"] - cv.y) / sk["depth"]) * sk["alpha"])
        cv.over(hexc("#ffffff"), (cover(cv, d) - cover(cv, d + 1.0)) * clamp((cv.y - (y1 - 10)) / 10) * sk.get("lip", 0.0))
    return face_d


def piece(cv, C, M, shape, role, box, face, lip_h=0.0, sunk=False, drop=True, orn=None, scale_cut=1.0):
    """A whole piece: drop, glow, outline, lip, then the surface and its ornament."""
    x0, y0, x1, y1 = box
    C.top, C.bottom = face
    body = shape.dist(cv, box, role, scale_cut)
    dr = M.get("drop")
    if drop and dr:
        shadow(cv, body, dr["dy"], dr["blur"], dr["alpha"], C(dr.get("color", "shadow")))
    gw = M.get("glow")
    if gw and drop:
        glow(cv, body, gw["blur"], gw["alpha"], C(gw["color"]))
    ol = M.get("outline")
    if ol:
        cv.over(C(ol.get("color", "outline")), cover(cv, body - ol["width"]))
    fbox = box
    if lip_h > 0:
        lp = M["lip"]
        cv.over(vgrad(cv, C(lp["top"]), C(lp["bottom"]), y1 - lip_h - 2, y1), cover(cv, body))
        fbox = (x0, y0, x1, y1 - lip_h)
        body = shape.dist(cv, fbox, role, scale_cut)
    face_d = surface(cv, C, M, body, fbox, sunk)
    if orn and orn.get("kind", "none") != "none":
        ornament(cv, C, face_d, orn, fbox)
    return face_d


# --------------------------------------------------------------------------
# ornaments: drawn in the corners (and along an edge only as a constant line)

def stroke(cv, x, y, ax, ay, bx, by, w):
    """Coverage of a segment of width `w`."""
    px, py = x - ax, y - ay
    vx, vy = bx - ax, by - ay
    t = clamp((px * vx + py * vy) / (vx * vx + vy * vy))
    dx, dy = px - vx * t, py - vy * t
    return clamp(0.5 + (w / 2 - np.sqrt(dx * dx + dy * dy)) * 4)


def ornament(cv, C, d, orn, box):
    kind = orn["kind"]
    col = C(orn.get("color", "orn"))
    a = orn.get("alpha", 1.0)
    size = orn.get("size", 10.0)
    x0, y0, x1, y1 = box
    inside = cover(cv, d)
    inner = d + orn.get("inset", 4.0)
    corners = [(x0, y0, 1, 1), (x1, y0, -1, 1), (x1, y1, -1, -1), (x0, y1, 1, -1)]
    if kind == "hairline":
        cv.over(col, (cover(cv, inner) - cover(cv, inner + 0.75)) * a)
    elif kind == "stitch":
        ring = cover(cv, inner) - cover(cv, inner + 0.9)
        # dashes: a step along the perimeter's x + y, the same along a stretched edge's band
        t = (cv.x + cv.y) / orn.get("period", 3.0)
        dash = ((t - np.floor(t)) < 0.55).astype(np.float64)
        cv.over(col, ring * dash * a)
    elif kind == "brackets":
        for cx, cy, sx, sy in corners:
            px, py = (cv.x - cx) * sx, (cv.y - cy) * sy
            arm = ((px >= 3) & (px < 3 + size) & (py >= 3) & (py < 4.5)) | ((py >= 3) & (py < 3 + size) & (px >= 3) & (px < 4.5))
            cv.over(col, arm * inside * a)
    elif kind == "studs":
        for cx, cy, sx, sy in corners:
            ux, uy = cv.x - (cx + sx * 6.5), cv.y - (cy + sy * 6.5)
            r = np.sqrt(ux * ux + uy * uy)
            vx, vy = ux + 0.6, uy + 0.6
            cv.over(hexc("#000000"), clamp(2.6 - r) * 0.5 * inside)
            cv.over(col, clamp(2.0 - r) * a * inside)
            cv.over(hexc("#ffffff"), clamp(0.9 - np.sqrt(vx * vx + vy * vy)) * 0.7 * inside)
    elif kind == "filigree":
        cv.over(col, (cover(cv, inner) - cover(cv, inner + 0.75)) * 0.5 * a)
        for cx, cy, sx, sy in corners:
            px, py = (cv.x - cx) * sx, (cv.y - cy) * sy
            dia = np.abs(px - 9) + np.abs(py - 9)
            cv.over(col, clamp(3.2 - dia) * a)
            ux, uy = px - 9, py - 9
            curl = np.abs(np.sqrt(ux * ux + uy * uy) - 6.5)
            cv.over(col, clamp(0.6 - curl) * ((px < 9) | (py < 9)) * 0.9 * a)
    elif kind == "runes":
        cv.over(col, (cover(cv, inner) - cover(cv, inner + 0.6)) * 0.35 * a)
        for cx, cy, sx, sy in corners:
            px, py = (cv.x - cx) * sx, (cv.y - cy) * sy
            m = (stroke(cv, px, py, 7, 6, 7, 13, 1.1) + stroke(cv, px, py, 7, 9, 10.5, 6.5, 1.1)
                 + stroke(cv, px, py, 7, 9, 10.5, 12, 1.1))
            cv.over(col, clamp(m) * a * inside)
    elif kind == "brush-corner":
        for cx, cy, sx, sy in corners[:1] + corners[2:3]:
            px, py = (cv.x - cx) * sx, (cv.y - cy) * sy
            # a tapered stroke along the corner, thick at its root
            along = clamp(1 - (px + py - 6) / (2 * size))
            m = stroke(cv, px, py, 4, 4 + size, 4 + size, 4, 0.5 + 3.0 * along)
            m = m * clamp((px + py - 4) * 2)
            cv.over(col, m * a * inside)
    elif kind == "notch":
        x, y = cv.x - (x0 + 3), cv.y - (y0 + 3)
        cv.over(col, ((x + y < size * 0.7) & (x >= 0) & (y >= 0)) * cover(cv, d + 1.0) * a)
    elif kind == "bar":
        # an accent bar on the leading edge, the same on every row of the stretch band
        inset = min(size, (y1 - y0) * 0.3)
        y_mid0, y_mid1 = y0 + inset, y1 - inset
        cov = clamp((cv.x - (x0 + 2.5)) * 4 + 0.5) * clamp((x0 + 4.0 - cv.x) * 4 + 0.5)
        cov = cov * clamp((cv.y - y_mid0) * 4 + 0.5) * clamp((y_mid1 - cv.y) * 4 + 0.5)
        cv.over(col, cov * a)
    elif kind == "tab":
        cov = clamp((cv.x - (x0 + 10)) * 4 + 0.5) * clamp((x0 + 10 + size * 2.4 - cv.x) * 4 + 0.5)
        cov = cov * clamp((cv.y - y0) * 4 + 0.5) * clamp((y0 + 1.5 - cv.y) * 4 + 0.5)
        cv.over(col, cov * a)
    else:
        raise SystemExit(f"unknown ornament {kind}")


# --------------------------------------------------------------------------
# the standard pieces

STATES = ("", "-hover", "-pressed")


def draw(spec):
    """Every piece of the spec: [(name, RGBA bytes, image entry)], the image
    entries as a pack's `images` writes them (slice in pixels, width in points)."""
    S = spec.get("scale", 4)
    C = Colors(spec["palette"])
    M = spec["material"]
    for k in M:
        if k not in MATERIAL_KEYS and k != "about":
            raise SystemExit(f"material: unknown key {k}")
    shape = Shape(spec["shape"])
    orns = spec.get("ornament", {})
    lip = M.get("lip", {}).get("height", 0.0) if M.get("lip") else 0.0
    press = M.get("press", {}).get("dy", 2.0)
    hover = M.get("hover", {"toward": "#ffffff", "t": 0.12})
    out = []

    def put(name, cv, fill=True, tint=False, band=None):
        t, r, b, l = cv.insets
        e = {"src": f"art/{name}.png", "slice": [t * S, r * S, b * S, l * S], "width": [t, r, b, l]}
        if not fill:
            e["fill"] = False
        if tint:
            e["tint"] = True
        out.append((name, cv.bytes(), e))

    def orn(role):
        o = orns.get(role)
        return o if o and o.get("kind", "none") != "none" else None

    m = 2.5
    # buttons: their insets clear the corner, the lip and a margin
    cr = shape.corner("control")
    bi = [max(12.0, cr + 4), max(14.0, cr + 6), max(12.0, cr + 4) + lip, max(14.0, cr + 6)]
    for name, role in [("button", "button"), ("button-primary", "primary"), ("button-danger", "danger")]:
        for st in STATES:
            cv = Canvas(S, bi)
            top, bottom = C.pair(role)
            if st == "-hover":
                tw = C(hover["toward"])
                top, bottom = mix(top, tw, hover["t"]), mix(bottom, tw, hover["t"] * 0.7)
            pressed = st == "-pressed"
            if lip > 0:
                l2 = max(lip - press, 1.0) if pressed else lip
                y_top = m + (lip - l2)
            else:
                l2 = 0.0
                y_top = m + (press * 0.4 if pressed else 0.0)
            piece(cv, C, M, shape, "control", (m, y_top, cv.w - m, cv.h - m), (top, bottom), lip_h=l2,
                  drop=not pressed, orn=orn("button"))
            put(f"{name}{st}", cv)
    cv = Canvas(S, bi)
    piece(cv, C, M, shape, "control", (m, m, cv.w - m, cv.h - m), C.pair("disabled"), lip_h=lip)
    put("button-disabled", cv)

    # panel: a larger corner, an optional header band in its top inset
    pc = max(24.0, shape.corner("panel") + 12)
    head = spec.get("panel", {}).get("header")
    hh = head["height"] if head else 0.0
    cv = Canvas(S, [pc + hh, pc, pc + lip * 0.5, pc])
    box = (3.0, 3.0, cv.w - 3.0, cv.h - 3.0)
    fd = piece(cv, C, M, shape, "panel", box, C.pair("panel"), lip_h=lip * 0.5, orn=orn("panel"), scale_cut=1.6)
    if head:
        band = cover(cv, fd) * clamp((3.0 + hh - cv.y) * S)
        hc = C(head["color"])
        cv.over(vgrad(cv, mix(hc, hexc("#ffffff"), head.get("lift", 0.15)), hc, 3.0, 3.0 + hh), band)
        if head.get("rule"):
            cv.over(C(head["rule"]), cover(cv, fd) * clamp(1 - np.abs(cv.y - (3.0 + hh - 0.5)) / 0.9))
    put("panel", cv)

    # plate: a read-out pill
    pr = min(shape.corner("control"), 14.0)
    pi = max(12.0, pr + 4)
    cv = Canvas(S, [pi, pi + 2, pi, pi + 2])
    piece(cv, C, M, shape, "control", (2.0, 2.0, cv.w - 2.0, cv.h - 2.0 - lip * 0.5), C.pair("plate"),
          lip_h=lip * 0.5, orn=orn("plate"))
    put("plate", cv)

    # chip: a small tag
    ci = max(9.0, shape.corner("chip") + 3)
    cv = Canvas(S, [ci, ci + 1, ci, ci + 1])
    piece(cv, C, M, shape, "chip", (2.0, 2.0, cv.w - 2.0, cv.h - 2.0 - lip * 0.4), C.pair("chip"),
          lip_h=lip * 0.4, drop=False, orn=orn("chip"), scale_cut=0.6)
    put("chip", cv)

    # slot and field: sunk wells
    si = max(16.0, shape.corner("slot") + 6)
    cv = Canvas(S, [si, si, si, si])
    piece(cv, C, M, shape, "slot", (2.0, 2.0, cv.w - 2.0, cv.h - 2.0), C.pair("slot"), sunk=True, drop=False,
          orn=orn("slot"), scale_cut=0.8)
    put("slot", cv)
    fi = max(12.0, min(shape.corner("control"), 10.0) + 4)
    cv = Canvas(S, [fi, fi, fi, fi])
    piece(cv, C, M, shape, "control", (2.0, 2.0, cv.w - 2.0, cv.h - 2.0), C.pair("field"), sunk=True,
          drop=False, scale_cut=0.8)
    put("field", cv)

    # divider: a rule fading at its ends, the accent in its middle band
    cv = Canvas(S, [4, 64, 4, 64], band=(4, 0))
    ends = clamp(np.minimum(cv.x, cv.w - cv.x) / 56)
    ends = ends * ends * (3 - 2 * ends)
    line = clamp(1 - np.abs(cv.y - 4) / 0.6)
    under = clamp(1 - np.abs(cv.y - 5.2) / 0.6)
    cv.over(C("shadow"), under * ends * 0.4)
    cv.over(C(spec.get("divider", "orn")), line * ends)
    put("divider", cv, fill=True)

    # badge: a notification pill
    cv = Canvas(S, [13, 14, 14, 14], band=(2, 0))
    body = rounded(cv.x, cv.y, (2.0, 1.0, cv.w - 2.0, cv.h - 1.0), 12.5)
    shadow(cv, body, 1.0, 0.9, 0.5, C("shadow"))
    cv.over(hexc("#ffffff"), cover(cv, body))
    face = body + 1.75
    cv.over(vgrad(cv, hexc("#d83834"), hexc("#a00c1a"), 3.0, cv.h - 2.0), cover(cv, face))
    cv.over(hexc("#ffffff"), cover(cv, face + 1.2) * clamp((11 - cv.y) / 7) * 0.35)
    put("badge", cv)

    # tint pieces: grey shading with alpha that the element's fill colours
    cv = Canvas(S, [si, si, si, si])
    body = shape.dist(cv, (2.0, 2.0, cv.w - 2.0, cv.h - 2.0), "slot", 0.8)
    ring = cover(cv, body) - cover(cv, body + 2.25)
    inner = clamp(1 + (body + 2.25) / 7) * cover(cv, body + 2.25)
    grey = 0.78 + 0.22 * lit(body)
    cv.over(np.stack([grey, grey, grey, np.ones_like(grey)], -1), clamp(ring + inner * 0.45))
    put("slot-rim", cv, fill=False, tint=True)
    cv = Canvas(S, [ci, ci + 1, ci, ci + 1])
    body = shape.dist(cv, (1.0, 1.0, cv.w - 1.0, cv.h - 1.0), "chip", 0.6)
    cv.over(np.array([0.16, 0.16, 0.16, 1.0]), cover(cv, body))
    rim = (0.17 + 0.36 * lit(body))[..., None]
    cv.over(np.concatenate([rim, rim, rim, np.ones_like(rim)], -1), cover(cv, body + 1.0))
    cv.over(vgrad(cv, np.array([0.33, 0.33, 0.33, 1]), np.array([0.24, 0.24, 0.24, 1]), 3.0, cv.h - 3.0),
            cover(cv, body + 2.0))
    put("chip-bevel", cv, tint=True)
    return out


# --------------------------------------------------------------------------
# outputs

def fnv(data):
    """FNV-1a 64 of bytes (the parity fixture's digest)."""
    h = 0xcbf29ce484222325
    for b in data:
        h = ((h ^ b) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def digest(px):
    hdr = np.array([px.shape[1], px.shape[0]], dtype="<u4").tobytes()
    return fnv(hdr + px.tobytes())


def write(spec, out_dir):
    from PIL import Image
    os.makedirs(os.path.join(out_dir, "art"), exist_ok=True)
    images = {}
    for name, px, e in draw(spec):
        Image.fromarray(px, "RGBA").save(os.path.join(out_dir, "art", f"{name}.png"), optimize=True)
        images[name] = e
    with open(os.path.join(out_dir, "images.json"), "w") as f:
        json.dump(images, f, indent=1)
        f.write("\n")


def sheet(specs, out):
    """A contact sheet: one row per spec, its pieces at 1x on a mid grey."""
    from PIL import Image
    rows = [draw(s) for s in specs]
    pad = 8
    width = max(sum(px.shape[1] + pad for _, px, _ in r) for r in rows) + pad
    height = sum(max(px.shape[0] for _, px, _ in r) + pad for r in rows) + pad
    canvas = Image.new("RGBA", (width, height), (128, 138, 148, 255))
    y = pad
    for r in rows:
        x = pad
        for _, px, _ in r:
            im = Image.fromarray(px, "RGBA")
            canvas.alpha_composite(im, (x, y))
            x += px.shape[1] + pad
        y += max(px.shape[0] for _, px, _ in r) + pad
    canvas.save(out)


# Specs the parity fixture covers: every material and shape the port draws,
# each with every ornament at least once.
def parity_specs():
    here = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(here, "theme-art-specs.json")) as f:
        return json.load(f)


if __name__ == "__main__":
    args = sys.argv[1:]
    if args and args[0] == "--sheet":
        sheet([json.load(open(p)) for p in args[1:-1]], args[-1])
    elif args and args[0] == "--parity":
        repo = args[1] if len(args) > 1 else "."
        fixture = {}
        for name, spec in parity_specs().items():
            fixture[name] = {n: digest(px) for n, px, _ in draw(spec)}
        path = os.path.join(repo, "crates", "og-cook", "tests", "ui_art_parity.json")
        with open(path, "w") as f:
            json.dump(fixture, f, indent=1, sort_keys=True)
            f.write("\n")
        print(path)
    elif len(args) == 2:
        write(json.load(open(args[0])), args[1])
    else:
        raise SystemExit(__doc__)
