#!/usr/bin/env python3
"""Theme prototype: draws one theme's frame art from its recipe's art spec
(material x shape x ornament x the resolved palette), reusing
templates/tools/make-frame-art.py's distance-field toolkit.

    python art.py spec.json   -> writes PNGs into spec["out"], prints the pack's images JSON

Materials are surface programs (glass, metal, paper, plastic, wood, pixel);
shape is a radius, cut corners or a pixel staircase; ornament is drawn in
the corners only (where nine-slice keeps it whole). Texture that must not
stretch (grain) is kept low-frequency or constant along the stretched axis.
"""
import importlib.util, json, os, sys
import numpy as np

here = os.path.dirname(os.path.abspath(__file__))
spec_ = importlib.util.spec_from_file_location(
    "mfa", os.path.join(here, "..", "..", "templates", "tools", "make-frame-art.py"))
mfa = importlib.util.module_from_spec(spec_)
spec_.loader.exec_module(mfa)
Canvas, shape, cover, lit, vgrad, shadow, glow, hexc, mix = (
    mfa.Canvas, mfa.shape, mfa.cover, mfa.lit, mfa.vgrad, mfa.shadow, mfa.glow, mfa.hexc, mfa.mix)
S = mfa.S


def noise(cv, scale, seed):
    """Smooth value noise, 0..1, at `scale` points per cell."""
    rng = np.random.default_rng(seed)
    gw, gh = int(cv.w / scale) + 3, int(cv.h / scale) + 3
    g = rng.random((gh, gw))
    x, y = cv.x / scale, cv.y / scale
    x0, y0 = np.floor(x).astype(int), np.floor(y).astype(int)
    fx, fy = x - x0, y - y0
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a, b = g[y0, x0], g[y0, x0 + 1]
    c, d = g[y0 + 1, x0], g[y0 + 1, x0 + 1]
    return (a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + d * fx) * fy


def body_shape(cv, box, sh):
    """The piece's signed distance under the recipe's shape language."""
    fam = sh["family"]
    r = sh["radius"]
    if fam == "cut":
        return shape(cv, box, min(r, 2), ("tl", "br"), sh["cut"])
    if fam == "cut4":
        return shape(cv, box, 0, ("tl", "tr", "br", "bl"), sh["cut"])
    if fam == "pixel":
        # a staircase: the rounded box sampled on a whole-point grid
        d = shape(cv, box, r)
        return d
    return shape(cv, box, r)


def pixelate(cv, step):
    """Snap the canvas to `step`-point blocks (nearest), so edges are a staircase."""
    k = int(step * S)
    a = cv.rgba
    small = a[k // 2::k, k // 2::k]
    cv.rgba = np.repeat(np.repeat(small, k, 0), k, 1)[: a.shape[0], : a.shape[1]]


def surface(cv, d, top, bottom, y0, y1, mat, P, seed=1, sunk=False):
    """Paints a face of `mat` over distance field `d`."""
    face = cover(d)
    L = lit(cv, d)
    if mat == "glass":
        cv.over(vgrad(cv, hexc(top, 0.9), hexc(bottom, 0.9), y0, y1), face)
        cv.over(hexc("#ffffff"), (cover(d) - cover(d + 1.0)) * 0.18)
        cv.over(hexc("#ffffff"), (cover(d) - cover(d + 1.0)) * np.clip((y0 + 6 - cv.y) / 6, 0, 1) * 0.25)
        return
    if mat == "metal":
        rim = hexc(P["rim_lo"]) * (1 - L) + hexc(P["rim_hi"]) * L
        cv.over(rim, face)
        inner = d + 2.0
        cv.over(vgrad(cv, hexc(top), hexc(bottom), y0, y1), cover(inner))
        # brushed: streaks constant along x, so a horizontal stretch keeps them
        rng = np.random.default_rng(seed)
        rows = rng.random(cv.rgba.shape[0])[:, None] * np.ones((1, cv.rgba.shape[1]))
        cv.over(hexc("#ffffff"), cover(inner) * (rows - 0.5).clip(0, 1) * 0.10)
        cv.over(hexc("#000000"), cover(inner) * (0.5 - rows).clip(0, 1) * 0.10)
        if not sunk:
            cv.over(hexc("#ffffff"), (cover(inner) - cover(inner + 1.0)) * np.clip((L[..., 0] - 0.5) * 2, 0, 1) * 0.5)
        return
    if mat == "paper":
        cv.over(vgrad(cv, hexc(top), hexc(bottom), y0, y1), face)
        n = noise(cv, 22, seed)
        cv.over(hexc(P["paper_shade"]), face * (n - 0.5).clip(0, 1) * 0.12)
        # burnt / aged edge: darker towards the outline
        cv.over(hexc(P["paper_shade"]), face * np.clip(1 + (d + 0.5) / 4, 0, 1) * 0.25)
        return
    if mat == "wood":
        cv.over(vgrad(cv, hexc(top), hexc(bottom), y0, y1), face)
        grain = 0.5 + 0.5 * np.sin(cv.y * 1.7 + 2.2 * noise(cv, 14, seed) * 3)
        cv.over(hexc(P["grain"]), face * grain * 0.28)
        inner = d + 2.0
        cv.over(hexc("#ffffff"), (cover(d) - cover(inner)) * np.clip((L[..., 0] - 0.5) * 2, 0, 1) * 0.25)
        cv.over(hexc("#000000"), (cover(d) - cover(inner)) * np.clip((0.5 - L[..., 0]) * 2, 0, 1) * 0.35)
        return
    if mat == "plastic":
        split = y0 + (y1 - y0) * 0.55
        f = np.where((cv.y < split)[..., None], hexc(top), vgrad(cv, hexc(mix(top, bottom, 0.55)), hexc(bottom), split, y1))
        cv.over(f, face)
        gb = d + 3.0
        band = cover(gb) * np.clip((y0 + (y1 - y0) * 0.42 - cv.y) / ((y1 - y0) * 0.3), 0, 1)
        cv.over(hexc("#ffffff"), band * 0.4)
        return
    if mat == "pixel":
        cv.over(hexc(bottom), face)
        cv.over(hexc(top), cover(d + 2.0))
        # 1-point bevel: lit above-left, shade below-right
        cv.over(hexc("#ffffff"), (cover(d + 1.0) - cover(d + 2.0)) * (L[..., 0] > 0.5) * 0.45)
        cv.over(hexc("#000000"), (cover(d + 1.0) - cover(d + 2.0)) * (L[..., 0] <= 0.5) * 0.35)
        return
    raise SystemExit(f"unknown material {mat}")


def ornament(cv, d, kind, color, box):
    x0, y0, x1, y1 = box
    corners = [(x0, y0, 1, 1), (x1, y0, -1, 1), (x1, y1, -1, -1), (x0, y1, 1, -1)]
    inner = d + 4.0
    if kind == "none":
        return
    if kind == "hairline":
        cv.over(hexc(color), (cover(inner) - cover(inner + 0.75)) * 0.7)
    elif kind == "stitch":
        line = cover(inner) - cover(inner + 0.9)
        dash = (np.sin((cv.x + cv.y) * 1.6) > 0.1).astype(float)
        cv.over(hexc(color), line * dash * 0.8)
    elif kind == "brackets":
        for cx, cy, sx, sy in corners:
            px, py = (cv.x - cx) * sx, (cv.y - cy) * sy
            arm = ((px >= 3) & (px < 13) & (py >= 3) & (py < 4.5)) | ((py >= 3) & (py < 13) & (px >= 3) & (px < 4.5))
            cv.over(hexc(color), arm * 1.0)
    elif kind == "studs":
        for cx, cy, sx, sy in corners:
            r = np.hypot(cv.x - (cx + sx * 6.5), cv.y - (cy + sy * 6.5))
            cv.over(hexc("#000000"), np.clip(2.6 - r, 0, 1) * 0.5)
            cv.over(hexc(color), np.clip(2.0 - r, 0, 1))
            cv.over(hexc("#ffffff"), np.clip(0.9 - np.hypot(cv.x - (cx + sx * 6.5) + 0.6, cv.y - (cy + sy * 6.5) + 0.6), 0, 1) * 0.7)
    elif kind == "filigree":
        cv.over(hexc(color), (cover(inner) - cover(inner + 0.75)) * 0.5)
        for cx, cy, sx, sy in corners:
            px, py = (cv.x - cx) * sx, (cv.y - cy) * sy
            dia = np.abs(px - 9) + np.abs(py - 9)
            cv.over(hexc(color), np.clip(3.2 - dia, 0, 1))
            curl = np.abs(np.hypot(px - 9, py - 9) - 6.5)
            cv.over(hexc(color), np.clip(0.6 - curl, 0, 1) * ((px < 9) | (py < 9)) * 0.9)
    else:
        raise SystemExit(f"unknown ornament {kind}")


def piece(cv, box, sh, mat, P, top, bottom, outline, lip=0, sunk=False, drop=True, orn=None, seed=1):
    x0, y0, x1, y1 = box
    body = body_shape(cv, box, sh)
    if drop and P["drop"] > 0:
        shadow(cv, body, 1.5, 1.4, P["drop"], P["shadow"])
    if outline:
        cv.over(hexc(outline), cover(body - P["outline_w"]))
    if lip:
        cv.over(hexc(P["lip"] if isinstance(P.get("lip"), str) else mix(bottom, "#000000", 0.45)), cover(body))
        body = body_shape(cv, (x0, y0, x1, y1 - lip), sh)
    surface(cv, body, top, bottom, y0, y1 - lip, mat, P, seed, sunk)
    if sunk:
        cv.over(hexc("#000000"), cover(body) * np.clip((y0 + 10 - cv.y) / 8, 0, 1) * 0.35)
    if orn:
        ornament(cv, body, orn[0], orn[1], (x0, y0, x1, y1 - lip))
    return body


def draw(spec):
    out = spec["out"]
    os.makedirs(out, exist_ok=True)
    sh, mat, P = spec["shape"], spec["material"], spec["palette"]
    orn = spec["ornament"]
    pix = mat == "pixel"
    lip = spec.get("lip", 0)
    made, images = {}, {}

    def put(name, cv, inset, fill=True):
        if pix:
            pixelate(cv, 1.0)
        cv.save(os.path.join(out, f"{name}.png"))
        im = {"src": f"art/{name}.png", "slice": [v * S for v in inset], "width": inset}
        if not fill:
            im["fill"] = False
        images[name] = im

    corner = max(sh["radius"], sh.get("cut", 0))
    bi = [max(12, corner + 4), max(14, corner + 6), max(12, corner + 4) + lip, max(14, corner + 6)]
    for name, key in [("button", "button"), ("button-primary", "primary"), ("button-danger", "danger")]:
        for st in ["", "-hover", "-pressed"]:
            cv = Canvas(96, 52)
            top, bottom = P[key]
            if st == "-hover":
                top, bottom = mix(top, "#ffffff", 0.12), mix(bottom, "#ffffff", 0.08)
            dy = 2.0 if st == "-pressed" else 0.0
            l = max(lip - 3, 1) if (st == "-pressed" and lip) else lip
            piece(cv, (2.5, 2.5 + dy + (lip - l), 93.5, 49.5), sh, mat, P, top, bottom, P["outline"], lip=l,
                  drop=st != "-pressed", orn=(orn["button"], P["orn"]) if orn["button"] != "none" else None)
            put(f"{name}{st}", cv, bi)
    cv = Canvas(96, 52)
    piece(cv, (2.5, 2.5, 93.5, 49.5), sh, mat, P, *P["disabled"], P["outline"], lip=lip)
    put("button-disabled", cv, bi)

    pc = max(24, corner + 12)
    cv = Canvas(128, 128)
    piece(cv, (4, 3, 124, 124), dict(sh, radius=sh["radius"] * 1.6, cut=sh.get("cut", 0) * 1.6), mat, P,
          *P["panel"], P["outline"], orn=(orn["panel"], P["orn"]), seed=3)
    if spec.get("header"):
        hd = body_shape(cv, (4, 3, 124, 124), dict(sh, radius=sh["radius"] * 1.6, cut=sh.get("cut", 0) * 1.6)) + P["outline_w"]
        cv.over(hexc(P["header"]), cover(hd) * np.clip((22 - cv.y) * S, 0, 1))
        cv.over(hexc(P["orn"]), cover(hd) * np.clip(1 - np.abs(cv.y - 22) / 0.6, 0, 1))
    put("panel", cv, [pc + (20 if spec.get("header") else 0), pc, pc, pc])

    cv = Canvas(96, 36)
    piece(cv, (2, 2, 94, 34), dict(sh, radius=min(sh["radius"], 14)), mat, P, *P["plate"], P["outline"],
          orn=(orn["plate"], P["orn"]) if orn["plate"] != "none" else None, seed=5)
    pi = max(12, min(corner, 14) + 4)
    put("plate", cv, [pi, pi + 2, pi, pi + 2])

    cv = Canvas(48, 28)
    piece(cv, (2, 2, 46, 26), dict(sh, radius=min(sh["radius"], 9), cut=min(sh.get("cut", 0), 6)), mat, P,
          *P["chip"], P["outline"], drop=False, seed=7)
    put("chip", cv, [10, 11, 10, 11])

    cv = Canvas(64, 64)
    piece(cv, (2, 2, 62, 62), dict(sh, radius=min(sh["radius"], 12)), mat, P, *P["slot"], P["outline"], sunk=True,
          drop=False, orn=(orn["slot"], P["orn"]) if orn["slot"] != "none" else None, seed=9)
    put("slot", cv, [18, 18, 18, 18])

    cv = Canvas(96, 44)
    piece(cv, (2, 2, 94, 42), dict(sh, radius=min(sh["radius"], 10)), mat, P, *P["field"], P["outline"], sunk=True,
          drop=False, seed=11)
    put("field", cv, [14, 14, 14, 14])
    return images


if __name__ == "__main__":
    spec = json.load(open(sys.argv[1]))
    print(json.dumps(draw(spec)))
